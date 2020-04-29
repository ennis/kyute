use crate::event::Event;
use crate::renderer::Theme;
use crate::visual::{EventCtx, NodeArena, NodeCursor, PaintCtx};
use crate::widget::align::Align;
use crate::widget::constrained::ConstrainedBox;
use crate::widget::{Axis, Baseline, Flex, LayoutCtx, Text};
use crate::{
    Alignment, Bounds, BoxConstraints, BoxedWidget, Layout, NodeData, Point, Visual, Widget,
    WidgetExt,
};
use generational_indextree::NodeId;
use kyute_shell::drawing::{Color, RectExt};
use std::any::Any;

pub struct Field<A> {
    label: String,
    widget: BoxedWidget<A>,
}

pub struct Form<A> {
    fields: Vec<Field<A>>,
}

impl<A: 'static> Form<A> {
    pub fn new() -> Form<A> {
        Form { fields: Vec::new() }
    }

    pub fn field(mut self, label: impl Into<String>, widget: impl Widget<A> + 'static) -> Form<A> {
        self.fields.push(Field {
            label: label.into(),
            widget: widget.boxed(),
        });
        self
    }
}

// constraints!(100 x _);
// (100 x 100 => 200 x 200

impl<A: 'static> Widget<A> for Form<A> {
    fn layout(
        self,
        ctx: &mut LayoutCtx<A>,
        nodes: &mut NodeArena,
        cursor: &mut NodeCursor,
        constraints: &BoxConstraints,
        theme: &Theme,
    ) -> NodeId {
        let mut vbox = Flex::new(Axis::Vertical);
        for f in self.fields.into_iter() {
            vbox = vbox.push(
                Flex::new(Axis::Horizontal)
                    .push(Baseline::new(
                        20.0,
                        ConstrainedBox::new(
                            BoxConstraints::new(100.0..100.0, ..),
                            Align::new(Alignment::TOP_RIGHT, Text::new(f.label)),
                        ),
                    ))
                    .push(Baseline::new(20.0, f.widget)),
            );
        }
        vbox.layout(ctx, nodes, cursor, constraints, theme)
    }
}
