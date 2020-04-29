use crate::event::{Event, MoveFocusDirection};
use crate::renderer::Theme;
use crate::visual::NodeCursor;
use crate::visual::{EventCtx, NodeArena, PaintCtx};
use crate::widget::LayoutCtx;
use crate::{
    layout::BoxConstraints, layout::Layout, layout::Offset, layout::Size, visual::NodeData,
    visual::Visual, widget::Widget, widget::WidgetExt, Bounds, BoxedWidget, Point,
};
use euclid::{Point2D, UnknownUnit};
use generational_indextree::NodeId;
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
    fn layout(
        self,
        ctx: &mut LayoutCtx<A>,
        nodes: &mut NodeArena,
        cursor: &mut NodeCursor,
        constraints: &BoxConstraints,
        theme: &Theme,
    ) -> NodeId {
        let node: NodeId = cursor.get_or_insert_default::<FlexVisual>(nodes);

        let axis = self.axis;

        {
            // layout child nodes
            let mut cursor = NodeCursor::Child(node);
            for c in self.children.into_iter() {
                c.layout(ctx, nodes, &mut cursor, constraints, theme);
            }
            cursor.remove_after(nodes);
        }

        let max_cross_axis_len = node
            .children(nodes)
            .map(|child_id| axis.cross_len(nodes[child_id].get().layout.size))
            .fold(0.0, f64::max);

        // preferred size of this flex: max size in axis direction, max elem width in cross-axis direction
        let cross_axis_len = match axis {
            Axis::Vertical => constraints.constrain_width(max_cross_axis_len),
            Axis::Horizontal => constraints.constrain_height(max_cross_axis_len),
        };

        // distribute children
        let mut d = 0.0;
        let mut child_id = nodes[node].first_child();
        while let Some(id) = child_id {
            let node = nodes[id].get_mut();
            let len = axis.main_len(node.layout.size);
            // offset children
            match axis {
                Axis::Vertical => node.layout.offset += Offset::new(0.0, d),
                Axis::Horizontal => node.layout.offset += Offset::new(d, 0.0),
            };
            d += len;
            child_id = nodes[id].next_sibling();
        }

        let size = match axis {
            Axis::Vertical => Size::new(cross_axis_len, constraints.constrain_height(d)),
            Axis::Horizontal => Size::new(constraints.constrain_width(d), cross_axis_len),
        };

        nodes[node].get_mut().layout = Layout::new(size);
        node
    }
}

#[derive(Default)]
pub struct FlexVisual;

impl Visual for FlexVisual {
    fn paint(&mut self, ctx: &mut PaintCtx, theme: &Theme) {
        let bounds = ctx.bounds();
        theme.draw_panel_background(ctx, bounds);
    }

    fn hit_test(&mut self, _point: Point, _bounds: Bounds) -> bool {
        unimplemented!()
    }

    fn event(&mut self, ctx: &mut EventCtx, event: &Event) {
        /*match event {
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
        }*/
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
