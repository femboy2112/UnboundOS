//! Snark Screen of Death — spec section 9.

use core::panic::PanicInfo;

pub fn from_rust_panic(_info: &PanicInfo) -> ! {
    // Disable interrupts, emit structured record to serial, render to
    // framebuffer, halt. Stub.
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
