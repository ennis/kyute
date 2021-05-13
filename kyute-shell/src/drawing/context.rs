//! Direct2D render target
use crate::{
    drawing::{
        brush::Brush, mk_color_f, mk_matrix_3x2, mk_point_f, mk_rect_f, Color, PathGeometry, Point,
        Rect, Transform,
    },
    text::TextLayout,
};
use bitflags::bitflags;
use std::mem::MaybeUninit;
use tracing::error;
use crate::bindings::Windows::Win32::Direct2D::{
    ID2D1Bitmap1, ID2D1DeviceContext, ID2D1DrawingStateBlock, ID2D1Factory,
    ID2D1Geometry, ID2D1Image, D2D1_ANTIALIAS_MODE, D2D1_COMPOSITE_MODE,
    D2D1_DRAWING_STATE_DESCRIPTION, D2D1_DRAW_TEXT_OPTIONS, D2D1_INTERPOLATION_MODE,
    D2D1_ROUNDED_RECT, D2D1_TEXT_ANTIALIAS_MODE,
};
use windows::Interface;

pub struct DrawingState(ID2D1DrawingStateBlock);

pub enum SaveState {
    DrawingState {
        transform: Transform,
        drawing_state: DrawingState,
    },
    AxisAlignedClip,
}

pub trait Geometry {
    fn as_raw_geometry(&self) -> ID2D1Geometry;
}

impl Geometry for PathGeometry {
    fn as_raw_geometry(&self) -> ID2D1Geometry {
        self.0.clone().into()
    }
}

/// Trait implemented by types that can be
pub trait Image {
    fn as_raw_image(&self) -> ID2D1Image;
}

pub struct Bitmap(pub(crate) ID2D1Bitmap1);

impl Image for Bitmap {
    fn as_raw_image(&self) -> ID2D1Image {
        self.0.cast().unwrap()
    }
}

