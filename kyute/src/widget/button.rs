use crate::event::Event;
use crate::layout::Alignment;
use crate::layout::BoxConstraints;
use crate::layout::EdgeInsets;
use crate::layout::Layout;
use crate::layout::Offset;
use crate::layout::Point;
use crate::layout::Size;
use crate::renderer::{ButtonState, Theme};
use crate::visual::{EventCtx, NodeData, PaintCtx};
use crate::visual::{NodeArena, NodeCursor, Visual};
use crate::widget::Text;
use crate::widget::Widget;
use crate::widget::WidgetExt;
use crate::widget::{BoxedWidget, LayoutCtx};
use crate::Bounds;
use generational_indextree::NodeId;
use std::any::Any;

/// Node visual for a button.
pub struct ButtonVisual<A> {
    on_click: Option<A>,
}

impl<A> Default for ButtonVisual<A> {
    fn default() -> Self {
        ButtonVisual { on_click: None }
    }
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

    fn event(&mut self, ctx: &mut EventCtx, event: &Event) {
        match event {
            Event::PointerDown(p) => {
                eprintln!("BUTTON CLICKED");
                ctx.set_handled();
            }
            _ => {}
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

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
    fn layout<'a>(
        self,
        ctx: &mut LayoutCtx<A>,
        nodes: &mut NodeArena,
        cursor: &mut NodeCursor,
        constraints: &BoxConstraints,
        theme: &Theme,
    ) -> NodeId {
        let on_click = self.on_click;
        let node_id = cursor.reconcile(nodes, move |_old: Option<NodeData<ButtonVisual<A>>>| {
            NodeData::new(Layout::default(), None, ButtonVisual { on_click })
        });

        let button_metrics = &theme.button_metrics();

        let label_id = self.label.layout_child(
            ctx,
            nodes,
            node_id,
            &constraints.deflate(&EdgeInsets::all(button_metrics.label_padding.into())),
            theme,
        );

        // base button size
        let mut label_layout = nodes[label_id].get().layout;
        let mut button_size = Size::new(
            label_layout.size.width + 2.0 * button_metrics.label_padding,
            label_layout.size.height + 2.0 * button_metrics.label_padding,
        );

        // apply minimum dimensions
        button_size = button_size.max(Size::new(
            button_metrics.min_width,
            button_metrics.min_height,
        ));
        // apply parent constraints
        button_size = constraints.constrain(button_size);

        let mut node_layout = Layout::new(button_size);
        Layout::align(&mut node_layout, &mut label_layout, Alignment::CENTER);

        nodes[node_id].get_mut().layout = node_layout;
        nodes[label_id].get_mut().layout = label_layout;
        node_id
    }
}
