//! Direct2D render target
use crate::drawing::brush::Brush;
use crate::drawing::{
    mk_color_f, mk_matrix_3x2, mk_point_f, mk_rect_f, Color, Point, Rect, Transform,
};
use crate::error::Error;
use crate::text::TextLayout;
use bitflags::bitflags;
use log::error;
use std::mem::MaybeUninit;
use std::ptr;
use winapi::shared::winerror::SUCCEEDED;
use winapi::um::d2d1::*;
use wio::com::ComPtr;

pub struct DrawingState(ComPtr<ID2D1DrawingStateBlock>);

pub enum SaveState {
    DrawingState {
        transform: Transform,
        drawing_state: DrawingState,
    },
    AxisAlignedClip,
}

pub struct RenderTarget {
    pub(crate) target: ComPtr<ID2D1RenderTarget>,
    pub(crate) factory: ComPtr<ID2D1Factory>,
    save_states: Vec<SaveState>,
    transform: Transform,
}

bitflags! {
    pub struct DrawTextOptions: u32 {
        const NO_SNAP = D2D1_DRAW_TEXT_OPTIONS_NO_SNAP;
        const CLIP = D2D1_DRAW_TEXT_OPTIONS_CLIP;
        const ENABLE_COLOR_FONT = D2D1_DRAW_TEXT_OPTIONS_ENABLE_COLOR_FONT;
    }
}

impl RenderTarget {
    pub unsafe fn from_raw(
        factory: ComPtr<ID2D1Factory>,
        ptr: *mut ID2D1RenderTarget,
    ) -> RenderTarget {
        RenderTarget {
            factory,
            target: ComPtr::from_raw(ptr),
            save_states: Vec::new(),
            transform: Transform::identity(),
        }
    }

    pub(crate) fn begin_draw(&mut self) {
        unsafe {
            self.target.BeginDraw();
        }
    }

    pub(crate) fn end_draw(&mut self) {
        unsafe {
            let mut tag1 = MaybeUninit::<D2D1_TAG>::uninit();
            let mut tag2 = MaybeUninit::<D2D1_TAG>::uninit();
            let hr = self.target.EndDraw(tag1.as_mut_ptr(), tag2.as_mut_ptr());
            let tag1 = tag1.assume_init();
            let tag2 = tag2.assume_init();
            if !SUCCEEDED(hr) {
                error!(
                    "EndDraw error: {}, tags=({},{})",
                    Error::HResultError(hr),
                    tag1,
                    tag2
                );
            }
            if !self.save_states.is_empty() {
                error!("save stack not empty");
            }
        }
    }

    pub fn clear(&mut self, color: Color) {
        unsafe {
            self.target.Clear(&mk_color_f(color));
        }
    }

    pub fn draw_text_layout(
        &mut self,
        origin: Point,
        text_layout: &TextLayout,
        default_fill_brush: &dyn Brush,
        text_options: DrawTextOptions,
    ) {
        unsafe {
            self.target.DrawTextLayout(
                mk_point_f(origin),
                text_layout.as_raw(),
                default_fill_brush.as_raw_brush(),
                text_options.bits,
            );
        }
    }

    pub fn transform(&mut self, transform: &Transform) {
        self.transform = self.transform.post_transform(transform);
        unsafe {
            self.target.SetTransform(&mk_matrix_3x2(&self.transform));
        }
    }

    pub fn draw_rectangle(&mut self, rect: Rect, brush: &dyn Brush, width: f64) {
        unsafe {
            self.target.DrawRectangle(
                &mk_rect_f(rect),
                brush.as_raw_brush(),
                width as f32,
                ptr::null_mut(),
            );
        }
    }

    pub fn fill_rectangle(&mut self, rect: Rect, brush: &dyn Brush) {
        unsafe {
            self.target
                .FillRectangle(&mk_rect_f(rect), brush.as_raw_brush());
        }
    }

    /// Safety: use a closure instead?
    pub fn push_axis_aligned_clip(&mut self, rect: Rect) {
        unsafe {
            self.target
                .PushAxisAlignedClip(&mk_rect_f(rect), D2D1_ANTIALIAS_MODE_ALIASED);
        }
    }

    pub fn pop_axis_aligned_clip(&mut self) {
        unsafe {
            self.target.PopAxisAlignedClip();
        }
    }

    pub fn save(&mut self) {
        unsafe {
            let mut ptr = ptr::null_mut();
            let desc = D2D1_DRAWING_STATE_DESCRIPTION {
                antialiasMode: D2D1_ANTIALIAS_MODE_PER_PRIMITIVE,
                textAntialiasMode: D2D1_TEXT_ANTIALIAS_MODE_DEFAULT,
                tag1: 0,
                tag2: 0,
                transform: mk_matrix_3x2(&Transform::identity()),
            };
            let hr = self
                .factory
                .CreateDrawingStateBlock(&desc, ptr::null_mut(), &mut ptr);
            assert!(SUCCEEDED(hr));
            //trace!("SaveDrawingState");
            self.target.SaveDrawingState(ptr);
            let transform = self.transform;
            self.save_states.push(SaveState::DrawingState {
                transform,
                drawing_state: DrawingState(ComPtr::from_raw(ptr)),
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
                        self.target.RestoreDrawingState(drawing_state.0.as_raw());
                    }
                    break;
                }
                SaveState::AxisAlignedClip => unsafe {
                    self.target.PopAxisAlignedClip();
                },
            }
        }
    }
}
