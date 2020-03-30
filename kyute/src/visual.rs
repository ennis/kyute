//! Elements of the visual tree (after layout): `Visual`s and `Node`s.
use crate::event::{Event, EventCtx};
use crate::layout::{Layout, PaintLayout, Point, Offset};
use crate::renderer::Painter;
use crate::renderer::Renderer;
use crate::Bounds;
use euclid::{Point2D, UnknownUnit};
use std::any::{Any, TypeId};
use std::cell::{RefCell, RefMut};
use std::ops::Range;
use std::rc::{Rc, Weak};
use std::{mem, any};
use crate::state::NodeKey;
use log::trace;

/// Context passed to [`Visual::paint`].
pub struct PaintCtx<'a, 'b> {
    /// Calculated bounds of the visual.
    pub bounds: Bounds,
    pub painter: &'a mut Painter<'b>,
}

/// The interface for painting a visual element on the screen, and handling events that target this
/// visual.
///
/// [`Visual`]s are typically wrapped in a [`Node`], which bundles the visual and the layout
/// information of the visual within a parent object.
pub trait Visual: Any {
    /// Draws the visual using the specified painter. `layout` specifies where on the screen the
    /// visual should be drawn.
    fn paint(&mut self, ctx: &mut PaintCtx);

    /// Checks if the given point falls inside the widget.
    fn hit_test(&mut self, point: Point, layout: &PaintLayout) -> bool;

    /// Handles an event that targets this visual, and returns the _actions_ emitted in response
    /// to this event.
    fn event(&mut self, event_ctx: &EventCtx, event: &Event);

    /// as_any for downcasting
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

/*
/// Boxed visuals implementation.
impl Visual for Box<dyn Visual> {
    fn paint(&mut self, ctx: &mut PaintCtx) {
        self.as_mut().paint(ctx)
    }

    fn hit_test(&mut self, point: Point, layout: &PaintLayout) -> bool {
        self.as_mut().hit_test(point, layout)
    }

    fn event(&mut self, event_ctx: &EventCtx, event: &Event) {
        self.as_mut().event(event_ctx, event)
    }
}*/

/// A visual that has no particular behavior, used for layout wrappers.
pub struct LayoutBox;

impl Visual for LayoutBox {
    fn paint(&mut self, _ctx: &mut PaintCtx) {
        // nothing to paint
    }

    fn hit_test(&mut self, _point: Point, _layout: &PaintLayout) -> bool {
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

// needs of the visual tree
// - ordered children
// - can edit children (reconciliation?)
//      - insert range, remove range
//
// - can keep a weak ref to an element in the visual tree
// - some parts of a node should be mutable
// - ref to nodes should not be invalidated on updates
//
// identification:
// - layout_ctx: push_id, pop_id
//
// modifying the tree
// - layout_ctx.cursor().create_node(/*new*/ || {}, /* recycle */ {});
//      -> will create a new node at the position of the cursor if it's the correct type and has the expected key,
//          otherwise, will scan the tree at the current level to find a matching node type and key,
//          and remove all nodes in-between
//      -> if not found, then will *insert* a node at the current position of the cursor

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
/// There seems to be two main classes of solutions to this problem:
/// - A: store nodes in a big vector or hash map, and use IDs or indices to identify them. The
///      parent-child relations are stored in another vector.
///     - to access the node, look up the index
///
/// - B: wrap nodes in `Rc<RefCell<>>`. We can hold a reference to node with `Weak<>` or `Rc<>`,
///   depending on the desired behavior.
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

//pub type VisualNode = Node<dyn Visual>;

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

    pub(crate) fn paint(&mut self, ctx: &mut PaintCtx) {
        let mut ctx = PaintCtx {
            bounds: self.bounds.expect("layout not done"),
            painter: ctx.painter,
        };
        self.visual.paint(&mut ctx);

        for c in self.children.iter() {
            c.borrow_mut().paint(&mut ctx);
        }
    }

    pub(crate) fn event(&mut self, ctx: &mut EventCtx, event: &Event) {
        // TODO
        unimplemented!()
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

/*/// Nodes can also be directly used as Visuals: they apply their layout's offset
/// to the `PaintLayout` before calling the wrapped visual.
impl<A, V: Visual<A>> Visual<A> for Node<V> {
    fn paint(&mut self, painter: &mut Painter, layout: &PaintLayout) {
        let layout = PaintLayout::new(layout.bounds.origin, &self.layout);
        self.visual.paint(painter, &layout)
    }

    fn hit_test(&mut self, point: Point, layout: &PaintLayout) -> bool {
        let layout = PaintLayout::new(layout.bounds.origin, &self.layout);
        self.visual.hit_test(point, &layout)
    }

    fn event(&mut self, ctx: &EventCtx, event: &Event) -> Vec<A> {
        let ctx = ctx.with_layout(&self.layout);
        self.visual.event(&ctx, event)
    }
}*/

/// A cursor that points to a child node inside a parent.
pub struct Cursor<'a> {
    // TODO: ref to key stack
    list: &'a mut Vec<Rc<RefCell<Node<dyn Visual>>>>,
    index: usize,
    skip_list: Vec<Range<usize>>,
}

// control flow for containers:
// - open container node, using key and type
// - move the cursor to the beginning of the children of the opened node
// - request all children, passing the cursor
// - the list of children is now updated

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
                trace!("[visual tree: {}] found where expected", any::type_name::<V>());
            }
            if overwrite {
                // match found, but replace anyway
                self.list[cur_pos] = Rc::new(RefCell::new(Node::new(layout, key, create_fn())));
            }
        } else {
            // no match, insert new
            self.list.insert(
                cur_pos,
                Rc::new(RefCell::new(Node::new(Layout::default(), key, create_fn()))),
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
    pub fn open<V: Visual, F: FnOnce() -> V>( &mut self,
                            key: Option<NodeKey>,
                            create_fn: F) -> RefMut<Node<V>>
    {
        self.open_internal(key, Layout::default(), false, create_fn)
    }

    /// Overwrite the visual at the cursor location if it has matching types. Otherwise, inserts it
    /// and returns a mutable reference to it.
    pub fn overwrite<V: Visual>(
        &mut self,
        key: Option<NodeKey>,
        layout: Layout,
        visual: V,
    ) -> RefMut<Node<V>>
    {
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