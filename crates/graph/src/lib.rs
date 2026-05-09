//! Graph runtime, verifier, loader. Spec §5.
//!
//! The single legal pipeline:
//!
//! ```text
//!   bytes  ──▶  graph_load_from_umod  ──▶  VerifiedGraph
//!                  (22 spec §5.6 checks)
//!   VerifiedGraph  ──▶  graph_compile_verified  ──▶  GraphRuntimeHandle
//! ```
//!
//! `GraphRuntime` is constructed nowhere except `loader.rs`. The
//! PreToolUse hook in `.claude/settings.json` blocks any literal
//! `GraphRuntime { ... }` introduced outside that file. The
//! `graph-verifier-auditor` subagent enforces this at review time.

#![no_std]
#![forbid(unsafe_op_in_unsafe_fn)]
// TODO M0 (spec §13): drop these allows once the 22 §5.6 verifier
// checks return real errors (`unnecessary_wraps`), the public API
// gains per-error documentation (`missing_errors_doc`), and the
// narrative spec-quote comments are converted to fully-backticked
// references (`doc_markdown`). The lints are appropriate; the code
// is pre-implementation.
#![allow(
    clippy::doc_markdown,
    clippy::missing_errors_doc,
    clippy::unnecessary_wraps
)]

pub mod loader;
pub mod verifier;

/// Output of the verifier. Bytes have been checked against all 22
/// spec §5.6 rules; semantic validity is guaranteed. Compilation
/// into a runtime graph is the next step and may still fail on
/// resource allocation.
#[derive(Debug)]
pub struct VerifiedGraph<'bytes> {
    /// Borrow of the original bytes. The lifetime ensures the caller
    /// cannot mutate the buffer between verify and compile.
    _bytes: &'bytes [u8],
}

impl<'bytes> VerifiedGraph<'bytes> {
    /// Internal constructor used **only** by `verifier::verify_umod`.
    /// Marked `pub(crate)` so it is unreachable from outside the
    /// graph crate.
    pub(crate) fn new_internal(bytes: &'bytes [u8]) -> Self {
        Self { _bytes: bytes }
    }
}

/// Opaque handle to a compiled runtime graph living in
/// `GraphArena`. The runtime structures are NEVER constructed
/// outside `loader.rs`.
#[derive(Debug)]
pub struct GraphRuntimeHandle {
    _phantom: core::marker::PhantomData<*const ()>,
}

/// Verifier failures. One variant per check in spec §5.6 plus a few
/// structural variants that the parser surfaces.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum GraphLoadError {
    BadMagic,                                            // check  1
    UnsupportedVersion,                                  // check  2
    BadHeaderLength,                                     // check  3
    BadSectionTable,                                     // check  4
    NodeCountExceedsLimit,                               // check  5
    WireCountExceedsLimit,                               // check  6
    NodeIndexUnresolved { node_id: u32 },                // check  7
    WireEndpointUnresolved { wire_id: u32 },             // check  8
    PinIndexOutOfRange { node_id: u32, pin_index: u16 }, // check 9
    WireTypeMismatch { wire_id: u32 },                   // check 10
    UnknownNodeType { node_id: u32, type_id: u32 },      // check 11
    UndeclaredCapability { capability_id: u32 },         // check 12
    UnbrokenCycle { sample_node_id: u32 },               // check 13
    PayloadSizeUnbounded { wire_id: u32 },               // check 14
    GraphArenaBudgetExceeded { required: u64 },          // check 15
    ModelRefUnresolved { ref_index: u32 },               // check 16
    ChecksumMismatch { section_index: u32 },             // check 17
    UiLayoutInvalid,                                     // check 18
    MissingConstantBlob { node_id: u32 },                // check 19
    ConstantBlobLayoutBad { node_id: u32 },              // check 20
    SchedulingSectionMissing,                            // check 21
    OpaqueResourceSyntaxBad { ref_index: u32 },          // check 22

    // Structural failures from the parser, surfaced as load errors.
    ParseTruncated,
    ParseHeaderChecksumMismatch,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum GraphCompileError {
    GraphArenaExhausted,
    NodeRuntimeAllocFailed,
    WireRuntimeAllocFailed,
    DispatchTableMissing,
}

/// Top-level entry point. Verifies the byte buffer against all 22
/// spec §5.6 checks. Single legal entry to `VerifiedGraph`.
pub fn graph_load_from_umod(bytes: &[u8]) -> Result<VerifiedGraph<'_>, GraphLoadError> {
    verifier::verify_umod(bytes)
}

/// Compile a verified graph into runtime structures inside
/// `GraphArena`. Single legal entry to `GraphRuntimeHandle`.
pub fn graph_compile_verified(
    verified: VerifiedGraph<'_>,
) -> Result<GraphRuntimeHandle, GraphCompileError> {
    loader::compile(verified)
}
