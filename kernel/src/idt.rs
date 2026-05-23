//! Interrupt Descriptor Table — spec §3.5, §9.2.
//!
//! Installs a 256-entry IDT. Every vector is wired to a handler so an
//! unexpected interrupt parks the CPU rather than triple-faulting.
//! Spec §3.5 mandates entries for at least
//! #DE (0), #UD (6), #DF (8), #GP (13), #PF (14); these vectors
//! are reachable through this table once `install` runs.
//!
//! The M0-required fatal vectors route through the SSOD diagnostic
//! surface. Full SSOD record formatting lands later, but these handlers
//! already fill `DiagnosticContext` and avoid blind halts.

use crate::ssod::{self, DiagnosticContext};
use core::sync::atomic::{AtomicBool, Ordering};

#[derive(Copy, Clone)]
#[repr(C)]
struct IdtEntry {
    offset_low: u16,
    selector: u16,
    ist: u8,
    type_attr: u8,
    offset_mid: u16,
    offset_high: u32,
    reserved: u32,
}

const _: () = assert!(core::mem::size_of::<IdtEntry>() == 16);

#[repr(C, packed)]
struct IdtPointer {
    limit: u16,
    base: u64,
}

const _: () = assert!(core::mem::size_of::<IdtPointer>() == 10);

/// Spec §9.3 reference shape. Reproduced here so the IDT module is
/// self-contained until the SSOD subsystem owns it.
#[derive(Copy, Clone)]
#[repr(C)]
pub struct InterruptStackFrame {
    pub instruction_pointer: u64,
    pub code_segment: u64,
    pub cpu_flags: u64,
    pub stack_pointer: u64,
    pub stack_segment: u64,
}

impl IdtEntry {
    const fn empty() -> Self {
        Self {
            offset_low: 0,
            selector: 0,
            ist: 0,
            type_attr: 0,
            offset_mid: 0,
            offset_high: 0,
            reserved: 0,
        }
    }

    fn set_handler(&mut self, handler: u64, selector: u16) {
        self.offset_low = (handler & 0xFFFF) as u16;
        self.selector = selector;
        self.ist = 0;
        // P=1, DPL=0, type=0xE (64-bit interrupt gate)
        self.type_attr = 0x8E;
        self.offset_mid = ((handler >> 16) & 0xFFFF) as u16;
        self.offset_high = (handler >> 32) as u32;
        self.reserved = 0;
    }
}

static mut IDT: [IdtEntry; 256] = [IdtEntry::empty(); 256];
static IDT_INSTALLED: AtomicBool = AtomicBool::new(false);

/// Vectors that push an error code onto the stack per Intel SDM
/// Vol. 3A 6.13. Anything not in this list uses the no-error-code
/// signature.
const VECTORS_WITH_ERROR_CODE: [usize; 10] = [8, 10, 11, 12, 13, 14, 17, 21, 29, 30];

extern "x86-interrupt" fn halt_handler(_frame: InterruptStackFrame) {
    halt_loop();
}

extern "x86-interrupt" fn halt_handler_with_err(_frame: InterruptStackFrame, _err: u64) {
    halt_loop();
}

extern "x86-interrupt" fn halt_handler_double_fault(_frame: InterruptStackFrame, _err: u64) -> ! {
    halt_loop()
}

extern "x86-interrupt" fn divide_error_handler(frame: InterruptStackFrame) {
    kernel_panic_no_error(0, "divide_error", frame);
}

extern "x86-interrupt" fn invalid_opcode_handler(frame: InterruptStackFrame) {
    kernel_panic_no_error(6, "invalid_opcode", frame);
}

extern "x86-interrupt" fn double_fault_handler(frame: InterruptStackFrame, err: u64) -> ! {
    kernel_panic_with_error(8, "double_fault", frame, err)
}

extern "x86-interrupt" fn general_protection_fault_handler(frame: InterruptStackFrame, err: u64) {
    kernel_panic_with_error(13, "general_protection_fault", frame, err);
}

extern "x86-interrupt" fn page_fault_handler(frame: InterruptStackFrame, err: u64) {
    kernel_panic_with_error(14, "page_fault", frame, err);
}

fn kernel_panic_no_error(vector: u8, reason: &str, frame: InterruptStackFrame) -> ! {
    ssod::kernel_panic(reason, diagnostic_context(vector, frame, None))
}

fn kernel_panic_with_error(vector: u8, reason: &str, frame: InterruptStackFrame, err: u64) -> ! {
    ssod::kernel_panic(reason, diagnostic_context(vector, frame, Some(err)))
}

fn diagnostic_context(
    vector: u8,
    frame: InterruptStackFrame,
    error_code: Option<u64>,
) -> DiagnosticContext {
    DiagnosticContext {
        vector,
        instruction_pointer: frame.instruction_pointer,
        code_segment: frame.code_segment,
        cpu_flags: frame.cpu_flags,
        stack_pointer: frame.stack_pointer,
        stack_segment: frame.stack_segment,
        has_error_code: error_code.is_some(),
        error_code: error_code.unwrap_or(0),
        arena_fault: None,
    }
}

