use crate::widget::align::Align;
use crate::widget::constrained::ConstrainedBox;
use crate::widget::flex::FlexVisual;
use crate::widget::padding::Padding;
use crate::widget::{Axis, Baseline, Flex, Text};
use crate::{
    Alignment, BoxConstraints, BoxedWidget, Environment, LayoutCtx, Measurements, Point, Rect,
    SideOffsets, TypedWidget, Visual, Widget, WidgetExt,
};
use generational_indextree::NodeId;
use std::any::Any;

pub struct Field<'a> {
    label: String,
    widget: BoxedWidget<'a>,
}

pub struct Form<'a> {
    fields: Vec<Field<'a>>,
}

impl<'a> Form<'a> {
    pub fn new() -> Form<'a> {
        Form { fields: Vec::new() }
    }

    pub fn field(mut self, label: impl Into<String>, widget: impl Widget + 'static) -> Form<'a> {
        self.fields.push(Field {
            label: label.into(),
            widget: widget.boxed(),
        });
        self
    }
}

impl<'a> TypedWidget for Form<'a> {
    type Visual = FlexVisual;

    /// Performs layout, consuming the widget.
    fn layout(
        self,
        context: &mut LayoutCtx,
        previous_visual: Option<Box<FlexVisual>>,
        constraints: &BoxConstraints,
        env: Environment,
    ) -> (Box<FlexVisual>, Measurements) {
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
                    .push(Padding::new(
                        SideOffsets::new(0.0, 0.0, 0.0, 4.0),
                        Baseline::new(20.0, f.widget),
                    )),
            );
        }
        TypedWidget::layout(vbox, context, previous_visual, constraints, env)
    }
}
