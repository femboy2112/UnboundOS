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
pub struct Arena {
    id: ArenaId,
    base: usize,
    cursor: usize,
    limit: usize,
}

impl Arena {
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

fn align_up(value: usize, alignment: usize) -> Option<usize> {
    debug_assert!(alignment.is_power_of_two());
    let mask = alignment - 1;
    value.checked_add(mask).map(|v| v & !mask)
}

#[cfg(test)]
mod tests {
    use super::{AllocError, Arena, ArenaId};

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
    }

    #[test]
    fn reset_returns_cursor_to_base() {
        let mut arena = Arena::new(ArenaId::Scratch, 0x4000, 0x40).unwrap();
        assert_eq!(arena.alloc_aligned(0x20, 0x20), Ok(0x4000));

        arena.reset();

        assert_eq!(arena.cursor(), 0x4000);
        assert_eq!(arena.remaining(), 0x40);
    }
}