bitflags! {
    #[derive(Default)]
    pub struct DrawTextOptions: u32 {
        const NO_SNAP = D2D1_DRAW_TEXT_OPTIONS::D2D1_DRAW_TEXT_OPTIONS_NO_SNAP.0;
        const CLIP = D2D1_DRAW_TEXT_OPTIONS::D2D1_DRAW_TEXT_OPTIONS_CLIP.0;
        const ENABLE_COLOR_FONT = D2D1_DRAW_TEXT_OPTIONS::D2D1_DRAW_TEXT_OPTIONS_ENABLE_COLOR_FONT.0;
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum PrimitiveBlend {
    SourceOver,
    Copy,
    Min,
    Add,
    Max,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum InterpolationMode {
    NearestNeighbor,
    Linear,
    Cubic,
    MultiSampleLinear,
    Anisotropic,
    HighQualityCubic,
}

impl InterpolationMode {
    fn to_d2d(self) -> D2D1_INTERPOLATION_MODE {
        match self {
            InterpolationMode::NearestNeighbor => {
                D2D1_INTERPOLATION_MODE::D2D1_INTERPOLATION_MODE_NEAREST_NEIGHBOR
            }
            InterpolationMode::Linear => D2D1_INTERPOLATION_MODE::D2D1_INTERPOLATION_MODE_LINEAR,
            InterpolationMode::Cubic => D2D1_INTERPOLATION_MODE::D2D1_INTERPOLATION_MODE_CUBIC,
            InterpolationMode::MultiSampleLinear => {
                D2D1_INTERPOLATION_MODE::D2D1_INTERPOLATION_MODE_MULTI_SAMPLE_LINEAR
            }
            InterpolationMode::Anisotropic => {
                D2D1_INTERPOLATION_MODE::D2D1_INTERPOLATION_MODE_ANISOTROPIC
            }
            InterpolationMode::HighQualityCubic => {
                D2D1_INTERPOLATION_MODE::D2D1_INTERPOLATION_MODE_HIGH_QUALITY_CUBIC
            }
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum CompositeMode {
    SourceOver,
    DestinationOver,
    SourceIn,
    DestinationIn,
    SourceOut,
    DestinationOut,
    SourceAtop,
    DestinationAtop,
    Xor,
    Plus,
    SourceCopy,
    BoundedSourceCopy,
    MaskInvert,
}

impl CompositeMode {
    fn to_d2d(self) -> D2D1_COMPOSITE_MODE {
        match self {
            CompositeMode::SourceOver => D2D1_COMPOSITE_MODE::D2D1_COMPOSITE_MODE_SOURCE_OVER,
            CompositeMode::DestinationOver => {
                D2D1_COMPOSITE_MODE::D2D1_COMPOSITE_MODE_DESTINATION_OVER
            }
            CompositeMode::SourceIn => D2D1_COMPOSITE_MODE::D2D1_COMPOSITE_MODE_SOURCE_IN,
            CompositeMode::DestinationIn => D2D1_COMPOSITE_MODE::D2D1_COMPOSITE_MODE_DESTINATION_IN,
            CompositeMode::SourceOut => D2D1_COMPOSITE_MODE::D2D1_COMPOSITE_MODE_SOURCE_OUT,
            CompositeMode::DestinationOut => {
                D2D1_COMPOSITE_MODE::D2D1_COMPOSITE_MODE_DESTINATION_OUT
            }
            CompositeMode::SourceAtop => D2D1_COMPOSITE_MODE::D2D1_COMPOSITE_MODE_SOURCE_ATOP,
            CompositeMode::DestinationAtop => {
                D2D1_COMPOSITE_MODE::D2D1_COMPOSITE_MODE_DESTINATION_ATOP
            }
            CompositeMode::Xor => D2D1_COMPOSITE_MODE::D2D1_COMPOSITE_MODE_XOR,
            CompositeMode::Plus => D2D1_COMPOSITE_MODE::D2D1_COMPOSITE_MODE_PLUS,
            CompositeMode::SourceCopy => D2D1_COMPOSITE_MODE::D2D1_COMPOSITE_MODE_SOURCE_COPY,
            CompositeMode::BoundedSourceCopy => {
                D2D1_COMPOSITE_MODE::D2D1_COMPOSITE_MODE_BOUNDED_SOURCE_COPY
            }
            CompositeMode::MaskInvert => D2D1_COMPOSITE_MODE::D2D1_COMPOSITE_MODE_MASK_INVERT,
        }
    }
}

pub struct DrawContext {
    pub(crate) ctx: ID2D1DeviceContext,
    pub(crate) factory: ID2D1Factory,
    save_states: Vec<SaveState>,
    transform: Transform,
}

impl Drop for DrawContext {
    fn drop(&mut self) {
        self.end_draw()
    }
}

impl DrawContext {
    /// Acquires (shared) ownership of the device context.
    /// A target must already be set on the DC with SetTarget.
    pub unsafe fn from_device_context(
        factory: ID2D1Factory,
        device_context: ID2D1DeviceContext,
    ) -> DrawContext {
        device_context.BeginDraw();
        DrawContext {
            factory,
            ctx: device_context,
            save_states: Vec::new(),
            transform: Transform::identity(),
        }
    }

    /*pub fn new(device: &mut Device, image: &mut dyn Image) -> DrawContext {
        device_context.ctx.SetTarget(target.as_raw_image());
        DrawContext {
            ctx: device_context,
            image
        }
    }*/

    pub(crate) fn end_draw(&mut self) {
        unsafe {
            let mut tag1 = MaybeUninit::<u64>::uninit();
            let mut tag2 = MaybeUninit::<u64>::uninit();
            let hr = self.ctx.EndDraw(tag1.as_mut_ptr(), tag2.as_mut_ptr());
            let tag1 = tag1.assume_init();
            let tag2 = tag2.assume_init();
            if hr.is_err() {
                error!(
                    "EndDraw error: {}, tags=({},{})",
                    windows::Error::from(hr),
                    tag1,
                    tag2
                );
            }
            if !self.save_states.is_empty() {
                error!("save stack not empty");
            }
        }
    }

    /// Safety: use a closure instead?
    pub fn push_axis_aligned_clip(&mut self, rect: Rect) {
        unsafe {
            self.ctx.PushAxisAlignedClip(
                &mk_rect_f(rect),
                D2D1_ANTIALIAS_MODE::D2D1_ANTIALIAS_MODE_ALIASED,
            );
        }
    }

    pub fn pop_axis_aligned_clip(&mut self) {
        unsafe {
            self.ctx.PopAxisAlignedClip();
        }
    }

    pub fn save(&mut self) {
        unsafe {
            let desc = D2D1_DRAWING_STATE_DESCRIPTION {
                antialiasMode: D2D1_ANTIALIAS_MODE::D2D1_ANTIALIAS_MODE_PER_PRIMITIVE,
                textAntialiasMode: D2D1_TEXT_ANTIALIAS_MODE::D2D1_TEXT_ANTIALIAS_MODE_DEFAULT,
                tag1: 0,
                tag2: 0,
                transform: mk_matrix_3x2(&Transform::identity()),
            };

            let mut dsb = None;
            let dsb = self
                .factory
                .CreateDrawingStateBlock(&desc, None, &mut dsb)
                .and_some(dsb)
                .unwrap();

            //trace!("SaveDrawingState");
            self.ctx.SaveDrawingState(&dsb);
            let transform = self.transform;
            self.save_states.push(SaveState::DrawingState {
                transform,
                drawing_state: DrawingState(dsb),
            });
        }
    }

    pub fn restore(&mut self) {
        while let Some(state) = self.save_states.pop() {
            match state {
                SaveState::DrawingState {
                    transform,
                    drawing_state,
                } => {
                    //trace!("RestoreDrawingState");
                    unsafe {
                        self.transform = transform;
                        self.ctx.RestoreDrawingState(&drawing_state.0);
                    }
                    break;
                }
                SaveState::AxisAlignedClip => unsafe {
                    self.ctx.PopAxisAlignedClip();
                },
            }
        }
    }

    pub fn transform(&mut self, transform: &Transform) {
        self.transform = self.transform.post_transform(transform);
        unsafe {
            self.ctx.SetTransform(&mk_matrix_3x2(&self.transform));
        }
    }
}

impl DrawContext {
    pub fn clear(&mut self, color: Color) {
        unsafe {
            self.ctx.Clear(&mk_color_f(color));
        }
    }

    pub fn draw_text_layout(
        &mut self,
        origin: Point,
        text_layout: &TextLayout,
        default_fill_brush: &Brush,
        text_options: DrawTextOptions,
    ) {
        unsafe {
            self.ctx.DrawTextLayout(
                mk_point_f(origin),
                text_layout.as_raw(),
                default_fill_brush.to_base_brush(),
                D2D1_DRAW_TEXT_OPTIONS::from(text_options.bits),
            );
        }
    }

    pub fn draw_rectangle(&mut self, rect: Rect, brush: &Brush, width: f64) {
        unsafe {
            self.ctx
                .DrawRectangle(&mk_rect_f(rect), brush.to_base_brush(), width as f32, None);
        }
    }

    pub fn draw_rounded_rectangle(
        &mut self,
        rect: Rect,
        radius_x: f64,
        radius_y: f64,
        brush: &Brush,
        width: f64,
    ) {
        unsafe {
            let rounded_rect = D2D1_ROUNDED_RECT {
                rect: mk_rect_f(rect),
                radiusX: radius_x as f32,
                radiusY: radius_y as f32,
            };

            self.ctx
                .DrawRoundedRectangle(&rounded_rect, brush.to_base_brush(), width as f32, None);
        }
    }

    pub fn fill_rectangle(&mut self, rect: Rect, brush: &Brush) {
        unsafe {
            self.ctx
                .FillRectangle(&mk_rect_f(rect), brush.to_base_brush());
        }
    }

    pub fn fill_rounded_rectangle(
        &mut self,
        rect: Rect,
        radius_x: f64,
        radius_y: f64,
        brush: &Brush,
    ) {
        unsafe {
            let rounded_rect = D2D1_ROUNDED_RECT {
                rect: mk_rect_f(rect),
                radiusX: radius_x as f32,
                radiusY: radius_y as f32,
            };
            self.ctx
                .FillRoundedRectangle(&rounded_rect, brush.to_base_brush());
        }
    }

    pub fn draw_image<I: Image>(
        &mut self,
        image: &I,
        at: Point,
        source_rect: Rect,
        interpolation_mode: InterpolationMode,
        composite_mode: CompositeMode,
    ) {
        unsafe {
            self.ctx.DrawImage(
                image.as_raw_image(),
                &mk_point_f(at),
                &mk_rect_f(source_rect),
                interpolation_mode.to_d2d(),
                composite_mode.to_d2d(),
            );
        }
    }

    pub fn fill_geometry<G: Geometry>(&mut self, geometry: &G, brush: &Brush) {
        unsafe {
            self.ctx
                .FillGeometry(geometry.as_raw_geometry(), brush.to_base_brush(), None);
        }
    }

    pub fn draw_geometry<G: Geometry>(&mut self, geometry: &G, brush: &Brush, width: f64) {
        unsafe {
            self.ctx.DrawGeometry(
                geometry.as_raw_geometry(),
                brush.to_base_brush(),
                width as f32,
                None,
            );
        }
    }

    /// Scale factor between DIPs and pixels (1 DIP = scale-factor pixels).
    pub fn scale_factor(&self) -> f64 {
        unsafe {
            let mut dpi_x = 0.0f32;
            let mut dpi_y = 0.0f32;
            self.ctx.GetDpi(&mut dpi_x, &mut dpi_y);
            // assume that both DPI values are the same (pixels are square).
            // TODO non-square pixels?
            dpi_x as f64 / 96.0
        }
    }
}
