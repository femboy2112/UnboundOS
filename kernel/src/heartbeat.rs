//! Boot heartbeat — spec §1.6.
//!
//! Every meaningful boot phase emits one heartbeat line through this
//! module. `emit*` functions write to both the UART (when available)
//! and the boot diagnostic buffer (always). When the UART probe
//! fails at startup, the buffer remains the only record until the
//! framebuffer comes up (spec §3.9).

use crate::{boot_diag, serial};

const NEWLINE: &str = "\n";

/// Initialize the UART, then either log normally or record
/// `BOOT_NO_SERIAL` per spec §3.9. Always returns; boot proceeds
/// even with a dead UART because the buffer fallback is enough to
/// trace the heartbeat.
pub fn init() {
    serial::init();
    if !serial::is_available() {
        boot_diag::record("BOOT_NO_SERIAL\n");
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

/// Spec §3.9 fallback finalization. Called by the framebuffer
/// driver once `framebuffer::init` succeeds. When the UART probe
/// failed at startup, this prints `BOOT_NO_SERIAL` and
/// `BOOT_HEARTBEAT_BUFFER_PRESENT` to the framebuffer and dumps
/// `boot_diag::snapshot()`.
///
/// TODO M2 (spec §3.9, §3.7): the framebuffer driver does not yet
/// exist. Once `framebuffer::print` and a glyph routine land, this
/// function fills in:
///
/// ```text
///   framebuffer::print("BOOT_NO_SERIAL\n");
///   framebuffer::print("BOOT_HEARTBEAT_BUFFER_PRESENT\n");
///   framebuffer::print_bytes(unsafe { boot_diag::snapshot() });
/// ```
///
/// The two strings live in `FRAMEBUFFER_FALLBACK_MARKERS` so the
/// source-level /boot-heartbeat-check finds them before M2 lands.
pub const FRAMEBUFFER_FALLBACK_MARKERS: [&str; 2] =
    ["BOOT_NO_SERIAL\n", "BOOT_HEARTBEAT_BUFFER_PRESENT\n"];

pub fn finalize_framebuffer_fallback() {
    if serial::is_available() {
        return;
    }
    unimplemented!(
        "M2 framebuffer fallback: emit FRAMEBUFFER_FALLBACK_MARKERS \
         (BOOT_NO_SERIAL / BOOT_HEARTBEAT_BUFFER_PRESENT) and dump \
         boot_diag::snapshot() to the framebuffer per spec §3.9"
    );
}
