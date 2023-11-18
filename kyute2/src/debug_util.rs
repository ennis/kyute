//! Debugging utilities
use std::{
    any::Any,
    collections::{hash_map::DefaultHasher, HashMap},
    fmt,
    fmt::Debug,
    hash::{Hash, Hasher},
    ptr,
    sync::{
        atomic::{AtomicBool, Ordering},
        Mutex, MutexGuard,
    },
    time::Duration,
};

use kurbo::{Affine, Rect};
use once_cell::sync::OnceCell;
use winit::window::WindowId;

use crate::{context::ElementIdTree, BoxConstraints, ChangeFlags, Element, ElementId, EventKind, Geometry};

pub trait PropertyValue: Any + Debug + Send + Sync {
    fn as_any(&self) -> &dyn Any;
}

impl<T> PropertyValue for T
where
    T: Any + Debug + Send + Sync,
{
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl dyn PropertyValue {
    pub fn cast<T>(&self) -> Option<&T>
    where
        T: Any,
    {
        self.as_any().downcast_ref::<T>()
    }
}

pub type ElementPtrId = u64;

/// Returns a unique ID based on memory address of the element.
///
/// The ID is stable as long as the address of the element stays the same.
///
/// FIXME: this may not work correctly with ZST element types.
pub fn elem_ptr_id(elem: &dyn Element) -> ElementPtrId {
    let mut hasher = DefaultHasher::new();
    // The cast to *const () is necessary because otherwise it might hash the vtable pointer
    // which is not guaranteed to be unique even for the same allocation.
    ptr::hash(elem as *const _ as *const (), &mut hasher);
    elem.type_id().hash(&mut hasher);
    hasher.finish()
}

/*
#[derive(Debug)]


#[derive(Debug)]
struct PropertyList<'a> {
    properties: Vec<Property<'a>>,
}

pub struct DebugTreeBuilder<'a> {
    properties: Vec<Property<'a>>,
}

impl<'a> DebugTreeBuilder<'a> {
    /// Adds a debug property.
    pub fn add<T>(&mut self, name: &str, property: T)
    where
        T: Debug + Any + 'a,
    {
        self.properties.push(Property {
            name: name.to_owned(),
            value: Box::new(property),
        });
    }

    pub fn with_child(&mut self, name: &str, f: impl FnOnce(&mut DebugTreeBuilder)) {
        let mut builder = DebugTreeBuilder { properties: vec![] };
        f(&mut builder);
        self.properties.push(Property {
            name: name.to_owned(),
            value: Box::new(builder.properties),
        });
    }
}*/

#[derive(Copy, Clone)]
pub enum PropertyValueKind<'a> {
    Erased(&'a dyn PropertyValue),
    Str(&'a str),
}

#[derive(Copy, Clone)]
pub struct Property<'a> {
    pub name: &'a str,
    pub value: PropertyValueKind<'a>,
}

impl<'a> Property<'a> {
    pub fn cast<T>(&self) -> Option<&'a T>
    where
        T: Any,
    {
        match self.value {
            PropertyValueKind::Erased(v) => v.cast(),
            PropertyValueKind::Str(_) => None,
        }
    }

    pub fn as_str(&self) -> Option<&'a str> {
        match self.value {
            PropertyValueKind::Erased(_) => None,
            PropertyValueKind::Str(v) => Some(v),
        }
    }
}

/// Debug information about an element in the UI tree.
///
/// This is built by the `Element::debug` method.
#[derive(Copy, Clone)]
pub struct ElementDebugNode<'a> {
    pub name: &'a str,
    pub ty: &'a str,
    pub ptr_id: ElementPtrId,
    pub id: ElementId,
    pub properties: &'a [Property<'a>],
    pub children: &'a [ElementDebugNode<'a>],
}

impl<'a> ElementDebugNode<'a> {
    pub fn property<T: Any + Copy>(&self, name: &str) -> Option<&'a T> {
        self.properties.iter().find(|p| p.name == name)?.cast()
    }

    pub fn str_property(&self, name: &str) -> Option<&'a str> {
        self.properties.iter().find(|p| p.name == name)?.as_str()
    }

    pub fn find_by_ptr(&'a self, ptr_id: ElementPtrId) -> Option<&'a ElementDebugNode<'a>> {
        if self.ptr_id == ptr_id {
            return Some(self);
        }
        self.children.iter().find_map(|c| c.find_by_ptr(ptr_id))
    }

    pub fn find_by_id(&'a self, id: ElementId) -> Option<&'a ElementDebugNode<'a>> {
        if self.id == id {
            return Some(self);
        }
        self.children.iter().find_map(|c| c.find_by_id(id))
    }
}

