use crate::{BoxedWidget, Widget, BoxConstraints, Measurements, LayoutCtx, Environment, Visual, TypedWidget, EventCtx, PaintCtx, Rect, Point};
use generational_indextree::NodeId;
use std::any::TypeId;
use crate::event::Event;
use std::any::Any;

/*
/// Components are self-contained GUI elements with internal state (retained), and that produce a widget tree
/// when asked.
/// Components are associated (_anchored_) to a _anchor node_ in the NodeTree. The component is in charge of
/// producing the node tree at the anchor.
///
/// TODO proper description
pub trait Component<'a> {
    type Params: Clone + 'a;

    fn view(&mut self, params: Self::Params) -> BoxedWidget;

    fn new(params: Self::Params) -> Self where Self: Sized;
}

/// A wrapper for a component in the node tree.
struct ComponentWrapper<C: for<'a> Component<'a>> {
    component: C,
}

impl<C: for<'a> Component<'a>> Visual for ComponentWrapper<C> {
    fn paint(&mut self, ctx: &mut PaintCtx, env: &Environment) {}
    fn hit_test(&mut self, point: Point, bounds: Rect) -> bool { false }
    fn event(&mut self, event_ctx: &mut EventCtx, event: &Event) {}
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

struct ComponentWidget<'a, C: Component<'a>> {
    params: C::Params,
}

impl<C: for<'a> Component<'a>> TypedWidget for ComponentWidget<C>
{
    type Visual = ComponentWrapper<C>;

    fn layout(self,
              context: &mut LayoutCtx,
              previous_visual: Option<Box<ComponentWrapper<'a, C>>>,
              constraints: &BoxConstraints,
              env: Environment) -> (Box<ComponentWrapper<'a, C>>, Measurements)
    {
        // extract the component instance from the visual wrapper
        let wrapper = if let Some(wrapper) = previous_visual {
            wrapper
        } else {
            C::new(self.params.clone())
        };

        let (_, measurements) = context.emit_child(
            wrapper.component.view(self.params), constraints, env);

        (wrapper, measurements)
    }
}


*/