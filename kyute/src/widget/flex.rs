use crate::event::{Event, EventCtx};
use crate::renderer::Theme;
use crate::visual::{PaintCtx, NodeReplacer};
use crate::widget::LayoutCtx;
use crate::{
    layout::BoxConstraints, layout::Layout, layout::Offset, layout::PaintLayout, layout::Size,
    visual::Node, visual::Visual, widget::Widget, widget::WidgetExt, Bounds, BoxedWidget, Point,
};
use euclid::{Point2D, UnknownUnit};
use log::trace;
use std::any::Any;

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

pub struct Flex<A> {
    axis: Axis,
    children: Vec<BoxedWidget<A>>,
}

impl<A: 'static> Flex<A> {
    pub fn new(main_axis: Axis) -> Self {
        Flex {
            axis: main_axis,
            children: Vec::new(),
        }
    }

    pub fn push(mut self, child: impl Widget<A> + 'static) -> Self {
        self.children.push(child.boxed());
        self
    }
}

impl<A: 'static> Widget<A> for Flex<A>
{
    type Visual = FlexVisual;

    fn layout(
        self,
        ctx: &mut LayoutCtx<A>,
        node: Option<Node<FlexVisual>>,
        constraints: &BoxConstraints,
        theme: &Theme
    ) -> Node<FlexVisual>
    {
        let mut node = node.unwrap_or(Node::new(Layout::default(), None, FlexVisual));

        let axis = self.axis;

        {
            // layout child nodes
            let mut placer = node.children.replacer();
            for c in self.children.into_iter() {
                c.layout(ctx, &mut placer, constraints, theme)
            }
        }

        let max_cross_axis_len = node
            .children
            .list
            .iter()
            .map(|s| axis.cross_len(s.borrow().layout.size))
            .fold(0.0, f64::max);

        // preferred size of this flex: max size in axis direction, max elem width in cross-axis direction
        let cross_axis_len = match axis {
            Axis::Vertical => constraints.constrain_width(max_cross_axis_len),
            Axis::Horizontal => constraints.constrain_height(max_cross_axis_len),
        };

        // distribute children
        let mut x = 0.0;
        for child in node.children.list.iter() {
            let mut child = child.borrow_mut();
            let len = axis.main_len(child.layout.size);
            // offset children
            match axis {
                Axis::Vertical => child.layout.offset += Offset::new(0.0, x),
                Axis::Horizontal => child.layout.offset += Offset::new(x, 0.0),
            };
            x += dbg!(len);
        }

        let size = match axis {
            Axis::Vertical => Size::new(cross_axis_len, constraints.max_height()),
            Axis::Horizontal => Size::new(constraints.max_width(), cross_axis_len),
        };

        node.layout = Layout::new(size);
        node
    }
}


pub struct FlexVisual;

impl Default for FlexVisual {
    fn default() -> Self {
        FlexVisual
    }
}

impl Visual for FlexVisual {
    fn paint(&mut self, ctx: &mut PaintCtx, theme: &Theme) {
        let bounds = ctx.bounds();
        theme.draw_panel_background(ctx, bounds);
    }

    fn hit_test(&mut self, _point: Point, _bounds: Bounds) -> bool {
        unimplemented!()
    }

    fn event(&mut self, event_ctx: &mut EventCtx, event: &Event) {
        //unimplemented!()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