pub struct DebugWriter<'a> {
    arena: &'a bumpalo::Bump,
    ty: &'a str,
    properties: Vec<Property<'a>>,
    children: Vec<ElementDebugNode<'a>>,
}

impl<'a> DebugWriter<'a> {
    pub fn type_name(&mut self, ty: &'a str) {
        self.ty = ty;
    }

    pub fn str_property(&mut self, name: &'a str, value: &str) {
        let value = self.arena.alloc_str(value);
        self.properties.push(Property {
            name,
            value: PropertyValueKind::Str(value),
        });
    }

    pub fn property(&mut self, name: &'a str, value: impl Copy + Debug + Any + Send + Sync) {
        let value = self.arena.alloc(value);
        self.properties.push(Property {
            name,
            value: PropertyValueKind::Erased(value),
        });
    }

    pub fn child(&mut self, name: &'a str, inner: &dyn Element) {
        let node = dump_ui_tree_inner(self.arena, name, inner);
        self.children.push(node);
    }
}

pub type DebugArena = bumpalo::Bump;

fn dump_ui_tree_inner<'a>(arena: &'a DebugArena, name: &'a str, element: &dyn Element) -> ElementDebugNode<'a> {
    let mut writer = DebugWriter {
        arena,
        ty: "",
        properties: vec![],
        children: vec![],
    };
    element.debug(&mut writer);
    ElementDebugNode {
        name,
        ty: writer.ty,
        ptr_id: elem_ptr_id(element),
        id: element.id(),
        properties: arena.alloc_slice_copy(&writer.properties),
        children: arena.alloc_slice_copy(&writer.children),
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

// Global debug arena
static mut DEBUG_ARENA: OnceCell<DebugArena> = OnceCell::new();
static DEBUG_SNAPSHOTS: OnceCell<Mutex<Vec<DebugSnapshot>>> = OnceCell::new();
static ENABLE_COLLECTION: AtomicBool = AtomicBool::new(true);

unsafe fn get_debug_arena() -> &'static DebugArena {
    DEBUG_ARENA.get_or_init(|| DebugArena::new())
}

/// Debug information collected during painting.
#[derive(Clone, Debug)]
pub struct PaintElementDebugInfo {
    /// The element that was painted.
    pub element_ptr: ElementPtrId,
    /// Transform applied to the element (relative to the window).
    pub transform: Affine,
}

#[derive(Default, Clone, Debug)]
pub struct PaintDebugInfo {
    pub elements: HashMap<ElementPtrId, PaintElementDebugInfo>,
}

impl PaintDebugInfo {
    pub fn add(&mut self, element_info: PaintElementDebugInfo) {
        self.elements.insert(element_info.element_ptr, element_info);
    }

    pub fn get(&self, ptr_id: ElementPtrId) -> Option<&PaintElementDebugInfo> {
        self.elements.get(&ptr_id)
    }
}

/// Debug information about how an element handled an event.
#[derive(Clone, Debug)]
pub struct EventHandlingDebugInfo {
    /// The element that received the event.
    pub element_ptr: ElementPtrId,
    /// Element ID.
    pub element_id: ElementId,
    /// The event that was received.
    pub event: EventKind,
    /// Whether the event was handled by the element.
    pub handled: bool,
    pub change_flags: ChangeFlags,
}

/// Debug information collected during event propagation.
#[derive(Default, Clone, Debug)]
pub struct EventDebugInfo {
    /// Debug info for each element in the propagation path of the event.
    pub elements: Vec<EventHandlingDebugInfo>,
}

impl EventDebugInfo {
    pub fn add(&mut self, element_info: EventHandlingDebugInfo) {
        self.elements.push(element_info)
    }
    pub fn iter(&self) -> impl Iterator<Item = &EventHandlingDebugInfo> {
        self.elements.iter()
    }
}

#[derive(Clone, Debug)]
pub struct ElementLayoutDebugInfo {
    /// The element that was laid out.
    pub element_ptr: ElementPtrId,
    /// The geometry of the element.
    pub geometry: Geometry,
    /// The constraints that were used to lay out the element.
    pub constraints: BoxConstraints,
}

/// Debug information collected during the layout pass.
#[derive(Default, Clone, Debug)]
pub struct LayoutDebugInfo {
    pub elements: HashMap<ElementPtrId, ElementLayoutDebugInfo>,
}

impl LayoutDebugInfo {
    pub fn add(&mut self, element_info: ElementLayoutDebugInfo) {
        self.elements.insert(element_info.element_ptr, element_info);
    }

    pub fn get(&self, ptr_id: ElementPtrId) -> Option<&ElementLayoutDebugInfo> {
        self.elements.get(&ptr_id)
    }
}

#[derive(Copy, Clone, Debug)]
pub enum SnapshotCause {
    /// Snapshot taken after relayout.
    Relayout,
    /// Snapshot taken after event propagation.
    Event,
    /// Snapshot taken after painting.
    AfterPaint,
}

/// Debug snapshot of the state of a single window.
pub struct WindowSnapshot {
    /// ID of the window.
    pub window: WindowId,
    /// Window title
    pub window_title: String,
    /// Layout information.
    pub layout_info: LayoutDebugInfo,
    /// Layout information.
    pub paint_info: PaintDebugInfo,
    /// Event information.
    pub event_info: EventDebugInfo,
    /// The root debug node of the UI tree.
    pub root: ElementDebugNode<'static>,
    /// Focused widget
    pub focused: Option<ElementId>,
    pub pointer_grab: Option<ElementId>,
    /// Widget tree
    pub element_id_tree: ElementIdTree,
}

/// Debug snapshot of the state of the whole application.
pub struct DebugSnapshot {
    /// The cause of the snapshot.
    pub cause: SnapshotCause,
    /// Time since the start of event loop.
    pub time: Duration,
    /// Snapshots of the state of each window.
    pub window_snapshots: Vec<WindowSnapshot>,
}

/// Dumps the given UI tree to a debug tree.
pub fn dump_ui_tree(tree_root: &dyn Element) -> ElementDebugNode<'static> {
    // SAFETY: we only access the debug arena here, and it's protected by DEBUG_ARENA_LOCK.
    // Values returned by the arena have static lifetime and cannot be invalidated.
    let arena = unsafe { get_debug_arena() };
    dump_ui_tree_inner(arena, "root", tree_root)
}

