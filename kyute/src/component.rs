use crate::event::Event;
use crate::{
    BoxConstraints, BoxedWidget, Environment, EventCtx, LayoutCtx, Measurements, PaintCtx, Point,
    Rect, TypedWidget, Visual, Widget,
};
use generational_indextree::NodeId;
use std::any::Any;
use std::any::TypeId;

use crate::node::{NodeData, NodeTree};
use std::cell::RefCell;
use std::marker::PhantomData;
use std::rc::Rc;

/// An action that modifies a particular node in the node tree.
pub(crate) struct Action {
    pub(crate) target: NodeId,
    pub(crate) run: Box<dyn FnOnce(&mut NodeData) -> Update + 'static>,
}

impl Action {
    pub(crate) fn new(
        target: NodeId,
        run: impl FnOnce(&mut NodeData) -> Update + 'static,
    ) -> Action {
        Action {
            target,
            run: Box::new(run),
        }
    }
}

/// Helper type to send commands
///
/// TODO better documentation.
#[derive(Clone)]
pub struct CommandSink<S> {
    target: NodeId,
    _phantom: PhantomData<*const S>,
}

impl<S: State> CommandSink<S> {
    pub fn emit(&self, c: S::Cmd) -> impl FnMut(&mut EventCtx) {
        let target = self.target;
        move |ectx| {
            let c = c.clone();
            ectx.push_action(Action::new(target, move |node| {
                dispatch_command::<S>(target, node, c)
            }));
        }
    }
}

/// In charge of forwarding commands to the component.
fn dispatch_command<S: State>(id: NodeId, node: &mut NodeData, cmd: S::Cmd) -> Update {
    node.visual // access the visual impl inside ...
        .as_mut() // .. by mut ref
        .expect("no visual")
        .as_any_mut() // convert to any
        .downcast_mut::<StateWrapper<S>>() // downcast to the expected component wrapper type
        .expect("not a component wrapper")
        .0
        .command(cmd)
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
    fn view<'a>(
        &'a self,
        state: &'a mut Self::State,
        cmd_sink: CommandSink<Self::State>,
    ) -> BoxedWidget<'a>;

    /// Creates a new instance of the component
    fn mount(&self) -> Self::State
    where
        Self: Sized;
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum Update {
    None,
    Relayout,
    Repaint,
}

impl Default for Update {
    fn default() -> Self {
        Update::None
    }
}

pub trait State: 'static {
    /// The type of the commands.
    type Cmd: Clone + 'static;

    /// Handles a command that modifies the state.
    fn command(&mut self, command: Self::Cmd) -> Update;
}

// Public because it's exposed as the visual type of components
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

impl<C: Component> TypedWidget for C {
    type Visual = StateWrapper<C::State>;

    fn layout(
        self,
        context: &mut LayoutCtx,
        previous_visual: Option<Box<StateWrapper<C::State>>>,
        constraints: &BoxConstraints,
        env: Environment,
    ) -> (Box<StateWrapper<C::State>>, Measurements) {
        // extract the component instance from the visual wrapper
        let mut wrapper = if let Some(wrapper) = previous_visual {
            wrapper
        } else {
            // start task
            Box::new(StateWrapper(self.mount()))
        };

        // create the channel for receiving and dispatching the commands emitted during event propagation
        let widget = self.view(
            &mut wrapper.0,
            CommandSink {
                target: context.node_id(),
                _phantom: PhantomData,
            },
        );
        let (_, measurements) = context.emit_child(widget, constraints, env, None);

        (wrapper, measurements)
    }
}
