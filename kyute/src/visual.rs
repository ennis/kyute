//! Elements of the visual tree (after layout): `Visual`s and `Node`s.
use crate::event::{Event, EventCtx, PointerEvent};
use crate::layout::{Layout, Offset, PaintLayout, Point};
use crate::renderer::Theme;
use crate::state::NodeKey;
use crate::Bounds;
use euclid::{Point2D, UnknownUnit};
use log::trace;
use std::any::{Any, TypeId};
use std::cell::{RefCell, RefMut};
use std::ops::{Deref, Range};
use std::rc::{Rc, Weak};
use std::any;

use kyute_shell::drawing::{Size, Transform};
use kyute_shell::window::DrawContext;
use winapi::_core::ops::DerefMut;

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
    fn event(&mut self, event_ctx: &EventCtx, event: &Event);

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

    fn event(&mut self, event_ctx: &EventCtx, event: &Event) {
        // nothing to handle
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
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
pub struct Node<V: ?Sized> {
    /// Parent node.
    pub parent: Weak<RefCell<Node<dyn Visual>>>,
    /// Child nodes.
    pub children: Vec<Rc<RefCell<Node<dyn Visual>>>>,
    /// Layout of the node relative to the containing window.
    pub layout: Layout,
    /// Calculated bounds in the containing window.
    pub bounds: Option<Bounds>,
    /// Key associated to the node.
    pub key: Option<u64>,
    /// The visual. Defines the painting, hit-testing, and event behaviors.
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
            children: Vec::new(),
            key,
            bounds: None,
            layout,
            visual,
        }
    }
}

impl<V: Visual + ?Sized> Node<V> {
    /// Returns the first child node, if it exists.
    pub fn first_child(&mut self) -> Option<RefMut<Node<dyn Visual>>> {
        self.children.first()?.borrow_mut().into()
    }

    /// Returns an editing cursor at the first child node.
    pub fn cursor(&mut self) -> Cursor {
        // problem: we can have V: !Sized (e.g. dyn Visual), but need to be able to cast to (dyn Visual)
        Cursor::new(self)
    }

    /// Calculate the window bounds of the node recursively.
    pub(crate) fn propagate_bounds(&mut self, origin: Point) {
        let origin = origin + self.layout.offset;
        self.bounds = Some(Bounds::new(origin, self.layout.size));
        trace!("node bounds: {}", self.bounds.unwrap());
        for child in self.children.iter_mut() {
            child.borrow_mut().propagate_bounds(origin);
        }
    }

    pub fn paint(&mut self, ctx: &mut PaintCtx, theme: &Theme) {
        let mut ctx2 = PaintCtx {
            size: self.layout.size,
            draw_ctx: ctx.draw_ctx,
        };

        let saved = ctx2.draw_ctx.save();
        ctx2.draw_ctx.transform(&self.layout.offset.to_transform());
        self.visual.paint(&mut ctx2, theme);

        for c in self.children.iter() {
            c.borrow_mut().paint(&mut ctx2, theme);
        }

        ctx2.draw_ctx.restore();
    }

