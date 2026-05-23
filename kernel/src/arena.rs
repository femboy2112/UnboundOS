//! Bounded named arenas. Spec section 4.

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[allow(dead_code)]
pub enum ArenaId {
    Boot,
    Kernel,
    Graph,
    Scratch,
    ModelWeight,
    Inference,
    KvCache,
    Tokenizer,
    Sampler,
    ScratchTensor,
}

impl ArenaId {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Boot => "BootArena",
            Self::Kernel => "KernelArena",
            Self::Graph => "GraphArena",
            Self::Scratch => "ScratchArena",
            Self::ModelWeight => "ModelWeightArena",
            Self::Inference => "InferenceArena",
            Self::KvCache => "KVCacheArena",
            Self::Tokenizer => "TokenizerArena",
            Self::Sampler => "SamplerArena",
            Self::ScratchTensor => "ScratchTensorArena",
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[allow(dead_code)]
pub enum ArenaPhase {
    BootOnly,
    WholeBootSession,
    VerifiedGraphCompilation,
    ScratchPhase,
    LoadedModel,
    ActiveInference,
    ActiveChat,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct ArenaDescriptor {
    pub id: ArenaId,
    pub name: &'static str,
    pub phase: ArenaPhase,
}

pub const BOOT_ARENA: ArenaDescriptor = ArenaDescriptor {
    id: ArenaId::Boot,
    name: "BootArena",
    phase: ArenaPhase::BootOnly,
};

pub const KERNEL_ARENA: ArenaDescriptor = ArenaDescriptor {
    id: ArenaId::Kernel,
    name: "KernelArena",
    phase: ArenaPhase::WholeBootSession,
};

pub const GRAPH_ARENA: ArenaDescriptor = ArenaDescriptor {
    id: ArenaId::Graph,
    name: "GraphArena",
    phase: ArenaPhase::VerifiedGraphCompilation,
};

pub const SCRATCH_ARENA: ArenaDescriptor = ArenaDescriptor {
    id: ArenaId::Scratch,
    name: "ScratchArena",
    phase: ArenaPhase::ScratchPhase,
};

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[allow(dead_code)]
pub enum AllocError {
    InvalidAlignment {
        arena: ArenaId,
        requested: usize,
        alignment: usize,
    },
    Overflow {
        arena: ArenaId,
        requested: usize,
        alignment: usize,
        cursor: usize,
    },
    OutOfArenaMemory {
        arena: ArenaId,
        graph_id: u64,
        node_id: u32,
        model_id: u64,
        requested: usize,
        alignment: usize,
        base: usize,
        cursor: usize,
        limit: usize,
    },
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct ArenaFaultContext {
    pub arena: ArenaId,
    pub requested: usize,
    pub alignment: usize,
    pub base: usize,
    pub cursor: usize,
    pub limit: usize,
}

impl AllocError {
    pub const fn arena_fault_context(self) -> Option<ArenaFaultContext> {
        match self {
            Self::OutOfArenaMemory {
                arena,
                requested,
                alignment,
                base,
                cursor,
                limit,
                ..
            } => Some(ArenaFaultContext {
                arena,
                requested,
                alignment,
                base,
                cursor,
                limit,
            }),
            Self::InvalidAlignment { .. } | Self::Overflow { .. } => None,
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Arena {
    id: ArenaId,
    base: usize,
    cursor: usize,
    limit: usize,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct ArenaRange {
    pub base: usize,
    pub size: usize,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct M2ArenaRegions {
    pub boot: ArenaRange,
    pub kernel: ArenaRange,
    pub graph: ArenaRange,
    pub scratch: ArenaRange,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct M2ArenaSet {
    boot: Arena,
    kernel: Arena,
    graph: Arena,
    scratch: Arena,
}

impl Arena {
    pub const fn from_descriptor(
        descriptor: ArenaDescriptor,
        range: ArenaRange,
    ) -> Result<Self, AllocError> {
        Self::new(descriptor.id, range.base, range.size)
    }

    pub const fn new(id: ArenaId, base: usize, size: usize) -> Result<Self, AllocError> {
        let Some(limit) = base.checked_add(size) else {
            return Err(AllocError::Overflow {
                arena: id,
                requested: size,
                alignment: 1,
                cursor: base,
            });
        };
        Ok(Self {
            id,
            base,
            cursor: base,
            limit,
        })
    }

    pub const fn id(&self) -> ArenaId {
        self.id
    }

    pub const fn base(&self) -> usize {
        self.base
    }

    pub const fn cursor(&self) -> usize {
        self.cursor
    }

    pub const fn limit(&self) -> usize {
        self.limit
    }

    pub const fn remaining(&self) -> usize {
        self.limit - self.cursor
    }

    pub fn reset(&mut self) {
        self.cursor = self.base;
    }

    pub fn alloc_aligned(&mut self, size: usize, alignment: usize) -> Result<usize, AllocError> {
        if !alignment.is_power_of_two() {
            return Err(AllocError::InvalidAlignment {
                arena: self.id,
                requested: size,
                alignment,
            });
        }

        let aligned = align_up(self.cursor, alignment).ok_or(AllocError::Overflow {
            arena: self.id,
            requested: size,
            alignment,
            cursor: self.cursor,
        })?;
        let end = aligned.checked_add(size).ok_or(AllocError::Overflow {
            arena: self.id,
            requested: size,
            alignment,
            cursor: self.cursor,
        })?;

        if end > self.limit {
            return Err(AllocError::OutOfArenaMemory {
                arena: self.id,
                graph_id: 0,
                node_id: 0,
                model_id: 0,
                requested: size,
                alignment,
                base: self.base,
                cursor: self.cursor,
                limit: self.limit,
            });
        }

        self.cursor = end;
        Ok(aligned)
    }
}

impl M2ArenaSet {
    pub fn new(regions: M2ArenaRegions) -> Result<Self, AllocError> {
        Ok(Self {
            boot: Arena::from_descriptor(BOOT_ARENA, regions.boot)?,
            kernel: Arena::from_descriptor(KERNEL_ARENA, regions.kernel)?,
            graph: Arena::from_descriptor(GRAPH_ARENA, regions.graph)?,
            scratch: Arena::from_descriptor(SCRATCH_ARENA, regions.scratch)?,
        })
    }

    pub const fn boot(&self) -> &Arena {
        &self.boot
    }

    pub const fn kernel(&self) -> &Arena {
        &self.kernel
    }

    pub const fn graph(&self) -> &Arena {
        &self.graph
    }

    pub const fn scratch(&self) -> &Arena {
        &self.scratch
    }

    /// `BootArena` may allocate only before permanent kernel init completes.
    pub fn with_boot_arena<R>(&mut self, f: impl FnOnce(&mut Arena) -> R) -> R {
        f(&mut self.boot)
    }

    /// `KernelArena` owns permanent kernel structures for the whole boot session.
    pub fn with_kernel_arena<R>(&mut self, f: impl FnOnce(&mut Arena) -> R) -> R {
        f(&mut self.kernel)
    }

    /// `GraphArena` allocation is reserved for verified graph compilation.
    pub fn with_graph_arena<R>(&mut self, f: impl FnOnce(&mut Arena) -> R) -> R {
        f(&mut self.graph)
    }

    /// `ScratchArena` may allocate only during a declared scratch phase.
    pub fn with_scratch_arena<R>(&mut self, f: impl FnOnce(&mut Arena) -> R) -> R {
        f(&mut self.scratch)
    }
}

fn align_up(value: usize, alignment: usize) -> Option<usize> {
    debug_assert!(alignment.is_power_of_two());
    let mask = alignment - 1;
    value.checked_add(mask).map(|v| v & !mask)
}

#[cfg(test)]
mod tests {
    use super::{
        AllocError, Arena, ArenaFaultContext, ArenaId, ArenaPhase, ArenaRange, M2ArenaRegions,
        M2ArenaSet, BOOT_ARENA, GRAPH_ARENA, KERNEL_ARENA, SCRATCH_ARENA,
    };

    #[test]
    fn aligned_alloc_advances_cursor() {
        let mut arena = Arena::new(ArenaId::Boot, 0x1003, 0x100).unwrap();

        let ptr = arena.alloc_aligned(16, 16).unwrap();

        assert_eq!(arena.id(), ArenaId::Boot);
        assert_eq!(arena.base(), 0x1003);
        assert_eq!(arena.limit(), 0x1103);
        assert_eq!(ptr, 0x1010);
        assert_eq!(arena.cursor(), 0x1020);
        assert_eq!(arena.remaining(), 0xE3);
    }

    #[test]
    fn rejects_non_power_of_two_alignment() {
        let mut arena = Arena::new(ArenaId::Kernel, 0x2000, 0x100).unwrap();

        let err = arena.alloc_aligned(8, 24).unwrap_err();

        assert_eq!(
            err,
            AllocError::InvalidAlignment {
                arena: ArenaId::Kernel,
                requested: 8,
                alignment: 24,
            }
        );
        assert_eq!(arena.cursor(), 0x2000);
    }

    #[test]
    fn allocation_overflow_is_deterministic() {
        let mut arena = Arena::new(ArenaId::Scratch, usize::MAX - 7, 7).unwrap();

        let err = arena.alloc_aligned(1, 16).unwrap_err();

        assert_eq!(
            err,
            AllocError::Overflow {
                arena: ArenaId::Scratch,
                requested: 1,
                alignment: 16,
                cursor: usize::MAX - 7,
            }
        );
    }

    #[test]
    fn exhaustion_reports_arena_context() {
        let mut arena = Arena::new(ArenaId::Graph, 0x3000, 0x20).unwrap();
        assert_eq!(arena.alloc_aligned(0x10, 0x10), Ok(0x3000));
        assert_eq!(arena.alloc_aligned(0x10, 0x10), Ok(0x3010));

        let err = arena.alloc_aligned(1, 1).unwrap_err();

        assert_eq!(
            err,
            AllocError::OutOfArenaMemory {
                arena: ArenaId::Graph,
                graph_id: 0,
                node_id: 0,
                model_id: 0,
                requested: 1,
                alignment: 1,
                base: 0x3000,
                cursor: 0x3020,
                limit: 0x3020,
            }
        );
        assert_eq!(
            err.arena_fault_context(),
            Some(ArenaFaultContext {
                arena: ArenaId::Graph,
                requested: 1,
                alignment: 1,
                base: 0x3000,
                cursor: 0x3020,
                limit: 0x3020,
            })
        );
    }

    #[test]
    fn reset_returns_cursor_to_base() {
        let mut arena = Arena::new(ArenaId::Scratch, 0x4000, 0x40).unwrap();
        assert_eq!(arena.alloc_aligned(0x20, 0x20), Ok(0x4000));

        arena.reset();

        assert_eq!(arena.cursor(), 0x4000);
        assert_eq!(arena.remaining(), 0x40);
    }

    #[test]
    fn m2_descriptors_name_required_arenas_and_phases() {
        assert_eq!(BOOT_ARENA.name, "BootArena");
        assert_eq!(ArenaId::Boot.as_str(), "BootArena");
        assert_eq!(BOOT_ARENA.phase, ArenaPhase::BootOnly);
        assert_eq!(KERNEL_ARENA.name, "KernelArena");
        assert_eq!(ArenaId::Kernel.as_str(), "KernelArena");
        assert_eq!(KERNEL_ARENA.phase, ArenaPhase::WholeBootSession);
        assert_eq!(GRAPH_ARENA.name, "GraphArena");
        assert_eq!(ArenaId::Graph.as_str(), "GraphArena");
        assert_eq!(GRAPH_ARENA.phase, ArenaPhase::VerifiedGraphCompilation);
        assert_eq!(SCRATCH_ARENA.name, "ScratchArena");
        assert_eq!(ArenaId::Scratch.as_str(), "ScratchArena");
        assert_eq!(SCRATCH_ARENA.phase, ArenaPhase::ScratchPhase);
    }

    #[test]
    fn m2_arena_set_uses_named_guard_methods() {
        let mut arenas = M2ArenaSet::new(M2ArenaRegions {
            boot: ArenaRange {
                base: 0x1000,
                size: 0x100,
            },
            kernel: ArenaRange {
                base: 0x2000,
                size: 0x100,
            },
            graph: ArenaRange {
                base: 0x3000,
                size: 0x100,
            },
            scratch: ArenaRange {
                base: 0x4000,
                size: 0x100,
            },
        })
        .unwrap();

        assert_eq!(arenas.boot().id(), ArenaId::Boot);
        assert_eq!(arenas.kernel().id(), ArenaId::Kernel);
        assert_eq!(arenas.graph().id(), ArenaId::Graph);
        assert_eq!(arenas.scratch().id(), ArenaId::Scratch);
        assert_eq!(
            arenas.with_boot_arena(|a| a.alloc_aligned(8, 8)),
            Ok(0x1000)
        );
        assert_eq!(
            arenas.with_kernel_arena(|a| a.alloc_aligned(8, 8)),
            Ok(0x2000)
        );
        assert_eq!(
            arenas.with_graph_arena(|a| a.alloc_aligned(8, 8)),
            Ok(0x3000)
        );
        assert_eq!(
            arenas.with_scratch_arena(|a| a.alloc_aligned(8, 8)),
            Ok(0x4000)
        );
    }
}
