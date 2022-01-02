use crate::{
    composable,
    core2::{LayoutCtx, PaintCtx},
    BoxConstraints, Environment, Event, EventCtx, LayoutItem, Measurements, Offset, Rect, Size,
    Widget, WidgetPod,
};
use kyute_shell::drawing::Color;
use tracing::trace;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Axis {
    Horizontal,
    Vertical,
}

impl Axis {
    pub fn cross_axis(self) -> Axis {
        match self {
            Axis::Horizontal => Axis::Vertical,
            Axis::Vertical => Axis::Horizontal,
        }
    }

    pub fn main_len(self, size: Size) -> f64 {
        match self {
            Axis::Vertical => size.height,
            Axis::Horizontal => size.width,
        }
    }

    pub fn cross_len(self, size: Size) -> f64 {
        match self {
            Axis::Vertical => size.width,
            Axis::Horizontal => size.height,
        }
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

pub struct Flex {
    axis: Axis,
    items: Vec<WidgetPod>,
}

impl Flex {
    #[composable(uncached)]
    pub fn new(axis: Axis, items: Vec<WidgetPod>) -> WidgetPod<Flex> {
        WidgetPod::new(Flex { axis, items })
    }
}

impl Widget for Flex {
    fn debug_name(&self) -> &str {
        std::any::type_name::<Self>()
    }

    fn event(&self, ctx: &mut EventCtx, event: &mut Event) {
        for item in self.items.iter() {
            item.event(ctx, event);
        }
    }

    fn layout(
        &self,
        ctx: &mut LayoutCtx,
        constraints: BoxConstraints,
        env: &Environment,
    ) -> Measurements {
        let axis = self.axis;

        let item_measures: Vec<Measurements> = self
            .items
            .iter()
            .map(|item| item.layout(ctx, constraints, env))
            .collect();

        let max_cross_axis_len = item_measures
            .iter()
            .map(|l| axis.cross_len(l.size()))
            .fold(0.0, f64::max);

        // preferred size of this flex: max size in axis direction, max elem width in cross-axis direction
        let cross_axis_len = match axis {
            Axis::Vertical => constraints.constrain_width(max_cross_axis_len),
            Axis::Horizontal => constraints.constrain_height(max_cross_axis_len),
        };

        // distribute children
        let mut d = 0.0;
        //let spacing = env.get(theme::FlexSpacing);
        let spacing = 1.0;

        for i in 0..self.items.len() {
            //eprintln!("flex {:?} item pos {}", self.axis, d);
            let len = axis.main_len(item_measures[i].size());
            let offset = match axis {
                Axis::Vertical => Offset::new(0.0, d),
                Axis::Horizontal => Offset::new(d, 0.0),
            };
            self.items[i].set_child_offset(offset);
            d += len + spacing;
            d = d.ceil();
        }

        let size = match axis {
            Axis::Vertical => Size::new(cross_axis_len, constraints.constrain_height(d)),
            Axis::Horizontal => Size::new(constraints.constrain_width(d), cross_axis_len),
        };

        Measurements::new(size)
    }

    fn paint(&self, ctx: &mut PaintCtx, bounds: Rect, env: &Environment) {
        //ctx.canvas.clear(kyute_shell::skia::Color4f::new(0.55, 0.55, 0.55, 1.0));
        for item in self.items.iter() {
            // eprintln!("flex {:?} paint item {:?}", self.axis, item.child_offset());
            item.paint(ctx, bounds, env);
        }
    }
}