    fn translate_pointer_event(&self, pointer: &PointerEvent) -> Option<PointerEvent>
    {
        // don't use `PointerEvent::position` because we may receive the event directly from the
        // event loop and not from the direct parent. (so position can be relative to the window in
        // the former case, or to the parent node in the latter).
        let bounds = self.bounds.expect("layout");
        let hit = bounds.contains(pointer.window_position);

        trace!("hit test point={} bounds={}", pointer.window_position, bounds);

        if hit {
            trace!("hit test node {}", bounds);
            Some(PointerEvent {
                position: pointer.window_position - bounds.origin.to_vector(),
                ..*pointer
            })
        } else {
            None
        }
    }

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
                // TODO pointer grab
                self.translate_pointer_event(p).map(Event::PointerMove)
            },
            e => Some(*e),
        };

        if let Some(child_event) = child_event {
            self.visual.event(ctx, &child_event);
            // distribute event to children
            // FIXME children don't receive events if hit test on the parent fails:
            // this might not always be the behavior that we want
            for c in self.children.iter() {
                c.borrow_mut().event(ctx, &child_event);
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


/// A cursor that points to a child node inside a parent.
pub struct Cursor<'a> {
    // TODO: ref to key stack
    list: &'a mut Vec<Rc<RefCell<Node<dyn Visual>>>>,
    index: usize,
    skip_list: Vec<Range<usize>>,
}


impl<'a> Cursor<'a> {
    ///
    pub fn new<V: Visual + ?Sized>(node: &'a mut Node<V>) -> Cursor<'a> {
        Cursor {
            list: &mut node.children,
            index: 0,
            skip_list: Vec::new(),
        }
    }

    pub fn get(&self) -> Option<&Rc<RefCell<Node<dyn Visual>>>> {
        self.list.get(self.index)
    }

    /// Finds a node that matches the given visual type and key, starting from the cursor to the end
    /// of the list.
    fn find_matching_node_position<V: Visual>(&self, key: Option<NodeKey>) -> Option<usize> {
        self.list[self.index..].iter().position(move |node| {
            let node = node.borrow_mut();
            node.visual.type_id() == TypeId::of::<V>() && node.key == key
        })
    }

    fn open_internal<V: Visual, F: FnOnce() -> V>(
        &mut self,
        key: Option<NodeKey>,
        layout: Layout,
        overwrite: bool,
        create_fn: F,
    ) -> RefMut<Node<V>> {
        // look for matching node
        let pos = self.find_matching_node_position::<V>(key);
        let cur_pos = self.index;

        // match found
        if let Some(pos) = pos {
            if pos > cur_pos {
                // match found, but further in the list: rotate in place
                self.list[cur_pos..=pos].rotate_right(1);
                trace!("[visual tree: {}] rotate in place", any::type_name::<V>());
            } else {
                trace!(
                    "[visual tree: {}] found where expected",
                    any::type_name::<V>()
                );
            }
            if overwrite {
                // match found, but replace anyway
                self.list[cur_pos] = Rc::new(RefCell::new(Node::new(layout, key, create_fn())));
            }
        } else {
            // no match, insert new
            self.list.insert(
                cur_pos,
                Rc::new(RefCell::new(Node::new(layout, key, create_fn()))),
            );
            trace!("[visual tree: {}] create new", any::type_name::<V>());
        }

        RefMut::map(self.next().unwrap(), |node| node.downcast_mut().unwrap())
    }

    /// Inserts a node into the tree or retrieves the node at the current position.
    ///
    /// This function searches for a node that matches the provided type T and key in the list of
    /// nodes, starting from the cursor location.
    /// If a matching node is found, returns a ref-counted reference to the node, and mark as "dead"
    /// all nodes that were skipped during the search.
    ///
    /// If no matching node is found, a new one is created by invoking the provided closure, and
    /// inserted at the current cursor position. The current cursor position is then advanced to
    /// the next node.
    pub fn open<V: Visual, F: FnOnce() -> V>(
        &mut self,
        key: Option<NodeKey>,
        create_fn: F,
    ) -> RefMut<Node<V>> {
        self.open_internal(key, Layout::default(), false, create_fn)
    }

    /// Overwrite the visual at the cursor location if it has matching types. Otherwise, inserts it
    /// and returns a mutable reference to it.
    pub fn overwrite<V: Visual>(
        &mut self,
        key: Option<NodeKey>,
        layout: Layout,
        visual: V,
    ) -> RefMut<Node<V>> {
        self.open_internal(key, layout, true, move || visual)
    }

    /// Returns the next node and advances the cursor, or returns None if it has reached the end of
    /// the list.
    pub fn next(&mut self) -> Option<RefMut<Node<dyn Visual>>> {
        let node = self.list.get(self.index);
        if node.is_some() {
            self.index += 1;
        }
        node.map(|node| node.borrow_mut())
    }

    /// Removes all the nodes marked as dead, as well as all nodes that come after the cursor.
    fn sweep_dead(mut self) {
        self.list.drain(self.index..);
    }
}

impl<'a> Drop for Cursor<'a> {
    fn drop(&mut self) {
        self.list.drain(self.index..);
    }
}
