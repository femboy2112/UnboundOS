//! Graph runtime loader. **The only legal site of `GraphRuntime`
//! construction in the entire workspace.** Spec §5.7, §5.8.
//!
//! `graph-verifier-auditor` verifies the same invariant at review time. There
//! is no test-only path, no dev-mode bypass, no IDE editor shortcut. The IDE
//! itself routes through `verify_umod` → `compile`.

use crate::{GraphCompileError, GraphRuntimeHandle, VerifiedGraph};

#[allow(dead_code)]
type WireId = u32;
#[allow(dead_code)]
type NodeId = u32;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[allow(dead_code)]
struct WireRuntime {
    wire_id: WireId,
    epoch: u64,
    producer_node: NodeId,
    consumer_count: u32,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[allow(dead_code)]
struct ConsumerObservation {
    wire_id: WireId,
    consumer_node: NodeId,
    last_observed_epoch: u64,
}

impl WireRuntime {
    #[allow(dead_code)]
    const fn new(wire_id: WireId, producer_node: NodeId, consumer_count: u32) -> Self {
        Self {
            wire_id,
            epoch: 0,
            producer_node,
            consumer_count,
        }
    }

    #[allow(dead_code)]
    fn publish(&mut self) {
        self.epoch = self.epoch.saturating_add(1);
    }

    #[allow(dead_code)]
    const fn ready_for(&self, observation: ConsumerObservation) -> bool {
        self.wire_id == observation.wire_id && self.epoch > observation.last_observed_epoch
    }

    #[allow(dead_code)]
    const fn epoch(&self) -> u64 {
        self.epoch
    }
}

impl ConsumerObservation {
    #[allow(dead_code)]
    const fn new(wire_id: WireId, consumer_node: NodeId) -> Self {
        Self {
            wire_id,
            consumer_node,
            last_observed_epoch: 0,
        }
    }

    #[allow(dead_code)]
    fn observe(&mut self, wire: WireRuntime) {
        if self.wire_id == wire.wire_id {
            self.last_observed_epoch = wire.epoch;
        }
    }
}

/// Compile a verified graph into runtime structures inside
/// `GraphArena`. This is where the only legal `GraphRuntime { … }`
/// construction in the workspace lives.
pub(crate) fn compile(
    _verified: VerifiedGraph<'_>,
) -> Result<GraphRuntimeHandle, GraphCompileError> {
    #![allow(clippy::unnecessary_wraps)]

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

#[cfg(test)]
mod tests {
    use super::{ConsumerObservation, WireRuntime};

    #[test]
    fn readiness_is_epoch_greater_than_last_observed() {
        let mut wire = WireRuntime::new(7, 1, 1);
        let mut consumer = ConsumerObservation::new(7, 2);

        assert!(!wire.ready_for(consumer));

        wire.publish();
        assert_eq!(wire.epoch(), 1);
        assert!(wire.ready_for(consumer));

        consumer.observe(wire);
        assert!(!wire.ready_for(consumer));

        wire.publish();
        assert_eq!(wire.epoch(), 2);
        assert!(wire.ready_for(consumer));
    }

    #[test]
    fn consumer_observation_is_wire_specific() {
        let mut wire = WireRuntime::new(8, 1, 1);
        let mut other_consumer = ConsumerObservation::new(9, 2);

        wire.publish();
        other_consumer.observe(wire);

        assert!(wire.ready_for(ConsumerObservation::new(8, 2)));
        assert!(!wire.ready_for(other_consumer));
    }
}
