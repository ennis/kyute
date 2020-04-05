use crate::event::{Event, EventCtx};
use crate::layout::Alignment;
use crate::layout::BoxConstraints;
use crate::layout::EdgeInsets;
use crate::layout::Layout;
use crate::layout::Point;
use crate::layout::Size;
use crate::layout::{Offset, PaintLayout};
use crate::renderer::{ButtonState, Theme};
use crate::visual::{Cursor, Node, PaintCtx};
use crate::visual::{RcNode, Visual};
use crate::widget::Text;
use crate::widget::Widget;
use crate::widget::WidgetExt;
use crate::widget::{BoxedWidget, LayoutCtx};
use crate::Bounds;
use std::any::Any;

/// Button element.
pub struct Button<A> {
    label: BoxedWidget<A>,
    /// Action to emit on button click.
    on_click: Option<A>,
}

impl<A: 'static> Button<A> {
    /// Creates a new button with the given text as the label.
    pub fn new(label: &str) -> Button<A> {
        Button {
            label: Text::new(label).boxed(),
            on_click: None,
        }
    }
}

impl<A: 'static> Widget<A> for Button<A> {
    fn layout(
        self,
        ctx: &mut LayoutCtx<A>,
        tree_cursor: &mut Cursor,
        constraints: &BoxConstraints,
        theme: &Theme,
    ) {
        let on_click = self.on_click;
        let mut node = tree_cursor.open(None, move || ButtonVisual { on_click });
        let node = &mut *node; // reborrow RefMut as &mut, prevents RefMut-related lifetime confusion

        let button_metrics = &theme.button_metrics();

        // measure label
        self.label.layout(
            ctx,
            &mut node.cursor(),
            &constraints.deflate(&EdgeInsets::all(button_metrics.label_padding.into())),
            theme,
        );

        let mut label_node = node.children.first().unwrap().borrow_mut();
        // base button size
        let mut button_size = Size::new(
            label_node.layout.size.width + 2.0 * button_metrics.label_padding,
            label_node.layout.size.height + 2.0 * button_metrics.label_padding,
        );
        // apply minimum dimensions
        button_size = button_size.max(Size::new(
            button_metrics.min_width,
            button_metrics.min_height,
        ));
        // apply parent constraints
        button_size = constraints.constrain(button_size);

        node.layout = Layout::new(button_size);
        Layout::align(&mut node.layout, &mut label_node.layout, Alignment::CENTER);
    }
}

/// Node visual for a button.
pub struct ButtonVisual<A> {
    on_click: Option<A>,
}

impl<A: 'static> Visual for ButtonVisual<A> {
    fn paint(&mut self, ctx: &mut PaintCtx, theme: &Theme) {
        let bounds = ctx.bounds();
        theme.draw_button_frame(
            ctx,
            bounds,
            &ButtonState {
                disabled: false,
                clicked: false,
                hot: true,
            },
        );
    }

    fn hit_test(&mut self, _point: Point, _bounds: Bounds) -> bool {
        false
    }

    fn event(&mut self, event_ctx: &EventCtx, event: &Event) {}

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