pub fn enable_collection(enabled: bool) {
    ENABLE_COLLECTION.store(enabled, Ordering::Relaxed);
}

pub fn is_collection_enabled() -> bool {
    ENABLE_COLLECTION.load(Ordering::Relaxed)
}

/// Records a snapshot of the application state.
pub fn record_app_snapshot(snapshot: DebugSnapshot) {
    let mut snapshots = get_debug_snapshots();
    snapshots.push(snapshot);
}

/// Locks and returns the collection of recorded snapshots.
pub fn get_debug_snapshots() -> MutexGuard<'static, Vec<DebugSnapshot>> {
    let snapshots = DEBUG_SNAPSHOTS.get_or_init(|| Mutex::new(Vec::new()));
    snapshots.lock().unwrap()
}

////////////////////////////////////////////////////////////////////////////////////////////////////

/*
struct DebugDumpVisitor<'a> {
    indent: usize,
    output: &'a mut dyn fmt::Write,
}

impl<'a> DebugDumpVisitor<'a> {
    fn new(output: &'a mut dyn fmt::Write) -> DebugDumpVisitor<'a> {
        DebugDumpVisitor { indent: 0, output }
    }
}

impl<'a> DebugVisitor for DebugDumpVisitor<'a> {
    fn type_name(&mut self, ty: &str) {
        let pad: String = (0..self.indent).map(|_| ' ').collect();
        writeln!(self.output, "{}{}", pad, ty).unwrap();
    }

    fn property(&mut self, name: &str, value: &dyn PropertyValue) {
        let pad: String = (0..self.indent).map(|_| ' ').collect();
        writeln!(self.output, "{}{}: {:?}", pad, name, value).unwrap();
    }

    fn child(&mut self, name: &str, inner: &dyn Element) {
        let pad: String = (0..self.indent).map(|_| ' ').collect();
        writeln!(self.output, "{}{}:", pad, name).unwrap();
        self.indent += 2;
        inner.debug(self);
        self.indent -= 2;
    }
}

pub fn dump_element_tree(output: &mut dyn fmt::Write, element: &dyn Element) {
    let mut visitor = DebugDumpVisitor::new(output);
    element.debug(&mut visitor);
}

struct DebugJsonVisitor {
    value: serde_json::Value,
}

impl DebugVisitor for DebugJsonVisitor {
    fn type_name(&mut self, ty: &str) {
        self.value["type"] = json::Value::String(ty.to_owned());
    }

    fn property(&mut self, name: &str, value: &dyn PropertyValue) {
        self.value["properties"][name] = json::Value::String(format!("{:?}", value));
    }

    fn child(&mut self, name: &str, inner: &dyn Element) {
        let mut value = json::Value::Object(json::Map::new());
        mem::swap(&mut self.value, &mut value);
        inner.debug(self);
        mem::swap(&mut self.value, &mut value);
        self.value[name] = value;
    }
}

pub fn dump_element_tree_to_json(element: &dyn Element) -> json::Value {
    let mut visitor = DebugJsonVisitor {
        value: json::Value::Object(json::Map::new()),
    };
    element.debug(&mut visitor);
    visitor.value
}
*/

