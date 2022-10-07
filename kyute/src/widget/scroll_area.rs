use crate::{
    drawing::ToSkia,
    event::WheelDeltaMode,
    widget::{grid::GridLayoutExt, prelude::*, DragController, Grid, LayoutInspector, Null, Viewport},
};

pub struct ScrollArea {
    inner: LayoutInspector<Grid>,
    line_height_dip: f64,
    scroll: Signal<f64>,
}

const DEFAULT_LINE_HEIGHT_DIP: f64 = 20.0;

impl ScrollArea {
    #[composable]
    pub fn new(contents: impl Widget + 'static) -> ScrollArea {
        #[state]
        let mut tmp_pos = 0.0;
        #[state]
        let mut content_pos: f64 = 0.0;

        // wheel scroll
        let scroll = Signal::new();

        // container grid: one row
        let mut grid_container = LayoutInspector::new(Grid::with_template("1fr / 1fr 5px"));

        // HACK: even if the content already fits in the grid container, we still have to wrap
        // the content widget in a Viewport, because otherwise the size returned by the LayoutInspector
        // will always be clamped to the size of the grid.
        let mut content_viewport = Viewport::new(LayoutInspector::new(contents)).constrain_width();

        // apply scroll to content pos
        if let Some(scroll) = scroll.value() {
            content_pos += scroll;
        }

        assert!(
            content_viewport.content().size().is_finite(),
            "the content widget of a ScrollArea should have finite dimensions"
        );

        // v_height = viewport height
        // c_height = content height
        // t = thumb height
        // t_p = thumb position
        let viewport_height = grid_container.size().height;
        let content_height = content_viewport.content().size().height;

        if content_height <= viewport_height {
            content_viewport.set_transform(Offset::new(0.0, 0.0).to_transform());
            grid_container.inner_mut().insert(content_viewport.grid_area((0, ..)));
            return ScrollArea {
                inner: grid_container,
                line_height_dip: DEFAULT_LINE_HEIGHT_DIP,
                scroll,
            };
        }

        let min_thumb_size = 30.0;
        let thumb_size = (viewport_height * viewport_height / content_height).max(min_thumb_size);
        let content_to_thumb = (viewport_height - thumb_size) / (content_height - viewport_height);
        let thumb_pos = content_pos * content_to_thumb;
        let content_max = content_height - viewport_height;

        trace!("viewport_height={viewport_height}, content_height={content_height}, content_to_thumb={content_to_thumb}, thumb_pos={thumb_pos}, content_max={content_max}, thumb_size={thumb_size}");

        //.box_style(Style::new().radius(2.dip()).background(Color::from_hex("#FF7F31"))),
        let scroll_thumb = DragController::new(content_pos, Null.fix_width(5.0).fix_height(thumb_size))
            .on_delta(|start_pos, offset| {
                content_pos = start_pos + offset.y / content_to_thumb;
            })
            .style("border-radius: 2px; background: #FF7F31;");

        content_pos = content_pos.clamp(0.0, content_max);
        content_viewport.set_transform(Offset::new(0.0, -content_pos).to_transform());

        let scroll_bar = Viewport::new(scroll_thumb).transform(Offset::new(0.0, thumb_pos).to_transform());

        grid_container.inner_mut().insert(content_viewport.grid_area((0, ..)));
        grid_container.inner_mut().insert(scroll_bar.grid_area((0, 1)));
        ScrollArea {
            inner: grid_container,
            scroll,
            line_height_dip: DEFAULT_LINE_HEIGHT_DIP,
        }
    }

    /*pub fn line_height(mut self, line_height: Length) -> Self {
        self.line_height = line_height.into();
        self
    }*/
}

impl Widget for ScrollArea {
    fn widget_id(&self) -> Option<WidgetId> {
        Widget::widget_id(&self.inner)
    }

    fn layout(&self, ctx: &mut LayoutCtx, constraints: &LayoutParams, env: &Environment) -> Geometry {
        Widget::layout(&self.inner, ctx, constraints, env)
    }

    fn event(&self, ctx: &mut EventCtx, event: &mut Event, env: &Environment) {
        Widget::route_event(&self.inner, ctx, event, env);

        if !ctx.handled {
            if let Event::Wheel(wheel) = event {
                match wheel.delta_mode {
                    WheelDeltaMode::Pixel => {
                        self.scroll.signal(-wheel.delta_y);
                    }
                    WheelDeltaMode::Line => {
                        self.scroll.signal(-self.line_height_dip * wheel.delta_y);
                    }
                    WheelDeltaMode::Page => {
                        // TODO
                        warn!("WheelDeltaMode::Page unimplemented");
                    }
                }
            }
        }
    }

    fn paint(&self, ctx: &mut PaintCtx) {
        let bounds = ctx.bounds;
        ctx.surface.canvas().save();
        ctx.surface
            .canvas()
            .clip_rect(bounds.to_skia(), skia_safe::ClipOp::Intersect, false);
        self.inner.paint(ctx);
        ctx.surface.canvas().restore();
    }
}
