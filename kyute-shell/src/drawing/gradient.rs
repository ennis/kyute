use crate::drawing::{
    mk_color_f, mk_matrix_3x2, mk_point_f, Brush, Color, DrawContext, Point, Transform,
};
use palette::{Gradient, LinSrgba, Mix};
use std::ptr;
use winapi::{
    shared::winerror::SUCCEEDED,
    um::{d2d1::*, d2d1_1::*},
};
use wio::com::ComPtr;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum ColorInterpolationMode {
    /// Gamma-correct interpolation
    GammaCorrect,
    /// Interpolation in Gamma 2.2
    Gamma22,
}

impl ColorInterpolationMode {
    fn to_d2d(self) -> D2D1_GAMMA {
        match self {
            ColorInterpolationMode::GammaCorrect => D2D1_GAMMA_1_0,
            ColorInterpolationMode::Gamma22 => D2D1_GAMMA_2_2,
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum ExtendMode {
    Clamp,
    Wrap,
    Mirror,
}

impl ExtendMode {
    fn to_d2d(self) -> D2D1_EXTEND_MODE {
        match self {
            ExtendMode::Clamp => D2D1_EXTEND_MODE_CLAMP,
            ExtendMode::Wrap => D2D1_EXTEND_MODE_WRAP,
            ExtendMode::Mirror => D2D1_EXTEND_MODE_MIRROR,
        }
    }
}

#[derive(Clone)]
pub struct GradientStopCollection(pub(crate) ComPtr<ID2D1GradientStopCollection1>);

impl GradientStopCollection {
    /// Gamma-correct gradient
    pub fn new(
        ctx: &DrawContext,
        colors: &[(f64, Color)],
        color_interpolation: ColorInterpolationMode,
        extend_mode: ExtendMode,
    ) -> Self {
        let gradient_stops: Vec<_> = colors
            .iter()
            .map(|(p, c)| D2D1_GRADIENT_STOP {
                position: *p as f32,
                color: mk_color_f(*c),
            })
            .collect();
        /*
        straightAlphaGradientStops: *const D2D1_GRADIENT_STOP,
        straightAlphaGradientStopsCount: UINT32,
        preInterpolationSpace: D2D1_COLOR_SPACE,
        postInterpolationSpace: D2D1_COLOR_SPACE,
        bufferPrecision: D2D1_BUFFER_PRECISION,
        extendMode: D2D1_EXTEND_MODE,
        colorInterpolationMode: D2D1_COLOR_INTERPOLATION_MODE,
        gradientStopCollection1: *mut *mut ID2D1GradientStopCollection1,
        */
        unsafe {
            let mut collection = ptr::null_mut();
            let hr = ctx.ctx.CreateGradientStopCollection(
                gradient_stops.as_ptr(),
                gradient_stops.len() as u32,
                D2D1_COLOR_SPACE_SRGB,
                D2D1_COLOR_SPACE_SRGB,
                D2D1_BUFFER_PRECISION_32BPC_FLOAT,
                extend_mode.to_d2d(),
                D2D1_COLOR_INTERPOLATION_MODE_PREMULTIPLIED,
                &mut collection,
            );
            assert!(SUCCEEDED(hr));
            GradientStopCollection(ComPtr::from_raw(collection))
        }
    }
}
