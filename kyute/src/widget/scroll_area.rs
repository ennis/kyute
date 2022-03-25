use crate::{
    cache::scoped,
    event::WheelDeltaMode,
    style::BoxStyle,
    widget::{
        grid::GridTrackDefinition, prelude::*, Container, DragController, Grid, GridLength, LayoutInspector, Null,
        Viewport,
    },
    Color, Length, UnitExt,
};

#[derive(Clone)]
pub struct ScrollArea {
    inner: LayoutInspector<Grid>,
    line_height: Length,
    scroll: Signal<f64>,
}

const DEFAULT_LINE_HEIGHT: Length = Length::Dip(20.0);

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
        let mut grid_container = {
            let mut grid = Grid::new();
            grid.push_row_definition(GridTrackDefinition::new(GridLength::Flex(1.0)));
            grid.append_column_definitions([
                GridTrackDefinition::new(GridLength::Flex(1.0)),
                GridTrackDefinition::new(GridLength::Fixed(5.dip())),
            ]);
            LayoutInspector::new(grid)
        };

        // HACK: even if the content already fits in the grid container, we still have to wrap
        // the content widget in a Viewport, because otherwise the size returned by the LayoutInspector
        // will always be clamped to the size of the grid.
        let mut content_viewport = Viewport::new(LayoutInspector::new(contents)).constrain_width();

        // apply scroll to content pos
        if let Some(scroll) = scroll.value() {
            content_pos += scroll;
        }

        // TODO check that content size is finite
        // v_height = viewport height
        // c_height = content height
        // t = thumb height
        // t_p = thumb position
        let viewport_height = grid_container.size().height;
        let content_height = content_viewport.contents().size().height;

        if content_height <= viewport_height {
            content_viewport.set_transform(Offset::new(0.0, 0.0).to_transform());
            grid_container.contents_mut().add_item(0, .., content_viewport);
            return ScrollArea {
                inner: grid_container,
                line_height: DEFAULT_LINE_HEIGHT,
                scroll,
            };
        }

        let min_thumb_size = 30.0;
        let thumb_size = (viewport_height * viewport_height / content_height).max(min_thumb_size);
        let content_to_thumb = (viewport_height - thumb_size) / (content_height - viewport_height);
        let thumb_pos = content_pos * content_to_thumb;
        let content_max = content_height - viewport_height;

        trace!(
            "viewport_height={}, content_height={}, content_to_thumb={}, thumb_pos={}, content_max={}, thumb_size={}",
            viewport_height,
            content_height,
            content_to_thumb,
            thumb_pos,
            content_max,
            thumb_size
        );

        let scroll_thumb = DragController::new(
            Container::new(Null)
                .fix_size(Size::new(5.0, thumb_size))
                .box_style(BoxStyle::new().radius(2.dip()).fill(Color::from_hex("#FF7F31"))),
        )
        .on_started(|| tmp_pos = content_pos)
        .on_delta(|offset| {
            content_pos = tmp_pos + offset.y / content_to_thumb;
        });

        content_pos = content_pos.clamp(0.0, content_max);
        content_viewport.set_transform(Offset::new(0.0, -content_pos).to_transform());

        let scroll_bar = Viewport::new(scroll_thumb).transform(Offset::new(0.0, thumb_pos).to_transform());

        grid_container.contents_mut().add_item(0, .., content_viewport);
        grid_container.contents_mut().add_item(0, 1, scroll_bar);
        ScrollArea {
            inner: grid_container,
            scroll,
            line_height: DEFAULT_LINE_HEIGHT,
        }
    }

    pub fn line_height(mut self, line_height: Length) -> Self {
        self.line_height = line_height.into();
        self
    }
}

impl Widget for ScrollArea {
    fn widget_id(&self) -> Option<WidgetId> {
        self.inner.widget_id()
    }

    fn event(&self, ctx: &mut EventCtx, event: &mut Event, env: &Environment) {
        self.inner.event(ctx, event, env);

        if !ctx.handled {
            if let Event::Wheel(wheel) = event {
                match wheel.delta_mode {
                    WheelDeltaMode::Pixel => {
                        self.scroll.signal(-wheel.delta_y);
                    }
                    WheelDeltaMode::Line => {
                        let line_height_dips = self.line_height.to_dips(ctx.scale_factor, self.inner.size().height);
                        self.scroll.signal(-line_height_dips * wheel.delta_y);
                    }
                    WheelDeltaMode::Page => {
                        // TODO
                        warn!("WheelDeltaMode::Page unimplemented");
                    }
                }
            }
        }
    }

    fn layout(&self, ctx: &mut LayoutCtx, constraints: BoxConstraints, env: &Environment) -> Measurements {
        self.inner.layout(ctx, constraints, env)
    }

    fn paint(&self, ctx: &mut PaintCtx, env: &Environment) {
        self.inner.paint(ctx, env)
    }
}
