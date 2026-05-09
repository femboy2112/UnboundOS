//! Authoritative diagnostic path. Spec §3.8 (serial logging) and
//! §1.6 (boot heartbeat).
//!
//! UART 16550 driver for COM1 (port 0x3F8). `init` runs an internal
//! loopback probe to detect whether the UART is wired. If the probe
//! passes, `is_available()` returns true and the write_* functions
//! emit characters. If the probe fails, every write is a silent
//! no-op; the heartbeat module records into `boot_diag` instead per
//! spec §3.9. Boot is never blind.

use core::sync::atomic::{AtomicBool, Ordering};

const COM1: u16 = 0x3F8;

const REG_DATA: u16 = COM1; // RBR/THR (DLAB=0); DLL (DLAB=1)
const REG_INTERRUPT_ENABLE: u16 = COM1 + 1; // IER (DLAB=0); DLM (DLAB=1)
const REG_FIFO_CTRL: u16 = COM1 + 2; // FCR
const REG_LINE_CTRL: u16 = COM1 + 3; // LCR (bit 7 = DLAB)
const REG_MODEM_CTRL: u16 = COM1 + 4; // MCR
const REG_LINE_STATUS: u16 = COM1 + 5; // LSR

const LSR_THR_EMPTY: u8 = 0x20;
const MCR_LOOPBACK: u8 = 0x10;

static UART_AVAILABLE: AtomicBool = AtomicBool::new(false);

#[inline]
unsafe fn outb(port: u16, val: u8) {
    // SAFETY: caller-asserted exclusive access to the I/O port.
    unsafe {
        core::arch::asm!(
            "out dx, al",
            in("dx") port,
            in("al") val,
            options(nomem, nostack, preserves_flags),
        );
    }
}

#[inline]
unsafe fn inb(port: u16) -> u8 {
    let val: u8;
    // SAFETY: caller-asserted exclusive access to the I/O port.
    unsafe {
        core::arch::asm!(
            "in al, dx",
            out("al") val,
            in("dx") port,
            options(nomem, nostack, preserves_flags),
        );
    }
    val
}

/// Initialize COM1 and probe with internal loopback. Sets
/// `UART_AVAILABLE` accordingly. Idempotent for the boot phase.
///
/// The probe writes `0xAE` through the loopback path and verifies it
/// reads back identically. A non-existent or stuck UART will fail
/// either step; in that case `is_available()` stays `false` and
/// every subsequent `write_*` is a no-op.
pub fn init() {
    // SAFETY: COM1 ports are reserved for the UART; the kernel is
    // single-threaded during early boot.
    unsafe {
        // 1. Disable interrupts.
        outb(REG_INTERRUPT_ENABLE, 0x00);
        // 2. Enable DLAB to set baud divisor.
        outb(REG_LINE_CTRL, 0x80);
        // 3. Set divisor for 115200 baud (divisor = 1).
        outb(REG_DATA, 0x01);
        outb(REG_INTERRUPT_ENABLE, 0x00);
        // 4. 8 bits, no parity, one stop bit (8N1); clear DLAB.
        outb(REG_LINE_CTRL, 0x03);
        // 5. Enable FIFO, clear buffers, 14-byte threshold.
        outb(REG_FIFO_CTRL, 0xC7);
        // 6. RTS/DSR/OUT2 set, IRQ off.
        outb(REG_MODEM_CTRL, 0x0B);

        // 7. Loopback probe — write a sentinel and read it back.
        outb(REG_MODEM_CTRL, MCR_LOOPBACK | 0x0A);
        outb(REG_DATA, 0xAE);
        let echo = inb(REG_DATA);
        if echo != 0xAE {
            // UART unusable. Leave loopback set so a stray write
            // cannot escape; flag unavailable so write_* no-ops.
            UART_AVAILABLE.store(false, Ordering::Release);
            return;
        }
        // 8. Disable loopback for normal operation.
        outb(REG_MODEM_CTRL, 0x0F);
        UART_AVAILABLE.store(true, Ordering::Release);
    }
}

/// Returns true iff the loopback probe in `init` passed.
pub fn is_available() -> bool {
    UART_AVAILABLE.load(Ordering::Acquire)
}

/// Spin until the transmit holding register is empty.
///
/// # Safety
///
/// Caller must hold `is_available() == true` and be single-threaded
/// with respect to other UART writers (boot phase).
unsafe fn wait_thr_empty() {
    // SAFETY: REG_LINE_STATUS is a read-only status port.
    while unsafe { inb(REG_LINE_STATUS) } & LSR_THR_EMPTY == 0 {
        core::hint::spin_loop();
    }
}

/// Write one byte. No-op when the UART is unavailable.
pub fn write_byte(b: u8) {
    if !is_available() {
        return;
    }
    // SAFETY: availability flag was checked; THR write is the only
    // legal way to send a byte through the 16550.
    unsafe {
        wait_thr_empty();
        outb(REG_DATA, b);
    }
}

/// Write a string. No-op when the UART is unavailable.
pub fn write_str(s: &str) {
    if !is_available() {
        return;
    }
    for &b in s.as_bytes() {
        write_byte(b);
    }
}

/// Write a u64 as `0x` + 16 uppercase hex digits.
pub fn write_hex_u64(v: u64) {
    if !is_available() {
        return;
    }
    write_str("0x");
    for shift in (0..16).rev() {
        let nibble = ((v >> (shift * 4)) & 0xF) as u8;
        let c = if nibble < 10 {
            b'0' + nibble
        } else {
            b'A' + (nibble - 10)
        };
        write_byte(c);
    }
}

/// Write a u64 as a decimal integer (1..=20 digits, no leading zero
/// on positive values; `0` writes a single `'0'`).
pub fn write_dec_u64(mut v: u64) {
    if !is_available() {
        return;
    }
    if v == 0 {
        write_byte(b'0');
        return;
    }
    let mut buf = [0u8; 20];
    let mut i = 0;
    while v != 0 {
        buf[i] = b'0' + (v % 10) as u8;
        v /= 10;
        i += 1;
    }
    while i > 0 {
        i -= 1;
        write_byte(buf[i]);
    }
}
