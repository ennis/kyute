use crate::event::Event;
use crate::widget::align::Align;
use crate::widget::constrained::ConstrainedBox;
use crate::widget::{Axis, Baseline, Flex, Text};
use crate::{Alignment, Bounds, BoxConstraints, BoxedWidget, Measurements, Point, Visual, Widget, WidgetExt, LayoutCtx, TypedWidget, Environment};
use generational_indextree::NodeId;
use kyute_shell::drawing::{Color, RectExt};
use std::any::Any;
use crate::widget::flex::FlexVisual;

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

impl<A: 'static> TypedWidget<A> for Form<A>
{
    type Visual = FlexVisual;

    /// Performs layout, consuming the widget.
    fn layout(
        self,
        context: &mut LayoutCtx<A>,
        previous_visual: Option<Box<FlexVisual>>,
        constraints: &BoxConstraints,
        env: Environment,
    ) -> (Box<FlexVisual>, Measurements)
    {
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
        vbox.layout(context, previous_visual, constraints, env)
    }
}
