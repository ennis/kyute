use crate::event::Event;
use crate::{
    BoxConstraints, BoxedWidget, Environment, EventCtx, LayoutCtx, Measurements, PaintCtx, Point,
    Rect, TypedWidget, Visual, Widget,
};
use generational_indextree::NodeId;
use std::any::Any;
use std::any::TypeId;

use crate::node::NodeTree;
use futures::channel::mpsc::{Sender, Receiver, channel};
use futures::{Stream, Sink};
use futures::StreamExt;
use std::cell::RefCell;
use std::rc::Rc;
use futures::task::LocalSpawnExt;
use crate::application::NodeTreeHandle;


pub struct CommandSink<C>(Sender<C>);

impl <C: Clone + 'static> CommandSink<C> {
    pub fn emit(&self, c: C) -> impl FnMut(&mut EventCtx) {
        let mut s = self.0.clone();
        move |_| {
            s.try_send(c.clone());
        }
    }
}

/// Reusable GUI elements with internal state.
///
/// Components are associated (_anchored_) to a _anchor node_ in the NodeTree. The component is in charge of
/// producing the node tree at the anchor.
///
/// TODO proper description
pub trait Component {
    type State: State;

    /// Returns the view.
    fn view<'a>(&'a self, state: &'a mut Self::State, cmd_sink: CommandSink<<Self::State as State>::Cmd>) -> BoxedWidget<'a>;

    /// Creates a new instance of the component
    fn mount(&self) -> Self::State
    where
        Self: Sized;
}

pub trait State: 'static {
    /// The type of the commands.
    type Cmd: Clone + 'static;

    /// Handles a command that modifies the state.
    fn command(&mut self, command: Self::Cmd);
}

//
#[doc(hidden)]
pub struct StateWrapper<S: State>(S);

impl<S: State> Visual for StateWrapper<S> {
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


impl<C: Component> TypedWidget for C
{
    type Visual = StateWrapper<C::State>;

    fn layout(
        self,
        context: &mut LayoutCtx,
        previous_visual: Option<Box<StateWrapper<C::State>>>,
        constraints: &BoxConstraints,
        env: Environment,
    ) -> (Box<StateWrapper<C::State>>, Measurements)
    {
        // extract the component instance from the visual wrapper
        let mut wrapper = if let Some(wrapper) = previous_visual {
            wrapper
        } else {
            // start task
            Box::new(StateWrapper(self.mount()))
        };

        // create the channel for receiving and dispatching the commands emitted during event propagation
        let (tx,rx) = channel::<<C::State as State>::Cmd>(10);
        let tree_handle = context.win_ctx.tree_handle.clone();
        let id = context.node_id();
        context.win_ctx.spawner.spawn_local(command_forwarder::<C::State>(tree_handle, id, rx));

        let widget = self.view(&mut wrapper.0, CommandSink(tx));
        let (_, measurements) = context.emit_child(widget, constraints, env, None);

        (wrapper, measurements)
    }
}



/// In charge of forwarding commands to the component.
async fn command_forwarder<S: State>(
    tree: NodeTreeHandle,
    node: NodeId,
    mut commands: Receiver<S::Cmd>,
) {
    while let Some(command) = commands.next().await {
        // received a command, lock the tree and send it to the component
        let mut tree = tree.borrow_mut();
        let tree = tree.as_mut().expect("no tree");

        // assert dominance
        tree.arena
            .get_mut(node) // get node in node tree arena
            .expect("node deleted")
            .get_mut() // access internal data
            .visual // access the visual impl inside ...
            .as_mut() // .. by mut ref
            .expect("no visual")
            .as_any_mut() // convert to any
            .downcast_mut::<StateWrapper<S>>() // downcast to the expected component wrapper type
            .expect("not a component wrapper")
            .0
            .command(command);
        // component may spawn new tasks or emit other commands in response
    }

    // broken channel, end task
}
