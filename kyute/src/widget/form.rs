use crate::event::Event;
use crate::renderer::Theme;
use crate::visual::reconciliation::NodePlace;
use crate::visual::{EventCtx, PaintCtx};
use crate::widget::{Axis, Baseline, Flex, LayoutCtx, Text};
use crate::{Bounds, BoxConstraints, BoxedWidget, Layout, Node, Point, Visual, Widget, WidgetExt, Alignment};
use kyute_shell::drawing::{Color, RectExt};
use std::any::Any;
use crate::widget::align::Align;
use crate::widget::constrained::ConstrainedBox;


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
    fn layout<'a>(
        self,
        ctx: &mut LayoutCtx<A>,
        place: &'a mut dyn NodePlace,
        constraints: &BoxConstraints,
        theme: &Theme,
    ) -> &'a mut Node {
        let mut vbox = Flex::new(Axis::Vertical);
        for f in self.fields.into_iter() {
            vbox = vbox.push(
                Flex::new(Axis::Horizontal)
                    .push(Baseline::new(20.0,
                                        ConstrainedBox::new(BoxConstraints::new(100.0..100.0 , ..),
                                                            Align::new(Alignment::TOP_RIGHT, Text::new(f.label)))))
                    .push(Baseline::new(20.0, f.widget)),
            );
        }
        vbox.layout(ctx, place, constraints, theme)
    }
}
