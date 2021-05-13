use crate::drawing::{
    mk_color_f, Color, DrawContext};
use crate::bindings::Windows::Win32::Direct2D::{D2D1_GAMMA, D2D1_EXTEND_MODE, ID2D1GradientStopCollection1, D2D1_GRADIENT_STOP, D2D1_COLOR_SPACE, D2D1_BUFFER_PRECISION, D2D1_COLOR_INTERPOLATION_MODE};

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
            ColorInterpolationMode::GammaCorrect => D2D1_GAMMA::D2D1_GAMMA_1_0,
            ColorInterpolationMode::Gamma22 => D2D1_GAMMA::D2D1_GAMMA_2_2,
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
            ExtendMode::Clamp => D2D1_EXTEND_MODE::D2D1_EXTEND_MODE_CLAMP,
            ExtendMode::Wrap => D2D1_EXTEND_MODE::D2D1_EXTEND_MODE_WRAP,
            ExtendMode::Mirror => D2D1_EXTEND_MODE::D2D1_EXTEND_MODE_MIRROR,
        }
    }
}

#[derive(Clone)]
pub struct GradientStopCollection(pub(crate) ID2D1GradientStopCollection1);

impl GradientStopCollection {
    /// Gamma-correct gradient
    pub fn new(
        ctx: &DrawContext,
        colors: &[(f64, Color)],
        _color_interpolation: ColorInterpolationMode,
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
            let mut collection = None;
            let collection = ctx.ctx.CreateGradientStopCollection2(
                gradient_stops.as_ptr(),
                gradient_stops.len() as u32,
                D2D1_COLOR_SPACE::D2D1_COLOR_SPACE_SRGB,
                D2D1_COLOR_SPACE::D2D1_COLOR_SPACE_SRGB,
                D2D1_BUFFER_PRECISION::D2D1_BUFFER_PRECISION_32BPC_FLOAT,
                extend_mode.to_d2d(),
                D2D1_COLOR_INTERPOLATION_MODE::D2D1_COLOR_INTERPOLATION_MODE_PREMULTIPLIED,
                &mut collection,
            ).and_some(collection).unwrap();
            GradientStopCollection(collection)
        }
    }
}
