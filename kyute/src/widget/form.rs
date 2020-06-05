use crate::widget::align::Align;
use crate::widget::constrained::ConstrainedBox;
use crate::widget::flex::FlexVisual;
use crate::widget::{Axis, Baseline, Flex, Text};
use crate::{
    Alignment, Bounds, BoxConstraints, BoxedWidget, Environment, LayoutCtx, Measurements, Point,
    TypedWidget, Visual, Widget, WidgetExt,
};
use generational_indextree::NodeId;
use std::any::Any;

pub struct Field {
    label: String,
    widget: BoxedWidget,
}

pub struct Form {
    fields: Vec<Field>,
}

impl Form {
    pub fn new() -> Form {
        Form { fields: Vec::new() }
    }

    pub fn field(mut self, label: impl Into<String>, widget: impl Widget + 'static) -> Form {
        self.fields.push(Field {
            label: label.into(),
            widget: widget.boxed(),
        });
        self
    }
}

impl TypedWidget for Form {
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
                    .push(Baseline::new(20.0, f.widget)),
            );
        }
        TypedWidget::layout(vbox, context, previous_visual, constraints, env)
    }
}
