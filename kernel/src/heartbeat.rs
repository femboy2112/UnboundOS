//! Boot heartbeat — spec §1.6.
//!
//! Every meaningful boot phase emits one heartbeat line through this
//! module. `emit*` functions write to both the UART (when available)
//! and the boot diagnostic buffer (always). When the UART probe
//! fails at startup, the buffer remains the only record until the
//! framebuffer comes up (spec §3.9).

use crate::{boot_diag, framebuffer::TextSurface, serial};

const NEWLINE: &str = "\n";

/// Initialize the UART, then either log normally or record
/// `BOOT_NO_SERIAL` per spec §3.9. Always returns; boot proceeds
/// even with a dead UART because the buffer fallback is enough to
/// trace the heartbeat.
pub fn init() {
    serial::init();
    if !serial::is_available() {
        boot_diag::record(boot_diag::BOOT_NO_SERIAL_MARKER);
    }
}

/// Emit a single heartbeat line (spec §1.6).
pub fn emit(line: &str) {
    serial::write_str(line);
    serial::write_str(NEWLINE);
    boot_diag::record(line);
    boot_diag::record(NEWLINE);
}

/// Emit `key=<u64-hex>` heartbeat line.
pub fn emit_kv_hex(key: &str, value: u64) {
    serial::write_str(key);
    serial::write_str("=");
    serial::write_hex_u64(value);
    serial::write_str(NEWLINE);
    boot_diag::record(key);
    boot_diag::record("=");
    boot_diag::record_hex_u64(value);
    boot_diag::record(NEWLINE);
}

/// Emit `key=<value>` heartbeat line for an ASCII string value.
pub fn emit_kv_str(key: &str, value: &str) {
    serial::write_str(key);
    serial::write_str("=");
    serial::write_str(value);
    serial::write_str(NEWLINE);
    boot_diag::record(key);
    boot_diag::record("=");
    boot_diag::record(value);
    boot_diag::record(NEWLINE);
}

/// Spec §3.9 fallback finalization. Called once framebuffer memory is
/// available. When the UART probe failed at startup, this prints
/// `BOOT_NO_SERIAL`, `BOOT_HEARTBEAT_BUFFER_PRESENT`, and the recorded
/// boot diagnostic buffer to the caller-provided framebuffer surface.
pub const FRAMEBUFFER_FALLBACK_MARKERS: [&str; 2] = [
    boot_diag::BOOT_NO_SERIAL_MARKER,
    boot_diag::BOOT_HEARTBEAT_BUFFER_PRESENT,
];

pub fn finalize_framebuffer_fallback(surface: Option<&mut TextSurface<'_>>) {
    if serial::is_available() {
        return;
    }
    let Some(surface) = surface else {
        return;
    };
    surface.write_str(FRAMEBUFFER_FALLBACK_MARKERS[0]);
    surface.write_str(FRAMEBUFFER_FALLBACK_MARKERS[1]);
    // SAFETY: fallback finalization is a boot-phase display operation. The
    // natural caller runs after framebuffer setup on the single boot CPU, so no
    // concurrent heartbeat recorder is in flight while the snapshot is read.
    let snapshot = unsafe { boot_diag::snapshot() };
    surface.write_bytes_ascii(snapshot);
}
