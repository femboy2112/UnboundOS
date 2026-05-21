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
/// Step 6 owns the full SSOD record format. At Step 3, fatal IDT stubs still
/// route through this single diagnostic surface instead of halting directly,
/// preserving H10 and keeping boot failures inspectable through serial or the
/// boot-diagnostic buffer.
pub fn kernel_panic(reason: &str, ctx: DiagnosticContext) -> ! {
    // SAFETY: fatal path owns the CPU until reset.
    unsafe {
        core::arch::asm!("cli", options(nomem, nostack, preserves_flags));
    }

    emit_line("UNBOUNDOS_SSOD_STUB_BEGIN");
    emit_kv_str("reason", reason);
    emit_kv_hex("vector", u64::from(ctx.vector));
    emit_kv_hex("rip", ctx.instruction_pointer);
    emit_kv_hex("cs", ctx.code_segment);
    emit_kv_hex("rflags", ctx.cpu_flags);
    emit_kv_hex("rsp", ctx.stack_pointer);
    emit_kv_hex("ss", ctx.stack_segment);
    if ctx.has_error_code {
        emit_kv_hex("error_code", ctx.error_code);
    }
    emit_line("UNBOUNDOS_SSOD_STUB_END");

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
