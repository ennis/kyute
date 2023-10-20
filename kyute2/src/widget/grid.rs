//! Grid layout.
//!

use crate::Length;

/// Length of a grid track.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum TrackBreadth {
    /// Size to content.
    Auto,
    /// Fixed size.
    Fixed(Length),
    /// Proportion of remaining space.
    Flex(f64),
}

pub struct Grid {}

pub struct GridElement {}