/*
impl DebugWidgetTreeNode {
    /// Try to extract the base widget type name (e.g. `Container` in `kyute::widgets::Container<...>`).
    pub fn base_type_name(&self) -> &str {
        let first_angle_bracket = self.name.find('<');
        let last_double_colon = if let Some(p) = first_angle_bracket {
            self.name[0..p].rfind("::").map(|p| p + 2)
        } else {
            self.name.rfind("::").map(|p| p + 2)
        };
        &self.name[last_double_colon.unwrap_or(0)..first_angle_bracket.unwrap_or(self.name.len())]
    }
}

 */

/*pub(crate) fn get_debug_widget_tree<W: Widget>(w: &W) -> DebugWidgetTreeNode {
    let mut nodes = Vec::new();
    send_utility_event(
        w,
        &mut Event::Internal(InternalEvent::DumpTree { nodes: &mut nodes }),
        &Environment::default(),
    );
    assert_eq!(nodes.len(), 1);
    nodes.into_iter().next().unwrap()
}

pub(crate) fn dump_widget_tree_rec(node: &DebugWidgetTreeNode, indent: usize, lines: &mut Vec<usize>, is_last: bool) {
    let mut pad = vec![' '; indent];
    for &p in lines.iter() {
        pad[p] = '│';
    }

    let mut msg: String = pad.into_iter().collect();
    msg += &format!("{}{}", if is_last { "└" } else { "├" }, node.base_type_name());
    if let Some(id) = node.id {
        msg += &format!("({:?})", id);
    }
    if let Some(ref content) = node.debug_node.content {
        msg += "  `";
        msg += content;
        msg += "`";
    }
    println!("{}", msg);

    if !is_last {
        lines.push(indent);
    }

    for (i, n) in node.children.iter().enumerate() {
        if i == node.children.len() - 1 {
            dump_widget_tree_rec(n, indent + 2, lines, true);
        } else {
            dump_widget_tree_rec(n, indent + 2, lines, false);
        }
    }

    if !is_last {
        lines.pop();
    }
}

pub(crate) fn dump_widget_tree<W: Widget>(w: &W) {
    let node = get_debug_widget_tree(w);
    dump_widget_tree_rec(&node, 0, &mut Vec::new(), true);
}
*/

pub struct DebugRect(pub Rect);

impl fmt::Debug for DebugRect {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "[({:.1},{:.1}) {:.1}×{:.1}]",
            self.0.origin().x,
            self.0.origin().y,
            self.0.size().width,
            self.0.size().height
        )
    }
}

pub struct DebugAffine(pub Affine);

impl fmt::Debug for DebugAffine {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let coeffs = self.0.as_coeffs();
        write!(
            f,
            "{:6.1} {:6.1} {:6.1}\n{:6.1} {:6.1} {:6.1}\n",
            coeffs[0], coeffs[2], coeffs[4], coeffs[1], coeffs[3], coeffs[5]
        )?;
        Ok(())
    }
}
