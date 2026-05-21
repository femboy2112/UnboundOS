//! Interrupt Descriptor Table — spec §3.5, §9.2.
//!
//! Installs a 256-entry IDT. Every vector is wired to a halt
//! handler so an unexpected interrupt parks the CPU rather than
//! triple-faulting. Spec §3.5 mandates entries for at least
//! #DE (0), #UD (6), #DF (8), #GP (13), #PF (14); these vectors
//! are reachable through this table once `install` runs.
//!
//! Differentiated SSOD reporting (spec §9.2 "Mandatory diagnostic
//! fields") is layered on top of this scaffolding by the
//! ssod-diagnostics-engineer subagent later. For now the priority
//! is making `UNBOUNDOS_IDT_OK` honest in the §1.6 heartbeat — i.e.
//! an IDT really is loaded by the time the line is emitted.

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
