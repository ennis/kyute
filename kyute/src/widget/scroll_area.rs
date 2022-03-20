use crate::{
    style::BoxStyle,
    widget::{
        grid::GridTrackDefinition, prelude::*, Container, DragController, Grid, GridLength, LayoutInspector, Null,
        Viewport, WidgetWrapper,
    },
    Color, UnitExt,
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
                GridTrackDefinition::new(GridLength::Fixed(5.0.dip())),
            ],
        ));

        // HACK: even if the content already fits in the grid container, we still have to wrap
        // the content widget in a Viewport, because otherwise the size returned by the LayoutInspector
        // will always be clamped to the size of the grid.
        let content_viewport =
            Viewport::new(LayoutInspector::new(contents)).transform(Offset::new(0.0, -content_pos).to_transform());

        // TODO check that content size is finite
        // v_height = viewport height
        // c_height = content height
        // t = thumb height
        // t_p = thumb position
        let viewport_height = grid_container.size().height;
        let content_height = content_viewport.contents().size().height;

        if content_height <= viewport_height {
            grid_container.contents_mut().add_item(0, 0, content_viewport);
            return ScrollArea { inner: grid_container };
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
            content_pos = (tmp_pos + offset.y / content_to_thumb).clamp(0.0, content_max);
        });

        let scroll_bar = Viewport::new(scroll_thumb).transform(Offset::new(0.0, thumb_pos).to_transform());

        grid_container.contents_mut().add_item(0, 0, content_viewport);
        grid_container.contents_mut().add_item(0, 1, scroll_bar);
        ScrollArea { inner: grid_container }
    }
}

impl WidgetWrapper for ScrollArea {
    type Inner = LayoutInspector<Grid>;

    fn inner(&self) -> &Self::Inner {
        &self.inner
    }

    fn inner_mut(&mut self) -> &mut Self::Inner {
        &mut self.inner
    }
}
