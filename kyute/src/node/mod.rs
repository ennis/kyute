use crate::layout::Measurements;
use crate::visual::Visual;
use crate::{Rect, DummyVisual, Environment, Size, Offset, Point};
use generational_indextree::NodeId;
use kyute_shell::drawing::DrawContext;
use kyute_shell::platform::Platform;
use std::cell::Cell;

mod event;
mod layout;
mod paint;

pub use self::event::EventCtx;
pub use self::event::FocusState;
pub use self::event::RepaintRequest;
pub use self::layout::LayoutCtx;
pub use self::paint::PaintCtx;
pub use self::paint::DebugLayout;
pub use self::paint::PaintOptions;
use std::any::TypeId;
use winit::window::WindowId;

/// A node within the visual tree.
///
/// It contains the bounds of the visual, and an instance of [`Visual`] that defines its behavior:
/// painting, hit-testing, and how it responds to events that target the visual.
pub(crate) struct NodeData<V: ?Sized = dyn Visual> {
    /// Offset of the node relative to the parent element
    pub(crate) offset: Offset,
    /// Layout of the node (size and baseline).
    pub(crate) measurements: Measurements,
    /// Position of the node in window coordinates.
    pub(crate) window_pos: Cell<Point>,
    /// Key associated to the node.
    pub(crate) key: Option<u64>,
    /// Defines the painting, hit-testing, and event behaviors.
    pub(crate) visual: Option<Box<V>>,
    /// Environment
    pub(crate) env: Environment,
}

impl NodeData<dyn Visual> {
    pub(crate) fn new(key: Option<u64>, env: Environment) -> NodeData<dyn Visual> {
        NodeData {
            offset: Default::default(),
            measurements: Default::default(),
            window_pos: Cell::new(Default::default()),
            key,
            visual: None,
            env,
        }
    }

    /// Creates a dummy node.
    pub(crate) fn dummy(env: Environment) -> NodeData<dyn Visual> {
        NodeData {
            offset: Default::default(),
            measurements: Default::default(),
            window_pos: Cell::new(Default::default()),
            key: None,
            visual: Some(Box::new(DummyVisual::default())),
            env,
        }
    }

    pub(crate) fn visual_type_id(&self) -> Option<TypeId> {
        self.visual.as_ref().map(|v| v.as_ref().type_id())
    }

    ///
    pub(crate) fn window_id(&self) -> Option<WindowId> {
        self.visual.as_ref().and_then(|v| v.window_id())
    }

    /*/// Downcasts this node to a concrete type.
    pub(crate) fn downcast_mut<V: Visual>(&mut self) -> Option<&mut NodeData<V>> {
        if self.visual.as_any().is::<V>() {
            // SAFETY: see <dyn Any>::downcast_mut in std
            // TODO: this may be somewhat different since it's a DST?
            unsafe { Some(&mut *(self as *mut Self as *mut NodeData<V>)) }
        } else {
            None
        }
    }*/
}

/// Container for trees of nodes.
pub(crate) type NodeArena = generational_indextree::Arena<NodeData>;

/// A tree of visual nodes representing the user interface elements shown in a window.
///
/// Contrary to the widget tree, those nodes are retained (as much as possible) across data updates
/// and relayouts. It is incrementally updated by [widgets](crate::widget::Widget) during layout.
///
/// See also: [`Widget::layout`](crate::widget::Widget::layout).
pub struct NodeTree {
    pub(crate) arena: NodeArena,
    root: NodeId,
    //focus: FocusState,
    /// TODO useless?
    window_origin: Point,
}

impl NodeTree {
    /// Creates a new node tree containing a single root node.
    pub fn new() -> NodeTree {
        let mut nodes = NodeArena::new();
        // create the root node
        let root = nodes.new_node(NodeData::dummy(Environment::new()));
        NodeTree {
            arena: nodes,
            root,
            //focus: FocusState::new(),
            window_origin: Point::origin(),
        }
    }
}
