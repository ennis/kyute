//! Elements of the visual tree (after layout): `Visual`s and `Node`s.
use crate::event::{Event, EventCtx, PointerEvent};
use crate::layout::{Layout, Offset, PaintLayout, Point};
use crate::renderer::Theme;
use crate::state::NodeKey;
use crate::{Bounds, BoxConstraints, Widget};
use euclid::{Point2D, UnknownUnit};
use log::trace;
use std::any::{Any, TypeId};
use std::cell::{RefCell, RefMut};
use std::ops::{Deref, Range};
use std::rc::{Rc, Weak};
use std::any;

use kyute_shell::drawing::{Size, Transform};
use kyute_shell::window::DrawContext;
use std::ops::DerefMut;
use crate::application::WindowCtx;
use crate::widget::ActionSink;

/// Context passed to [`Visual::paint`].
pub struct PaintCtx<'a, 'b> {
    pub(crate) draw_ctx: &'a mut DrawContext<'b>,
    pub(crate) size: Size,
}

impl<'a, 'b> PaintCtx<'a, 'b> {
    /// Returns the bounds of the visual.
    pub fn bounds(&self) -> Bounds {
        Bounds::new(Point::origin(), self.size)
    }

    pub fn size(&self) -> Size {
        self.size
    }
}

impl<'a, 'b> Deref for PaintCtx<'a, 'b> {
    type Target = DrawContext<'b>;

    fn deref(&self) -> &Self::Target {
        self.draw_ctx
    }
}

impl<'a, 'b> DerefMut for PaintCtx<'a, 'b> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.draw_ctx
    }
}


/// Last known state of various input devices.
pub(crate) struct InputState {
    /// Current state of keyboard modifiers.
    mods: winit::event::ModifiersState,
    /// Current state of pointers.
    pointers: HashMap<DeviceId, PointerState>,
}

pub(crate) struct FocusState {
    /// the node that has the pointer grab.
    pointer_grab: Weak<RefCell<Node<dyn Visual>>>,
    /// the node that has the keyboard focus.
    focus: Weak<RefCell<Node<dyn Visual>>>
}

/// Context passed to [`Visual::event`] during event propagation.
/// Also serves as a return value for this function.
pub struct EventCtx<'a> {
    input_state: &'a InputState,
    focus_state: &'a mut FocusState,
    /// The bounds of the current visual.
    bounds: Bounds,
    /// A redraw has been requested.
    redraw_requested: bool,
    /// The passed event was handled.
    handled: bool,
    /// The current node has asked to get the pointer grab
    pointer_grab: bool,
}

impl<'a> EventCtx<'a> {
    pub(crate) fn new(input_state: &'a InputState, focus_state: &'a mut FocusState) -> EventCtx<'a> {
        EventCtx {
            input_state,
            focus_state,
            bounds: Bounds::default(),
            redraw_requested: false,
            handled: false,
            pointer_grab: false,
        }
    }

    fn make_child_ctx(&mut self, bounds: Bounds) -> EventCtx<'a> {
        EventCtx {
            input_state: self.input_state,
            focus_state: self.focus_state,
            bounds,
            handled: false,
            pointer_grab: false,
            redraw_requested: false,
        }
    }

    /// Returns the bounds of the current widget.
    pub fn bounds(&self) -> Bounds {
        bounds
    }

    /// Requests a redraw of the current visual.
    pub fn request_redraw(&mut self) {
        self.redraw_requested = true;
    }

    /// Requests that the current node grabs all pointer events.
    pub fn set_pointer_grab(&mut self) {
        self.pointer_grab = true;
    }

    /// Releases the pointer grab, if the current node is holding it.
    pub fn reset_pointer_grab(&mut self) {
    }

    /// Signals that the passed event was handled and should not bubble up further.
    pub fn set_handled(&mut self) {
        self.handled = true;
    }
}


/// The interface for painting a visual element on the screen, and handling events that target this
/// visual.
///
/// [`Visual`]s are typically wrapped in a [`Node`], which bundles the visual and the layout
/// information of the visual within a parent object.
pub trait Visual: Any {
    /// Draws the visual using the specified painter.
    ///
    fn paint(&mut self, ctx: &mut PaintCtx, theme: &Theme);