fn halt_loop() -> ! {
    loop {
        // SAFETY: hlt is always safe; the CPU parks until reset.
        unsafe {
            core::arch::asm!("cli; hlt", options(nomem, nostack, preserves_flags));
        }
    }
}

/// Install the IDT. After this returns, vectors 0..256 are wired to
/// halt handlers; vectors 0/6 use the no-error-code signature, 8
/// uses the diverging double-fault signature, and the rest of the
/// error-code vectors (10/11/12/13/14/17/21/29/30) use the
/// error-code signature.
///
/// Spec §3.5 conformance: the mandated vectors #DE/#UD/#DF/#GP/#PF
/// all have entries in the table.
///
/// # Safety
///
/// Called exactly once during boot, before STI. The kernel holds
/// exclusive ownership of the IDT memory for the lifetime of the
/// boot session.
pub unsafe fn install() {
    let cs: u16;
    // SAFETY: reading CS is always safe and reflects the segment
    // selector the bootloader handed us.
    unsafe {
        core::arch::asm!(
            "mov {0:x}, cs",
            out(reg) cs,
            options(nomem, nostack, preserves_flags),
        );
    }

    // SAFETY: boot phase, single-threaded, exclusive access to IDT.
    let idt = unsafe { &mut *core::ptr::addr_of_mut!(IDT) };

    for (vector, entry) in idt.iter_mut().enumerate() {
        if vector == 8 {
            entry.set_handler(halt_handler_double_fault as *const () as u64, cs);
        } else if VECTORS_WITH_ERROR_CODE.contains(&vector) {
            entry.set_handler(halt_handler_with_err as *const () as u64, cs);
        } else {
            entry.set_handler(halt_handler as *const () as u64, cs);
        }
    }

    idt[0].set_handler(divide_error_handler as *const () as u64, cs);
    idt[6].set_handler(invalid_opcode_handler as *const () as u64, cs);
    idt[8].set_handler(double_fault_handler as *const () as u64, cs);
    idt[13].set_handler(general_protection_fault_handler as *const () as u64, cs);
    idt[14].set_handler(page_fault_handler as *const () as u64, cs);

    let pointer = IdtPointer {
        limit: (core::mem::size_of::<[IdtEntry; 256]>() - 1) as u16,
        base: core::ptr::addr_of!(IDT) as u64,
    };

    // SAFETY: `pointer` lives for the duration of the lidt
    // instruction; the IDT is set up and immutable from this point
    // until the SSOD subsystem rewrites individual entries.
    unsafe {
        core::arch::asm!(
            "lidt [{}]",
            in(reg) &pointer,
            options(readonly, nostack, preserves_flags),
        );
    }

    IDT_INSTALLED.store(true, Ordering::Release);
}

/// Returns true iff `install` has completed.
pub fn is_installed() -> bool {
    IDT_INSTALLED.load(Ordering::Acquire)
}

/// M1 QEMU smoke selector for forced diagnostics faults.
///
/// Normal builds do nothing. Dedicated Makefile targets set
/// `UNBOUNDOS_FORCE_FAULT` while compiling the kernel image so QEMU can prove
/// the installed IDT routes the selected vector through SSOD.
pub fn trigger_forced_fault_from_env() {
    match option_env!("UNBOUNDOS_FORCE_FAULT") {
        Some("divide_error") => trigger_divide_error(),
        Some("invalid_opcode") => trigger_invalid_opcode(),
        Some("page_fault") => trigger_page_fault(),
        _ => {}
    }
}

fn trigger_divide_error() {
    // SAFETY: intentional M1 forced-fault smoke. Dividing by zero raises #DE
    // after the IDT is installed; the handler must route to SSOD.
    unsafe {
        core::arch::asm!(
            "xor edx, edx",
            "mov eax, 1",
            "xor ecx, ecx",
            "div ecx",
            options(nomem, nostack),
        );
    }
}

fn trigger_invalid_opcode() {
    // SAFETY: intentional M1 forced-fault smoke. `ud2` raises #UD and must be
    // caught by the installed invalid-opcode handler.
    unsafe {
        core::arch::asm!("ud2", options(nomem, nostack));
    }
}

fn trigger_page_fault() {
    const UNMAPPED_TEST_ADDRESS: usize = 0x4000_0000;
    // SAFETY: intentional M1 forced-fault smoke. The M0 Multiboot2 bootstrap
    // maps the first GiB only, so this read crosses into an unmapped page and
    // must route through #PF.
    unsafe {
        core::ptr::read_volatile(UNMAPPED_TEST_ADDRESS as *const u64);
    }
}
