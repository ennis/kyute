use crate::event::{Event, MoveFocusDirection};
use crate::{
    layout::BoxConstraints, layout::Measurements, layout::Offset, layout::Size, theme, Bounds,
    BoxedWidget, Environment, EventCtx, LayoutCtx, PaintCtx, Point, TypedWidget, Visual, Widget,
    WidgetExt,
};
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

pub struct Flex {
    axis: Axis,
    children: Vec<BoxedWidget>,
}

impl Flex {
    pub fn new(main_axis: Axis) -> Self {
        Flex {
            axis: main_axis,
            children: Vec::new(),
        }
    }

    pub fn push(mut self, child: impl Widget + 'static) -> Self {
        self.children.push(child.boxed());
        self
    }

    pub fn push_boxed(mut self, child: BoxedWidget) -> Self {
        self.children.push(child);
        self
    }
}

impl TypedWidget for Flex {
    type Visual = FlexVisual;

    fn layout(
        self,
        context: &mut LayoutCtx,
        previous_visual: Option<Box<FlexVisual>>,
        constraints: &BoxConstraints,
        env: Environment,
    ) -> (Box<FlexVisual>, Measurements) {
        let visual = previous_visual.unwrap_or_default();

        let axis = self.axis;

        // layout child nodes
        let mut child_nodes = Vec::with_capacity(self.children.len());
        for c in self.children.into_iter() {
            child_nodes.push(context.emit_child(c, constraints, env.clone()));
        }

        let max_cross_axis_len = child_nodes
            .iter()
            .map(|(_, m)| axis.cross_len(m.size))
            .fold(0.0, f64::max);

        // preferred size of this flex: max size in axis direction, max elem width in cross-axis direction
        let cross_axis_len = match axis {
            Axis::Vertical => constraints.constrain_width(max_cross_axis_len),
            Axis::Horizontal => constraints.constrain_height(max_cross_axis_len),
        };

        // distribute children
        let mut d = 0.0;
        let spacing = env.get(theme::FlexSpacing);
        for (id, m) in child_nodes.iter() {
            let len = axis.main_len(m.size);
            // offset children
            let offset = match axis {
                Axis::Vertical => Offset::new(0.0, d),
                Axis::Horizontal => Offset::new(d, 0.0),
            };
            context.set_child_offset(*id, offset);
            d += len + spacing;
        }

        let size = match axis {
            Axis::Vertical => Size::new(cross_axis_len, constraints.constrain_height(d)),
            Axis::Horizontal => Size::new(constraints.constrain_width(d), cross_axis_len),
        };

        (visual, Measurements::new(size))
    }
}

#[derive(Default)]
pub struct FlexVisual;

impl Visual for FlexVisual {
    fn paint(&mut self, ctx: &mut PaintCtx, env: &Environment) {
        let bounds = ctx.bounds();
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
