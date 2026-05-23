//! Tensor kernel implementations selected through dispatch.
//!
//! M10 starts with safe scalar kernels. Future SIMD backends must stay under
//! this module tree and remain reachable only through `dispatch.rs`.

pub mod scalar;
