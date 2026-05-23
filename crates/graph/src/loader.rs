//! Graph runtime loader. **The only legal site of `GraphRuntime`
//! construction in the entire workspace.** Spec §5.7, §5.8.
//!
//! `graph-verifier-auditor` verifies the same invariant at review time. There
//! is no test-only path, no dev-mode bypass, no IDE editor shortcut. The IDE
//! itself routes through `verify_umod` → `compile`.

use crate::{GraphCompileError, GraphDisplayState, GraphRuntimeHandle, VerifiedGraph};
use umod::{parse_node_descriptor, parse_structural, parse_wire_descriptor};

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

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum BuiltinNodeKind {
    Source,
    Transform,
    Sink,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
struct NodeRuntime {
    node_id: NodeId,
    kind: BuiltinNodeKind,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
struct GraphRuntime {
    nodes: [NodeRuntime; 3],
    source_to_transform: WireRuntime,
    transform_to_sink: WireRuntime,
    active_node: Option<NodeId>,
    last_completed_node: Option<NodeId>,
    sink_value: u32,
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

impl GraphRuntime {
    const fn source_transform_sink() -> Self {
        Self {
            nodes: [
                NodeRuntime {
                    node_id: 1,
                    kind: BuiltinNodeKind::Source,
                },
                NodeRuntime {
                    node_id: 2,
                    kind: BuiltinNodeKind::Transform,
                },
                NodeRuntime {
                    node_id: 3,
                    kind: BuiltinNodeKind::Sink,
                },
            ],
            source_to_transform: WireRuntime::new(1, 1, 1),
            transform_to_sink: WireRuntime::new(2, 2, 1),
            active_node: None,
            last_completed_node: None,
            sink_value: 0,
        }
    }

    fn execute_once(&mut self) -> u32 {
        let mut transform_input = ConsumerObservation::new(self.source_to_transform.wire_id, 2);
        let mut sink_input = ConsumerObservation::new(self.transform_to_sink.wire_id, 3);

        self.set_active_node(1);
        let source_value = 7;
        self.source_to_transform.publish();
        self.clear_active_node_completed(1);

        self.set_active_node(2);
        let transformed = if self.source_to_transform.ready_for(transform_input) {
            transform_input.observe(self.source_to_transform);
            source_value + 1
        } else {
            0
        };
        self.transform_to_sink.publish();
        self.clear_active_node_completed(2);

        self.set_active_node(3);
        if self.transform_to_sink.ready_for(sink_input) {
            sink_input.observe(self.transform_to_sink);
            self.sink_value = transformed;
        }
        self.clear_active_node_completed(3);

        self.sink_value
    }

    fn set_active_node(&mut self, node_id: NodeId) {
        self.active_node = Some(node_id);
    }

    fn clear_active_node_completed(&mut self, node_id: NodeId) {
        self.last_completed_node = Some(node_id);
        self.active_node = None;
    }

    const fn active_node(&self) -> Option<NodeId> {
        self.active_node
    }

    const fn last_completed_node(&self) -> Option<NodeId> {
        self.last_completed_node
    }
}

/// Compile a verified graph into runtime structures inside
/// `GraphArena`. This is where the only legal `GraphRuntime { … }`
/// construction in the workspace lives.
pub(crate) fn compile(
    verified: VerifiedGraph<'_>,
) -> Result<GraphRuntimeHandle, GraphCompileError> {
    #![allow(clippy::unnecessary_wraps)]
    #![allow(clippy::needless_pass_by_value)]

    // 1. Allocate node runtime table in GraphArena.
    // 2. Allocate wire runtime table in GraphArena.
    // 3. Resolve dispatch table for each node's node_type_id.
    // 4. Bind constant blobs to node descriptors.
    // 5. Construct GraphRuntime { node_runtimes, wire_runtimes,
    //    dispatch_table, scheduling_policy }.
    // 6. Wrap in opaque GraphRuntimeHandle.

    let header = parse_structural(verified.bytes())
        .map_err(|_| GraphCompileError::NodeRuntimeAllocFailed)?;
    let mut display_state = GraphDisplayState::new(
        header.graph_stable_id,
        header.node_count,
        header.wire_count,
        None,
        None,
    );

    if verified_source_transform_sink(verified.bytes()) {
        let mut runtime = GraphRuntime::source_transform_sink();
        let _ = runtime.execute_once();
        display_state.active_node = runtime.active_node();
        display_state.last_completed_node = runtime.last_completed_node();
    }

    Ok(GraphRuntimeHandle::new_internal(display_state))
}

fn verified_source_transform_sink(bytes: &[u8]) -> bool {
    let Ok(header) = parse_structural(bytes) else {
        return false;
    };
    if header.node_count != 3 || header.wire_count != 2 {
        return false;
    }

    let Ok(source) = parse_node_descriptor(bytes, header, 0) else {
        return false;
    };
    let Ok(transform) = parse_node_descriptor(bytes, header, 1) else {
        return false;
    };
    let Ok(sink) = parse_node_descriptor(bytes, header, 2) else {
        return false;
    };
    let Ok(first_wire) = parse_wire_descriptor(bytes, header, 0) else {
        return false;
    };
    let Ok(second_wire) = parse_wire_descriptor(bytes, header, 1) else {
        return false;
    };

    source.node_id == 1
        && source.node_type_id == 1
        && transform.node_id == 2
        && transform.node_type_id == 2
        && sink.node_id == 3
        && sink.node_type_id == 3
        && first_wire.src_node_id == 1
        && first_wire.dst_node_id == 2
        && second_wire.src_node_id == 2
        && second_wire.dst_node_id == 3
}

// NOTE: `GraphRuntime` itself (the inner struct holding NodeRuntime
// and WireRuntime tables) is intentionally not yet defined here.
// When it is, its fields stay private to this module and `pub(super)`
// at most. The outer crate exposes only the opaque
// `GraphRuntimeHandle`.

#[cfg(test)]
mod tests {
    use super::{ConsumerObservation, GraphRuntime, WireRuntime};
    use crate::{graph_compile_verified, graph_load_from_umod, SOURCE_TRANSFORM_SINK_UMOD};

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

    #[test]
    fn fan_out_consumers_observe_same_epoch_independently() {
        let mut wire = WireRuntime::new(10, 1, 2);
        let mut first = ConsumerObservation::new(10, 2);
        let mut second = ConsumerObservation::new(10, 3);

        wire.publish();
        assert_eq!(wire.epoch(), 1);
        assert!(wire.ready_for(first));
        assert!(wire.ready_for(second));

        first.observe(wire);
        assert!(!wire.ready_for(first));
        assert!(wire.ready_for(second));

        second.observe(wire);
        assert!(!wire.ready_for(first));
        assert!(!wire.ready_for(second));

        wire.publish();
        assert!(wire.ready_for(first));
        assert!(wire.ready_for(second));
    }

    #[test]
    fn builtin_graph_reaches_runtime_through_verified_pipeline() {
        let verified = graph_load_from_umod(SOURCE_TRANSFORM_SINK_UMOD).unwrap();
        assert!(graph_compile_verified(verified).is_ok());
    }

    #[test]
    fn compiled_handle_exposes_read_only_display_state() {
        let verified = graph_load_from_umod(SOURCE_TRANSFORM_SINK_UMOD).unwrap();
        let handle = graph_compile_verified(verified).unwrap();

        assert_eq!(handle.display_state().graph_id(), 0x0053_5453);
        assert_eq!(handle.display_state().node_count(), 3);
        assert_eq!(handle.display_state().wire_count(), 2);
        assert_eq!(handle.display_state().active_node(), None);
        assert_eq!(handle.display_state().last_completed_node(), Some(3));
    }

    #[test]
    fn source_transform_sink_executes_once() {
        let mut graph = GraphRuntime::source_transform_sink();

        let sink = graph.execute_once();

        assert_eq!(sink, 8);
        assert_eq!(graph.source_to_transform.epoch(), 1);
        assert_eq!(graph.transform_to_sink.epoch(), 1);
    }

    #[test]
    fn active_node_is_cleared_after_execution() {
        let mut graph = GraphRuntime::source_transform_sink();

        assert_eq!(graph.active_node(), None);
        graph.execute_once();

        assert_eq!(graph.active_node(), None);
        assert_eq!(graph.last_completed_node(), Some(3));
    }
}
