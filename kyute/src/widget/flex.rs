use crate::event::{Event, MoveFocusDirection};
use crate::renderer::Theme;
use crate::visual::reconciliation::{NodeListReplacer, NodePlace};
use crate::visual::{EventCtx, PaintCtx};
use crate::widget::LayoutCtx;
use crate::{
    layout::BoxConstraints, layout::Layout, layout::Offset, layout::Size,
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

    pub fn push_boxed(mut self, child: BoxedWidget<A>) -> Self {
        self.children.push(child);
        self
    }
}

impl<A: 'static> Widget<A> for Flex<A> {
    fn layout<'a>(
        self,
        ctx: &mut LayoutCtx<A>,
        place: &'a mut dyn NodePlace,
        constraints: &BoxConstraints,
        theme: &Theme,
    ) -> &'a mut Node {
        let node: &mut Node<FlexVisual> = place.get_or_insert_default();

        let axis = self.axis;

        {
            // layout child nodes
            let mut replacer = NodeListReplacer::new(&mut node.visual.children);
            for c in self.children.into_iter() {
                c.layout(ctx, &mut replacer, constraints, theme);
            }
        }

        let max_cross_axis_len = node
            .visual
            .children
            .iter()
            .map(|s| axis.cross_len(s.layout.size))
            .fold(0.0, f64::max);

        // preferred size of this flex: max size in axis direction, max elem width in cross-axis direction
        let cross_axis_len = match axis {
            Axis::Vertical => constraints.constrain_width(max_cross_axis_len),
            Axis::Horizontal => constraints.constrain_height(max_cross_axis_len),
        };

        // distribute children
        let mut x = 0.0;
        for child in node.visual.children.iter_mut() {
            let len = axis.main_len(child.layout.size);
            // offset children
            match axis {
                Axis::Vertical => child.layout.offset += Offset::new(0.0, x),
                Axis::Horizontal => child.layout.offset += Offset::new(x, 0.0),
            };
            x += dbg!(len);
        }

        let size = match axis {
            Axis::Vertical => Size::new(cross_axis_len, constraints.constrain_height(x)),
            Axis::Horizontal => Size::new(constraints.constrain_width(x), cross_axis_len),
        };

        node.layout = Layout::new(size);
        node
    }
}

pub struct FlexVisual {
    children: Vec<Box<Node>>,
}

impl Default for FlexVisual {
    fn default() -> Self {
        FlexVisual {
            children: Vec::new(),
        }
    }
}

impl Visual for FlexVisual {
    fn paint(&mut self, ctx: &mut PaintCtx, theme: &Theme) {
        let bounds = ctx.bounds();
        theme.draw_panel_background(ctx, bounds);

        for c in self.children.iter_mut() {
            c.paint(ctx, theme)
        }
    }

    fn hit_test(&mut self, _point: Point, _bounds: Bounds) -> bool {
        unimplemented!()
    }

    fn event(&mut self, ctx: &mut EventCtx, event: &Event) {
        match event {
            Event::MoveFocus(direction) => {
                // find the focus path
                let mut i = if let Some(i) = self
                    .children
                    .iter()
                    .position(|node| node.is_on_focus_path(ctx))
                {
                    i as isize
                } else {
                    // this container does not contain the focus path
                    return;
                };

                self.children[i as usize].event(ctx, event);

                let len = self.children.len() as isize;
                while !ctx.handled() && (0..len).contains(&i) {
                    i += match direction {
                        MoveFocusDirection::Before => -1,
                        MoveFocusDirection::After => 1,
                    };

                    self.children[i as usize].event(ctx, &Event::SetFocus(*direction));
                }
            }
            // forward all other events
            event => {
                for c in self.children.iter_mut() {
                    c.event(ctx, event);
                    if ctx.handled() {
                        break;
                    }
                }
            }
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
