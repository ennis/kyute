use crate::{
    style::BoxStyle,
    theme,
    widget::{
        grid::GridTrackDefinition, prelude::*, Canvas, Container, DragController, Grid, GridLength, LayoutInspector,
        LayoutWrapper, Null, Thumb, Viewport,
    },
    Color, PointerEventKind, UnitExt,
};

#[derive(Clone)]
pub struct ScrollArea {
    inner: LayoutInspector<Grid>,
}

impl ScrollArea {
    #[composable]
    pub fn new(contents: impl Widget + 'static) -> ScrollArea {
        #[state]
        let mut tmp_pos = 0.0;
        #[state]
        let mut content_pos: f64 = 0.0;

        // container grid: one row
        let mut grid_container = LayoutInspector::new(Grid::with_rows_columns(
            [GridTrackDefinition::new(GridLength::Flex(1.0))],
            [
                GridTrackDefinition::new(GridLength::Flex(1.0)),
                GridTrackDefinition::new(GridLength::Fixed(5.0)),
            ],
        ));

        let content = LayoutInspector::new(contents);

        // TODO check that content size is finite
        // v_height = viewport height
        // c_height = content height
        // t = thumb height
        // t_p = thumb position
        let viewport_height = grid_container.size().height;
        let content_height = content.size().height;
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

        // place thumb in canvas

        // issue: the local position of the dragcontroller changes as we drag
        let scroll_thumb = DragController::new(
            Container::new(Null)
                .fix_size(Size::new(5.0, thumb_size))
                .box_style(BoxStyle::new().radius(2.dip()).fill(Color::from_hex("#FF7F31"))),
        )
        .on_started(|| tmp_pos = content_pos)
        .on_delta(|offset| {
            content_pos = (tmp_pos + offset.y / content_to_thumb).clamp(0.0, content_max);
        });

        let content_viewport = Viewport::new(content).transform(Offset::new(0.0, -content_pos).to_transform());
        let scroll_bar = Viewport::new(scroll_thumb).transform(Offset::new(0.0, thumb_pos).to_transform());

        grid_container.contents_mut().add_item(0, 0, content_viewport);
        grid_container.contents_mut().add_item(0, 1, scroll_bar);
        ScrollArea { inner: grid_container }
    }
}

impl Widget for ScrollArea {
    fn widget_id(&self) -> Option<WidgetId> {
        self.inner.widget_id()
    }

    fn event(&self, ctx: &mut EventCtx, event: &mut Event, env: &Environment) {
        self.inner.event(ctx, event, env)
    }

    fn layout(&self, ctx: &mut LayoutCtx, constraints: BoxConstraints, env: &Environment) -> Measurements {
        self.inner.layout(ctx, constraints, env)
    }

    fn paint(&self, ctx: &mut PaintCtx, bounds: Rect, transform: Transform, env: &Environment) {
        self.inner.paint(ctx, bounds, transform, env)
    }
}
