use crate::widget::{
    grid::GridTrackDefinition, prelude::*, Canvas, Container, DragController, Grid, GridLength, Null, Thumb,
};
use kyute::style::BoxStyle;
use kyute_common::{Color, UnitExt};

#[derive(Clone)]
pub struct ScrollArea {
    inner: Grid,
}

impl ScrollArea {
    #[composable]
    pub fn new(contents: impl Widget + 'static) -> ScrollArea {
        #[state]
        let mut thumb_pos = 0.0;
        #[state]
        let mut tmp_pos = 0.0;

        #[state]
        let mut content_offset: f64 = 0.0;

        let mut scroll_bar = DragController::new(
            Canvas::new().item(
                0.0,
                thumb_pos,
                Container::new(Null)
                    .fix_size(Size::new(5.0, 5.0))
                    .box_style(BoxStyle::new().radius(2.dip()).fill(Color::from_hex("#FF7F31"))),
            ),
        )
        .on_started(|| tmp_pos = thumb_pos)
        .on_delta(|offset| thumb_pos = tmp_pos + offset.y);

        let mut content_grid = Grid::with_column_definitions([
            GridTrackDefinition::new(GridLength::Flex(1.0)),
            GridTrackDefinition::new(GridLength::Fixed(5.0)),
        ]);
        //content_grid.add_item(0, 0, Canvas::new().add_item(Offset::new(contents));
        content_grid.add_item(0, 1, scroll_bar);

        // canvas not enough for scroll bar:
        // - positioning in the canvas depends on the parent

        // - inner canvas has a fixed size (say, 0-100 px)
        // - then positioning can be absolute

        ScrollArea { inner: content_grid }
    }
}
