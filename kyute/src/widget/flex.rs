use crate::{
    style::{BoxStyle, PaintCtxExt},
    theme,
    widget::prelude::*,
    RoundToPixel,
};
use std::sync::Arc;

pub fn main_axis_length(orientation: Orientation, size: Size) -> f64 {
    match orientation {
        Orientation::Vertical => size.height,
        Orientation::Horizontal => size.width,
    }
}

pub fn cross_axis_length(orientation: Orientation, size: Size) -> f64 {
    match orientation {
        Orientation::Vertical => size.width,
        Orientation::Horizontal => size.height,
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum MainAxisAlignment {
    Start,
    Center,
    End,
    SpaceBetween,
    SpaceEvenly,
    SpaceAround,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum CrossAxisAlignment {
    Baseline,
    Start,
    Center,
    End,
    Stretch,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum MainAxisSize {
    Min,
    Max,
}

#[derive(Clone)]
pub struct Flex {
    id: WidgetId,
    axis_orientation: Orientation,
    items: Vec<Arc<WidgetPod>>,
}

impl Flex {
    #[deprecated(note = "use Grid::row() and Grid::column() instead")]
    #[composable]
    pub fn new(axis_orientation: Orientation) -> Flex {
        Flex {
            id: WidgetId::here(),
            axis_orientation,
            items: vec![],
        }
    }

    /*#[deprecated(note = "use Grid::row() and Grid::column() instead")]
    pub fn horizontal() -> Flex {
        Flex::new(Orientation::Horizontal)
    }

    #[deprecated(note = "use Grid::row() and Grid::column() instead")]
    pub fn vertical() -> Flex {
        Flex::new(Orientation::Vertical)
    }*/

    #[composable]
    pub fn with(mut self, widget: impl Widget + 'static) -> Self {
        self.push(widget);
        self
    }

    #[composable]
    pub fn push(&mut self, widget: impl Widget + 'static) {
        self.items.push(Arc::new(WidgetPod::new(widget)));
    }
}

impl Widget for Flex {
    fn widget_id(&self) -> Option<WidgetId> {
        Some(self.id)
    }

    fn debug_name(&self) -> &str {
        std::any::type_name::<Self>()
    }

    fn event(&self, ctx: &mut EventCtx, event: &mut Event, env: &Environment) {
        for item in self.items.iter() {
            item.event(ctx, event, env);
        }
    }

    fn layout(&self, ctx: &mut LayoutCtx, constraints: BoxConstraints, env: &Environment) -> Measurements {
        let item_measures: Vec<Measurements> = self
            .items
            .iter()
            .map(|item| item.layout(ctx, constraints, env))
            .collect();

        let max_cross_axis_len = item_measures
            .iter()
            .map(|m| cross_axis_length(self.axis_orientation, m.size()))
            .fold(0.0, f64::max);

        // preferred size of this flex: max size in axis direction, max elem width in cross-axis direction
        let cross_axis_len = match self.axis_orientation {
            Orientation::Vertical => constraints.constrain_width(max_cross_axis_len),
            Orientation::Horizontal => constraints.constrain_height(max_cross_axis_len),
        };

        // distribute children
        let mut d = 0.0;
        //let spacing = env.get(theme::FlexSpacing);
        let spacing = 1.0;

        for i in 0..self.items.len() {
            //eprintln!("flex {:?} item pos {}", self.axis, d);
            let len = main_axis_length(self.axis_orientation, item_measures[i].size()).round_to_pixel(ctx.scale_factor);
            let offset = match self.axis_orientation {
                Orientation::Vertical => Offset::new(0.0, d),
                Orientation::Horizontal => Offset::new(d, 0.0),
            };
            self.items[i].set_child_offset(offset);
            d += len + spacing;
            d = d.ceil();
        }

        let size = match self.axis_orientation {
            Orientation::Vertical => Size::new(cross_axis_len, constraints.constrain_height(d)),
            Orientation::Horizontal => Size::new(constraints.constrain_width(d), cross_axis_len),
        };

        Measurements::new(Rect::new(Point::origin(), size).round_to_pixel(ctx.scale_factor))
    }

    fn paint(&self, ctx: &mut PaintCtx, bounds: Rect, env: &Environment) {
        ctx.draw_styled_box(
            bounds,
            &BoxStyle::new().fill(theme::keys::CONTROL_BACKGROUND_COLOR),
            env,
        );

        for item in self.items.iter() {
            // eprintln!("flex {:?} paint item {:?}", self.axis, item.child_offset());
            item.paint(ctx, bounds, env);
        }
    }
}