    /// Checks if the given point falls inside the widget.
    ///
    /// Usually it's a simple matter of checking whether the point falls in the provided bounds,
    /// but some widgets may want a more complex hit test.
    fn hit_test(&mut self, point: Point, bounds: Bounds) -> bool;

    /// Handles an event that targets this visual, and returns the _actions_ emitted in response
    /// to this event.
    fn event(&mut self, event_ctx: &mut EventCtx, event: &Event);

    /// as_any for downcasting
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

/// A visual that has no particular behavior, used for layout wrappers.
pub struct LayoutBox;

impl Visual for LayoutBox {
    fn paint(&mut self, ctx: &mut PaintCtx, theme: &Theme) {}

    fn hit_test(&mut self, _point: Point, _bounds: Bounds) -> bool {
        // TODO
        true
    }

    fn event(&mut self, event_ctx: &mut EventCtx, event: &Event) {
        // nothing to handle
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

/// A list of Rc-RefCell wrapped nodes with utilities for reconciliation.
pub struct NodeList {
    pub list: Vec<Rc<RefCell<Node<dyn Visual>>>>,
}

impl NodeList {
    pub fn new() -> NodeList {
        NodeList {
            list: Vec::new()
        }
    }

    pub fn replacer(&mut self) -> NodeReplacer {
        NodeReplacer {
            list: &mut self.list,
            index: 0,
        }
    }
}

/// A node within the visual tree.
///
/// It contains the bounds of the visual, and an instance of [`Visual`] that defines its behavior:
/// painting, hit-testing, and how it responds to events that target the visual.
///
/// ## Rationale behind using `Rc<RefCell<Node>>` instead of IDs
/// In some cases, we need to send events to a particular node in the tree. For instance, we may
/// want to send keyboard events directly to the focused node, without having to traverse the whole
/// visual tree, or send events to the node that corresponds to a particular window.
/// Because of those use cases, we need a way to reference and access (i.e. to "address") a node
/// inside the tree.
/// There seems to be three main classes of solutions to this problem:
/// - A: store nodes in a big vector or hash map, and use IDs or indices to identify them. The
///      parent-child relations are stored in another vector.
///     - to access the node, look up the index
///
/// - B: wrap nodes in `Rc<RefCell<>>`. We can hold a reference to node with `Weak<>` or `Rc<>`,
///   depending on the desired behavior.
///
/// - C: actually traverse the whole visual tree to search for the target node.
///  We want to avoid that.
///
/// At first glance, option A seems less wasteful, because all nodes are allocated in a big array
/// and people often suggest that this leads to more efficient memory access patterns.
/// It also does not need interior mutability to modify the nodes since they are not wrapped in `Rc`
/// (the ownership of nodes is well-defined).
/// However there are some drawbacks:
/// - additional book-keeping needed to update the parent-child relations
/// - indirection when wanting to access a node
/// - it's tricky to know if the node identified by an ID is still alive: the node might have been
///   deleted, and another node with the same ID / at the same index may have taken its place.
///   Data structures that use "generational indices" do not have this problem, but this piles even
///   more complexity on top, or necessitates the use of an external library.
/// In addition, there will be one dynamic allocation per node anyway, because they own polymorphic
/// [`Visuals`](Visual).
///
/// Option B has the disadvantage that it requires tracked interior mutability with `RefCell`, and
/// has the overhead of reference counting, but it has some advantages:
/// - the parent-child relations are encoded directly in the node in a familiar way
/// - nodes can be accessed directly without indirection
/// - using `Weak` references, it's trivial to check if the pointed-to node is still valid
///   (`Weak::upgrade` will fail).
/// This solution also need one dynamic allocation per node, since it's possible to have a
/// `Rc<RefCell<Node<dyn Visual>>>` through the magic of `Rc` and unsized types. So there is no
/// disadvantage in this regard.
///
/// ## Re-examination:
/// - Option D: Store references to a widget as a "dispatch chain" (like xxgui).
/// Sometimes, we want to be able to deliver an event to a particular node in the tree, but do we
/// want to bypass the parent nodes?
///
/// ## Visuals own child nodes?
/// - Advantages:
///     - less overhead
/// - Drawbacks:
///    - visuals must cooperate with everything that needs a traversal:
///        - event delivery
///        - painting
///    - can't send an event directly to the focused node
///
///
///
pub struct Node<V: ?Sized> {
    /// Parent node.
    pub parent: Weak<RefCell<Node<dyn Visual>>>,
    /// Child nodes.
    pub children: NodeList,
    /// Layout of the node relative to the containing window.
    pub layout: Layout,
    /// Calculated bounds in the containing window.
    pub bounds: Option<Bounds>,
    /// Key associated to the node.
    pub key: Option<u64>,
    /// The visual. Defines the painting, hit-testing, and event behaviors.
    /// The visual instance is set up by the [widget] during [layout](Widget::layout).
    pub visual: V,
}

pub type RcNode = Rc<RefCell<Node<dyn Visual>>>;

impl<V: Visual> Node<V> {
    /// Creates a new node from a layout and a visual.
    pub fn new(layout: Layout, key: Option<u64>, visual: V) -> Node<V> {
        Node {
            // A dummy type is specified here because Weak::new() has a Sized bound on T.
            // See discussion at https://users.rust-lang.org/t/why-cant-weak-new-be-used-with-a-trait-object/29976
            // also see issue https://github.com/rust-lang/rust/issues/50513
            // and https://github.com/rust-lang/rust/issues/60728
            parent: Weak::<RefCell<Node<LayoutBox>>>::new(),
            children: NodeList::new(),
            key,
            bounds: None,
            layout,
            visual,
        }
    }
}

impl<V: Visual + ?Sized> Node<V> {
    /// Calculate the window bounds of the node recursively.
    pub(crate) fn propagate_bounds(&mut self, origin: Point) {
        let origin = origin + self.layout.offset;
        self.bounds = Some(Bounds::new(origin, self.layout.size));
        for child in self.children.list.iter_mut() {
            child.borrow_mut().propagate_bounds(origin);
        }
    }

    /// Draws the node and its descendants using the specified theme, in the specified context.
    ///
    /// Effectively, it applies the transform of the node (which, right now, is only an offset relative to the parent),
    /// calls [`Visual::paint`] on `self.visual`, then recursively calls [`Node::paint`] on descendants.
    pub fn paint(&mut self, ctx: &mut PaintCtx, theme: &Theme) {
        let mut ctx2 = PaintCtx {
            size: self.layout.size,
            draw_ctx: ctx.draw_ctx,
        };

        let saved = ctx2.draw_ctx.save();
        ctx2.draw_ctx.transform(&self.layout.offset.to_transform());
        self.visual.paint(&mut ctx2, theme);

        for c in self.children.list.iter() {
            c.borrow_mut().paint(&mut ctx2, theme);
        }

        ctx2.draw_ctx.restore();
    }

    /// Performs hit-test of the specified [`PointerEvent`] on the node, then, if hit-test
    /// is successful, returns the [`PointerEvent`] mapped to local coordinates.
    ///
    /// [`PointerEvent`]: crate::event::PointerEvent
    fn translate_pointer_event(&self, pointer: &PointerEvent) -> Option<PointerEvent>
    {
        // don't use `PointerEvent::position` because we may receive the event directly from the
        // event loop and not from the direct parent (position can be relative to the window in
        // the former case, or to the parent node in the latter).
        let bounds = self.bounds.expect("layout");
        let hit = bounds.contains(pointer.window_position);

        if hit {
            Some(PointerEvent {
                position: pointer.window_position - bounds.origin.to_vector(),
                ..*pointer
            })
        } else {
            None
        }
    }

    /// Processes an event.
    ///
    /// This function will determine if the event is of interest for the node, then propagate it
    /// to child nodes. If the event was not handled by any child node,
    /// forward it to the [`Visual`] of the node.
    ///
    /// See also: [`Visual::event`].
    pub fn event(&mut self, ctx: &mut EventCtx, event: &Event)
    {
        let child_event = match event {
            Event::PointerUp(p) => {
                self.translate_pointer_event(p).map(Event::PointerUp)
            },
            Event::PointerDown(p) => {
                self.translate_pointer_event(p).map(Event::PointerDown)
            },
            Event::PointerMove(p) => {
                self.translate_pointer_event(p).map(Event::PointerMove)
            },
            e => Some(*e),
        };

        if let Some(child_event) = child_event {
            // distribute event to children first, so that they may capture it
            // FIXME children don't receive events if hit test on the parent fails:
            // this might not always be the behavior that we want

            for c in self.children.list.iter() {
                let mut ctx = ctx.make_child_ctx(self.bounds.unwrap());
                // FIXME this will send the event to all children, regardless of whether
                // they are visible or not: ask the visual to propagate the event?
                c.borrow_mut().event(&mut ctx, &child_event);

                if ctx.handled {

                }
            }

            if !ctx.handled {
                // event was not handled, bubble up
                self.visual.event(ctx, &child_event);
            }
        }
    }
}

impl Node<dyn Visual> {
    /// Downcasts this node to a concrete type.
    pub fn downcast_mut<V: Visual>(&mut self) -> Option<&mut Node<V>> {
        if self.visual.as_any().is::<V>() {
            // SAFETY: see <dyn Any>::downcast_mut in std
            // TODO: this may be somewhat different since it's a DST?
            unsafe { Some(&mut *(self as *mut Self as *mut Node<V>)) }
        } else {
            None
        }
    }
}


/// Handles reconciliation.
pub struct NodeReplacer<'a>
{
    list: &'a mut Vec<Rc<RefCell<Node<dyn Visual>>>>,
    index: usize,
}

impl<'a> NodeReplacer<'a> {
    ///
    pub fn new(list: &'a mut Vec<Rc<RefCell<Node<dyn Visual>>>>) -> NodeReplacer<'a> {
        NodeReplacer {
            list,
            index: 0,
        }
    }

    /// Finds a node that matches the given visual type and key, starting from the cursor to the end
    /// of the list.
    fn find_and_move_in_place(
        &mut self,
        ty: TypeId,
        key: Option<NodeKey>,
    ) -> bool
    {
        // look for matching node
        let pos = self.list[self.index..].iter().position(move |node| {
            let node = node.borrow_mut();
            node.visual.type_id() == ty && node.key == key
        });
        let cur_pos = self.index;

        // match found
        if let Some(pos) = pos {
            if pos > cur_pos {
                // match found, but further in the list: rotate in place
                self.list[cur_pos..=pos].rotate_right(1);
                //trace!("[visual tree] rotate in place");
            }
            true
        } else {
            false
        }
    }


    /// This function searches for a node that matches the provided type T and key in the list of
    /// nodes, starting from the cursor location.
    /// If a matching node is found, returns a ref-counted reference to the node, and mark as "dead"
    /// all nodes that were skipped during the search.
    ///
    /// If no matching node is found, a new one is created by invoking the provided closure, and
    /// inserted at the current cursor position. The current cursor position is then advanced to
    /// the next node.
    pub fn replace_or_create_with<V: Visual, F: FnOnce(Option<Node<V>>) -> Node<V>>(
        &mut self,
        key: Option<NodeKey>,
        f: F)
    {
        let found = self.find_and_move_in_place(TypeId::of::<V>(), key);
        let index = self.index;

        if found {
            // we know downcast won't fail because of the invariants of find_and_move_in_place if found == true
            let mut node = RefMut::map(self.list[index].borrow_mut(), |node| node.downcast_mut().unwrap());

            // sleight-of-hand to extract the node, pass it to layout(), and replace it with the node returned by
            // layout()
            replace_with::replace_with_or_abort(
                &mut *node,
                move |prev_node| {
                    f(Some(prev_node))
                }
            );
        } else {
            // not found, insert new node
            let new_node = f(None);
            self.list.insert(index, Rc::new(RefCell::new(new_node)));
        }

        self.index += 1;
    }
}

impl<'a> Drop for NodeReplacer<'a> {
    fn drop(&mut self) {
        self.list.drain(self.index..);
    }
}
