use kyute_shell::drawing::{Color, RectExt};
use crate::{BoxedWidget, Widget, BoxConstraints, Node, Layout, Visual, Point, Bounds, WidgetExt};
use crate::widget::{LayoutCtx, Flex, Axis, Text, Baseline};
use crate::visual::reconciliation::NodePlace;
use crate::renderer::Theme;
use crate::visual::{PaintCtx, EventCtx};
use std::any::Any;
use crate::event::Event;

pub struct Field<A> {
    label: String,
    widget: BoxedWidget<A>,
}

pub struct Form<A> {
    fields: Vec<Field<A>>
}

impl<A: 'static> Form<A> {
    pub fn new() -> Form<A> {
        Form {
            fields: Vec::new()
        }
    }

    pub fn field(mut self, label: impl Into<String>, widget: impl Widget<A> + 'static) -> Form<A> {
        self.fields.push(Field {
            label: label.into(),
            widget: widget.boxed()
        });
        self
    }
}

impl<A: 'static> Widget<A> for Form<A> {
    fn layout<'a>(
        self,
        ctx: &mut LayoutCtx<A>,
        place: &'a mut dyn NodePlace,
        constraints: &BoxConstraints,
        theme: &Theme,
    ) -> &'a mut Node
    {
        let mut vbox = Flex::new(Axis::Vertical);
        for f in self.fields.into_iter() {
            vbox = vbox.push(
                Flex::new(Axis::Horizontal)
                    .push(Baseline::new(20.0, Text::new(f.label)))
                    .push(Baseline::new(20.0, f.widget))
            );
        }
        vbox.layout(ctx, place, constraints, theme)
    }
}
