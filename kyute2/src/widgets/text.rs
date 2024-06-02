use kurbo::Size;
use skia_safe as sk;
use tracing::warn;
use tracy_client::span;

use crate::{
    core::{WeakWidget, WeakWidgetPtr},
    drawing::ToSkia,
    text::{get_font_collection, TextSpan, TextStyle},
    Binding, BoxConstraints, Ctx, Event, Geometry, HitTestResult, LayoutCtx, PaintCtx, Point, Widget, WidgetCtx,
    WidgetPod, WidgetPtr,
};

pub struct Text {
    weak: WeakWidgetPtr<Self>,
    text: TextSpan,
    relayout: bool,
    paragraph: sk::textlayout::Paragraph,
}

impl Text {
    pub fn new(text: TextSpan) -> WidgetPtr<Text> {
        let paragraph = text.build_paragraph();
        WidgetPod::new_cyclic(move |weak| Text {
            weak,
            text,
            relayout: true,
            paragraph,
        })
    }
}

impl WeakWidget for Text {
    fn weak_self(&self) -> WeakWidgetPtr<Self> {
        self.weak.clone()
    }
}

impl Widget for Text {
    fn mount(&mut self, _cx: &mut Ctx) {}

    fn update(&mut self, cx: &mut Ctx) {
        /*if self.text.update(cx) {
            self.relayout = true;
            self.paragraph = build_paragraph(&self.text.value());
            cx.mark_needs_layout();
        }*/
    }

    fn event(&mut self, _ctx: &mut Ctx, _event: &mut Event) {}

    fn hit_test(&mut self, _ctx: &mut HitTestResult, position: Point) -> bool {
        if self.relayout {
            warn!("hit_test called before layout");
        }
        let paragraph = &self.paragraph;
        let paragraph_size = Size {
            width: paragraph.longest_line() as f64,
            height: paragraph.height() as f64,
        };
        paragraph_size.to_rect().contains(position)
    }

    fn layout(&mut self, _ctx: &mut LayoutCtx, params: &BoxConstraints) -> Geometry {
        // layout paragraph in available space
        let _span = span!("text layout");

        // available space for layout
        let available_width = params.max.width;
        let _available_height = params.max.height;

        // We can reuse the previous layout if and only if:
        // - the new available width is >= the current paragraph width (otherwise new line breaks are necessary)
        // - the current layout is still valid (i.e. it hasn't been previously invalidated)

        let paragraph = &mut self.paragraph;

        if !self.relayout && paragraph.longest_line() <= available_width as f32 {
            let paragraph_size = Size {
                width: paragraph.longest_line() as f64,
                height: paragraph.height() as f64,
            };
            let size = params.constrain(paragraph_size);
            return Geometry {
                size,
                baseline: Some(paragraph.alphabetic_baseline() as f64),
                bounding_rect: paragraph_size.to_rect(),
                paint_bounding_rect: paragraph_size.to_rect(),
            };
        }

        paragraph.layout(available_width as sk::scalar);
        let w = paragraph.longest_line() as f64;
        let h = paragraph.height() as f64;
        let alphabetic_baseline = paragraph.alphabetic_baseline();
        let unconstrained_size = Size::new(w, h);
        let size = params.constrain(unconstrained_size);
        self.relayout = false;

        /*self.paragraph.max_width();
        self.paragraph.max_intrinsic_width();
        self.paragraph.height();
        self.paragraph.alphabetic_baseline();*/

        // update cached values
        //self.available_width = available_width;
        //self.available_height = available_height;
        //self.scale_factor = params.scale_factor;

        Geometry {
            size,
            baseline: Some(alphabetic_baseline as f64),
            bounding_rect: size.to_rect(),
            paint_bounding_rect: size.to_rect(),
        }
    }

    fn paint(&mut self, ctx: &mut PaintCtx) {
        ctx.with_canvas(|canvas| {
            self.paragraph.paint(canvas, Point::ZERO.to_skia());
        })
    }
}
