use crate::event::Event;
use crate::{
    BoxConstraints, BoxedWidget, Environment, EventCtx, LayoutCtx, Measurements, PaintCtx, Point,
    Rect, TypedWidget, Visual, Widget,
};
use generational_indextree::NodeId;
use std::any::Any;
use std::any::TypeId;

use crate::node::NodeTree;
use futures::channel::mpsc::{Receiver, channel};
use futures::Stream;
use futures::StreamExt;
use std::cell::RefCell;
use std::rc::Rc;

/// Reusable GUI elements with internal state.
///
/// Components are associated (_anchored_) to a _anchor node_ in the NodeTree. The component is in charge of
/// producing the node tree at the anchor.
///
/// TODO proper description
pub trait Component<'a>: 'static {
    /// The type of the parameters that the component expects.
    type Params: Clone + 'a;
    /// The type of the commands that the component receives.
    type Cmd: Clone + 'static;

    /// Handles a command.
    fn command(&mut self, command: Self::Cmd);

    /// Returns the view.
    fn view(&mut self, params: Self::Params) -> BoxedWidget;

    /// Creates a new instance of the component
    fn new(params: Self::Params) -> Self
    where
        Self: Sized;
}

/// A wrapper for a component in the node tree.
pub struct ComponentWrapper<C: for<'a> Component<'a>>(C);

impl<C: for<'a> Component<'a>> Visual for ComponentWrapper<C> {
    fn paint(&mut self, ctx: &mut PaintCtx, env: &Environment) {}
    fn hit_test(&mut self, point: Point, bounds: Rect) -> bool {
        false
    }
    fn event(&mut self, event_ctx: &mut EventCtx, event: &Event) {}
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

pub struct ComponentWidget<'a, C: Component<'a>> {
    params: C::Params,
}

impl<'a, C: Component<'a>> ComponentWidget<'a, C> {
    pub fn new(params: C::Params) -> ComponentWidget<'a, C> {
        ComponentWidget { params }
    }
}

impl<'a, C: for<'x> Component<'x>> TypedWidget for ComponentWidget<'a, C> {
    type Visual = ComponentWrapper<C>;

    fn layout(
        self,
        context: &mut LayoutCtx,
        previous_visual: Option<Box<ComponentWrapper<C>>>,
        constraints: &BoxConstraints,
        env: Environment,
    ) -> (Box<ComponentWrapper<C>>, Measurements) {
        // extract the component instance from the visual wrapper
        let mut wrapper = if let Some(wrapper) = previous_visual {
            wrapper
        } else {
            // start task
            Box::new(ComponentWrapper(C::new(self.params.clone())))
        };

       // let (tx,rx) = channel(10);

        let widget = wrapper.0.view(self.params);
        let (_, measurements) = context.emit_child(widget, constraints, env);

        (wrapper, measurements)
    }
}

/*
/// In charge of forwarding commands to the component.
async fn command_forwarder<C: for<'a> Component<'a>>(
    tree: Rc<RefCell<NodeTree>>,
    node: NodeId,
    mut commands: Receiver<C::Cmd>,
) {
    while let Some(command) = commands.next().await {
        // received a command, lock the tree and send it to the component
        let tree = tree.borrow_mut();

        // assert dominance
        tree.arena
            .get_mut(node) // get node in node tree arena
            .expect("node deleted")
            .get_mut() // access internal data
            .visual // access the visual impl inside ...
            .as_mut() // .. by mut ref
            .expect("no visual")
            .as_any_mut() // convert to any
            .downcast_mut::<ComponentWrapper<C>>() // downcast to the expected component wrapper type
            .expect("not a component wrapper")
            .0
            .command(command); // access component inside, send command
        // component may spawn new tasks or emit other commands in response
    }

    // broken channel, end task
}*/
