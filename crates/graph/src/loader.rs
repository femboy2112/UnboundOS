//! Graph runtime loader. **The only legal site of `GraphRuntime`
//! construction in the entire workspace.** Spec §5.7, §5.8.
//!
//! The PreToolUse hook in `.claude/settings.json` blocks any literal
//! `GraphRuntime { ... }` introduced outside this file. The
//! graph-verifier-auditor subagent verifies the same invariant at
//! review time. There is no test-only path, no dev-mode bypass, no
//! IDE editor shortcut. The IDE itself routes through
//! `verify_umod` → `compile`.

use crate::{GraphCompileError, GraphRuntimeHandle, VerifiedGraph};

/// Compile a verified graph into runtime structures inside
/// `GraphArena`. This is where the only legal `GraphRuntime { … }`
/// construction in the workspace lives.
pub(crate) fn compile(
    _verified: VerifiedGraph<'_>,
) -> Result<GraphRuntimeHandle, GraphCompileError> {
    // 1. Allocate node runtime table in GraphArena.
    // 2. Allocate wire runtime table in GraphArena.
    // 3. Resolve dispatch table for each node's node_type_id.
    // 4. Bind constant blobs to node descriptors.
    // 5. Construct GraphRuntime { node_runtimes, wire_runtimes,
    //    dispatch_table, scheduling_policy }.
    // 6. Wrap in opaque GraphRuntimeHandle.

    // Stub:
    Ok(GraphRuntimeHandle {
        _phantom: core::marker::PhantomData,
    })
}

// NOTE: `GraphRuntime` itself (the inner struct holding NodeRuntime
// and WireRuntime tables) is intentionally not yet defined here.
// When it is, its fields stay private to this module and `pub(super)`
// at most. The outer crate exposes only the opaque
// `GraphRuntimeHandle`.
