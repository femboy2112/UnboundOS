//! Snark Screen of Death — spec section 9.

use crate::{
    arena::{AllocError, ArenaFaultContext},
    boot_diag, serial,
};
use core::panic::PanicInfo;

pub const SSOD_REASON_BYTES: usize = 32;
pub const SSOD_REASON_BYTES_U32: u32 = 32;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct DiagnosticContext {
    pub vector: u8,
    pub instruction_pointer: u64,
    pub code_segment: u64,
    pub cpu_flags: u64,
    pub stack_pointer: u64,
    pub stack_segment: u64,
    pub has_error_code: bool,
    pub error_code: u64,
    pub arena_fault: Option<ArenaFaultContext>,
}

const SSOD_BEGIN: &str = "UNBOUNDOS_SSOD_BEGIN";
const SSOD_END: &str = "UNBOUNDOS_SSOD_END";
const M0_ABSENT_CONTEXT: &str = "none";

impl DiagnosticContext {
    pub const fn rust_panic() -> Self {
        Self {
            vector: 0xFF,
            instruction_pointer: 0,
            code_segment: 0,
            cpu_flags: 0,
            stack_pointer: 0,
            stack_segment: 0,
            has_error_code: false,
            error_code: 0,
            arena_fault: None,
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum SsodFaultFamily {
    CpuException = 1,
    RustPanic = 2,
    Arena = 3,
    Unknown = 255,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum SsodSnapshotError {
    ReasonTooLong { required: u32, available: u32 },
}

/// Fixed-width read-only SSOD facts for assistant explanation.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(C)]
pub struct SsodExplanationSnapshot {
    pub vector: u8,
    pub fault_family: u32,
    pub reason_len: u32,
    pub reason: [u8; SSOD_REASON_BYTES],
    pub instruction_pointer: u64,
    pub has_error_code: u8,
    pub error_code: u64,
}

impl SsodExplanationSnapshot {
    pub fn from_diagnostic(
        reason: &str,
        ctx: DiagnosticContext,
    ) -> Result<Self, SsodSnapshotError> {
        let reason_bytes = reason.as_bytes();
        if reason_bytes.len() > SSOD_REASON_BYTES {
            return Err(SsodSnapshotError::ReasonTooLong {
                required: len_to_u32(reason_bytes.len()),
                available: SSOD_REASON_BYTES_U32,
            });
        }

        let mut stored_reason = [0u8; SSOD_REASON_BYTES];
        stored_reason[..reason_bytes.len()].copy_from_slice(reason_bytes);
        Ok(Self {
            vector: ctx.vector,
            fault_family: fault_family_for(reason, ctx) as u32,
            reason_len: len_to_u32(reason_bytes.len()),
            reason: stored_reason,
            instruction_pointer: ctx.instruction_pointer,
            has_error_code: u8::from(ctx.has_error_code),
            error_code: ctx.error_code,
        })
    }
}

pub fn from_rust_panic(_info: &PanicInfo) -> ! {
    kernel_panic("rust_panic", DiagnosticContext::rust_panic())
}

pub fn from_arena_alloc_error(error: AllocError) -> ! {
    let mut ctx = DiagnosticContext::rust_panic();
    ctx.arena_fault = error.arena_fault_context();
    kernel_panic("arena_alloc_error", ctx)
}

/// M0-scope structured fatal diagnostic path.
///
/// Spec §9 requires fatal exceptions to pass through a structured diagnostic
/// record instead of silently halting. M0 emits the stable skeleton fields now;
/// later milestones fill arena/graph/node/model context when those systems
/// exist.
pub fn kernel_panic(reason: &str, ctx: DiagnosticContext) -> ! {
    // SAFETY: fatal path owns the CPU until reset.
    unsafe {
        core::arch::asm!("cli", options(nomem, nostack, preserves_flags));
    }

    emit_line(SSOD_BEGIN);
    emit_kv_str("format", "m0_ssod_v1");
    emit_kv_str("reason", reason);
    emit_kv_hex("vector", u64::from(ctx.vector));
    emit_kv_hex("rip", ctx.instruction_pointer);
    emit_kv_hex("cs", ctx.code_segment);
    emit_kv_hex("rflags", ctx.cpu_flags);
    emit_kv_hex("rsp", ctx.stack_pointer);
    emit_kv_hex("ss", ctx.stack_segment);
    if ctx.has_error_code {
        emit_kv_hex("error_code", ctx.error_code);
    } else {
        emit_kv_str("error_code", M0_ABSENT_CONTEXT);
    }
    if let Some(arena) = ctx.arena_fault {
        emit_arena_fault(arena);
    } else {
        emit_kv_str("arena_id", M0_ABSENT_CONTEXT);
    }
    emit_kv_str("graph_id", M0_ABSENT_CONTEXT);
    emit_kv_str("node_id", M0_ABSENT_CONTEXT);
    emit_kv_str("model_id", M0_ABSENT_CONTEXT);
    emit_line(SSOD_END);

    halt_idle()
}

fn emit_arena_fault(ctx: ArenaFaultContext) {
    emit_kv_str("arena_id", ctx.arena.as_str());
    emit_kv_hex("arena_requested", ctx.requested as u64);
    emit_kv_hex("arena_alignment", ctx.alignment as u64);
    emit_kv_hex("arena_base", ctx.base as u64);
    emit_kv_hex("arena_cursor", ctx.cursor as u64);
    emit_kv_hex("arena_limit", ctx.limit as u64);
}

const fn fault_family_for(reason: &str, ctx: DiagnosticContext) -> SsodFaultFamily {
    if str_eq(reason, "rust_panic") {
        SsodFaultFamily::RustPanic
    } else if str_eq(reason, "arena_alloc_error") || ctx.arena_fault.is_some() {
        SsodFaultFamily::Arena
    } else if ctx.vector != 0xFF {
        SsodFaultFamily::CpuException
    } else {
        SsodFaultFamily::Unknown
    }
}

const fn str_eq(left: &str, right: &str) -> bool {
    let left = left.as_bytes();
    let right = right.as_bytes();
    if left.len() != right.len() {
        return false;
    }

    let mut index = 0;
    while index < left.len() {
        if left[index] != right[index] {
            return false;
        }
        index += 1;
    }
    true
}

fn len_to_u32(len: usize) -> u32 {
    u32::try_from(len).unwrap_or(u32::MAX)
}

pub fn halt_idle() -> ! {
    loop {
        // SAFETY: hlt with interrupts disabled is safe and stable; the
        // CPU parks until reset.
        unsafe {
            core::arch::asm!("hlt");
        }
    }
}

fn emit_line(line: &str) {
    serial::write_str(line);
    serial::write_str("\n");
    boot_diag::record(line);
    boot_diag::record("\n");
}

fn emit_kv_str(key: &str, value: &str) {
    serial::write_str(key);
    serial::write_str("=");
    serial::write_str(value);
    serial::write_str("\n");
    boot_diag::record(key);
    boot_diag::record("=");
    boot_diag::record(value);
    boot_diag::record("\n");
}

fn emit_kv_hex(key: &str, value: u64) {
    serial::write_str(key);
    serial::write_str("=");
    serial::write_hex_u64(value);
    serial::write_str("\n");
    boot_diag::record(key);
    boot_diag::record("=");
    boot_diag::record_hex_u64(value);
    boot_diag::record("\n");
}
