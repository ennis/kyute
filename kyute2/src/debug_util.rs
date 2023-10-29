//! Debugging utilities

use crate::{ChangeFlags, Element, EventKind, Geometry};
use kyute2::WidgetId;
use once_cell::sync::OnceCell;
use serde_json as json;
use std::{
    any::Any,
    collections::hash_map::DefaultHasher,
    fmt,
    fmt::Debug,
    hash::Hasher,
    mem, ptr,
    sync::{Mutex, MutexGuard},
    time::Duration,
};
use threadbound::ThreadBound;
use winit::window::WindowId;

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
    ptr::hash(elem, &mut hasher);
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

#[derive(Copy, Clone)]
pub struct DebugNode<'a> {
    pub name: &'a str,
    pub ty: &'a str,
    pub ptr_id: ElementPtrId,
    pub id: WidgetId,
    pub properties: &'a [Property<'a>],
    pub children: &'a [DebugNode<'a>],
}

impl<'a> DebugNode<'a> {
    pub fn property<T: Any + Copy>(&self, name: &str) -> Option<&'a T> {
        self.properties.iter().find(|p| p.name == name)?.cast()
    }

    pub fn str_property(&self, name: &str) -> Option<&'a str> {
        self.properties.iter().find(|p| p.name == name)?.as_str()
    }

    pub fn find_by_ptr(&'a self, ptr_id: ElementPtrId) -> Option<&'a DebugNode<'a>> {
        if self.ptr_id == ptr_id {
            return Some(self);
        }
        self.children.iter().find_map(|c| c.find_by_ptr(ptr_id))
    }

    pub fn find_by_id(&'a self, id: WidgetId) -> Option<&'a DebugNode<'a>> {
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
    children: Vec<DebugNode<'a>>,
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
        let node = debug_element_tree(self.arena, name, inner);
        self.children.push(node);
    }
}

pub type DebugArena = bumpalo::Bump;

pub fn debug_element_tree<'a>(arena: &'a DebugArena, name: &'a str, element: &dyn Element) -> DebugNode<'a> {
    let mut writer = DebugWriter {
        arena,
        ty: "",
        properties: vec![],
        children: vec![],
    };
    element.debug(&mut writer);
    DebugNode {
        name,
        ty: writer.ty,
        ptr_id: elem_ptr_id(element),
        id: element.id(),
        properties: arena.alloc_slice_copy(&writer.properties),
        children: arena.alloc_slice_copy(&writer.children),
    }
}

// Global debug arena
static DEBUG_ARENA_LOCK: Mutex<()> = Mutex::new(());
static mut DEBUG_ARENA: OnceCell<DebugArena> = OnceCell::new();
static DEBUG_SNAPSHOTS: OnceCell<Mutex<Vec<DebugSnapshot>>> = OnceCell::new();

unsafe fn get_debug_arena() -> &'static DebugArena {
    DEBUG_ARENA.get_or_init(|| DebugArena::new())
}

pub(crate) fn get_debug_snapshots() -> MutexGuard<'static, Vec<DebugSnapshot>> {
    let snapshots = DEBUG_SNAPSHOTS.get_or_init(|| Mutex::new(Vec::new()));
    snapshots.lock().unwrap()
}

/// Collected debug information for an event.
pub struct DebugEventData {
    /// Calculated route.
    pub route: Vec<WidgetId>,
    /// Event kind.
    pub kind: EventKind,
    /// Change flags returned by the root element.
    pub change_flags: ChangeFlags,
}

pub struct DebugSnapshot {
    /// Time since the start of event loop.
    pub time: Duration,
    /// The window for which we are recording the snapshot.
    pub window: WindowId,
    /// The root debug node of the element tree.
    pub root: DebugNode<'static>,
    /// All events that happened for this window since the last snapshot.
    pub event_data: Vec<DebugEventData>,
}

/// Records a snapshot of the element tree after propagating an event.
pub(crate) fn debug_record_ui_snapshot(
    time: Duration,
    window: WindowId,
    root: &dyn Element,
    event_data: Vec<DebugEventData>,
) {
    let _lock = DEBUG_ARENA_LOCK.lock().unwrap();
    // SAFETY: we only access the debug arena here, and it's protected by DEBUG_ARENA_LOCK.
    // Values returned by the arena have static lifetime and cannot be invalidated.
    let arena = unsafe { get_debug_arena() };
    let root = debug_element_tree(&*arena, "root", root);
    let mut snapshots = get_debug_snapshots();
    snapshots.push(DebugSnapshot {
        time,
        window,
        root,
        event_data,
    });
}

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
