//! GPU backend stuff.
//!
//! There's only vulkan now, but we should support skia+metal at some point.

#[cfg(feature = "vulkan")]
mod vulkan;

#[cfg(feature = "vulkan")]
pub(crate) use vulkan::*;

#[cfg(feature = "d3d")]
mod d3d;
#[cfg(feature = "d3d")]
pub(crate) use d3d::*;
