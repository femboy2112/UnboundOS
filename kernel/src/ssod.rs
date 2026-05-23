//! Snark Screen of Death — spec section 9.

use crate::{boot_diag, serial};
use core::panic::PanicInfo;

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
        }
    }
}

pub fn from_rust_panic(_info: &PanicInfo) -> ! {
    kernel_panic("rust_panic", DiagnosticContext::rust_panic())
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
    emit_kv_str("arena_id", M0_ABSENT_CONTEXT);
    emit_kv_str("graph_id", M0_ABSENT_CONTEXT);
    emit_kv_str("node_id", M0_ABSENT_CONTEXT);
    emit_kv_str("model_id", M0_ABSENT_CONTEXT);
    emit_line(SSOD_END);

    halt_idle()
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
