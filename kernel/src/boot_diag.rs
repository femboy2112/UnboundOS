//! Boot diagnostic buffer — spec §3.9 fallback for the §1.6
//! heartbeat.
//!
//! When the UART probe fails, heartbeat events are recorded into a
//! reserved 4 KiB ring of bytes. Once framebuffer output becomes
//! available (M2), `heartbeat::finalize_framebuffer_fallback` dumps
//! `BOOT_NO_SERIAL`, `BOOT_HEARTBEAT_BUFFER_PRESENT`, and the buffer
//! contents to the screen.
//!
//! The buffer is append-only and saturates on overflow; older
//! entries are preserved.

use core::sync::atomic::{AtomicUsize, Ordering};

const DIAG_CAPACITY: usize = 4096;

/// Recorded when the COM1 probe fails and the diagnostic buffer is
/// the only heartbeat sink.
pub const BOOT_NO_SERIAL_MARKER: &str = "BOOT_NO_SERIAL\n";

/// Source-visible marker for the future M2 framebuffer dump path.
pub const BOOT_HEARTBEAT_BUFFER_PRESENT: &str = "BOOT_HEARTBEAT_BUFFER_PRESENT\n";

static mut DIAG_BUFFER: [u8; DIAG_CAPACITY] = [0u8; DIAG_CAPACITY];
static DIAG_CURSOR: AtomicUsize = AtomicUsize::new(0);

/// Append `s` to the buffer. Truncates silently on overflow.
pub fn record(s: &str) {
    let bytes = s.as_bytes();
    if bytes.is_empty() {
        return;
    }
    // Reserve the destination range with fetch_add so concurrent
    // recorders cannot overlap. Boot is single-threaded so this is
    // overkill, but it keeps the abstraction sound for later.
    let start = DIAG_CURSOR.fetch_add(bytes.len(), Ordering::Relaxed);
    if start >= DIAG_CAPACITY {
        return;
    }
    let copy_len = core::cmp::min(bytes.len(), DIAG_CAPACITY - start);
    // SAFETY: start..start+copy_len was just reserved by fetch_add
    // and is uniquely owned by this caller until it returns.
    unsafe {
        let dst = core::ptr::addr_of_mut!(DIAG_BUFFER).cast::<u8>().add(start);
        core::ptr::copy_nonoverlapping(bytes.as_ptr(), dst, copy_len);
    }
}

/// Append a u64 as `0x` + 16 uppercase hex digits.
pub fn record_hex_u64(v: u64) {
    let mut buf = [0u8; 18];
    buf[0] = b'0';
    buf[1] = b'x';
    for i in 0..16 {
        let nibble = ((v >> ((15 - i) * 4)) & 0xF) as u8;
        buf[2 + i] = if nibble < 10 {
            b'0' + nibble
        } else {
            b'A' + (nibble - 10)
        };
    }
    // SAFETY: every byte written above is ASCII 0..=9 / A..=F or
    // the literal '0' / 'x'.
    let s = unsafe { core::str::from_utf8_unchecked(&buf) };
    record(s);
}

/// Number of bytes currently recorded (clamped to `DIAG_CAPACITY`).
pub fn used() -> usize {
    DIAG_CURSOR.load(Ordering::Acquire).min(DIAG_CAPACITY)
}

/// Read-only view of the recorded bytes. Used by the M2 framebuffer
/// dump path.
///
/// # Safety
///
/// Caller must guarantee no concurrent `record` calls are in flight.
/// The boot phase is single-threaded, so the natural caller (the
/// framebuffer fallback dump) satisfies this trivially.
pub unsafe fn snapshot() -> &'static [u8] {
    let len = used();
    // SAFETY: cursor advances monotonically; bytes 0..len have been
    // written by `record`. Caller-asserted exclusive access.
    unsafe {
        let ptr = core::ptr::addr_of!(DIAG_BUFFER).cast::<u8>();
        core::slice::from_raw_parts(ptr, len)
    }
}
