use crate::{core::Widget, style::{State, StyleSet}, BoxConstraints, CompositionCtx, LayoutCtx, Measurements, PaintCtx, Rect, WidgetDelegate, Environment};

struct Container {
    background: StyleSet,
}

impl WidgetDelegate for Container {
    fn layout(
        &mut self,
        ctx: &mut LayoutCtx,
        children: &mut [Widget],
        constraints: &BoxConstraints,
        _env: &Environment
    ) -> Measurements {
        // expects only one children
        let mut measurements = Measurements::default();
        let constraints = constraints.deflate(&self.background.content_padding());
        for c in children {
            measurements = c.layout(ctx, &constraints);
        }
        measurements
    }

    fn paint(&mut self, ctx: &mut PaintCtx, children: &mut [Widget], bounds: Rect,
             _env: &Environment) {
        self.background.draw_box(ctx, &bounds, State::empty());
        for c in children {
            c.paint(ctx);
        }
    }
}

pub fn container<F>(cx: &mut CompositionCtx, background: StyleSet, contents: F)
where
    F: FnMut(&mut CompositionCtx),
{
    cx.enter(0);
    let _result = cx.emit_node(
        |_cx| Container {
            background: background.clone(),
        },
        |_cx, container| container.background = background.clone(),
        contents,
    );
    cx.exit();
}
