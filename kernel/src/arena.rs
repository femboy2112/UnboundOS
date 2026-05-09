//! Bounded named arenas. Spec section 4.

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
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
pub enum AllocError {
    InvalidAlignment,
    Overflow,
    OutOfArenaMemory {
        arena: ArenaId,
        graph_id: u64,
        node_id: u32,
        model_id: u64,
        requested: usize,
        cursor: usize,
        limit: usize,
    },
}
