//! Widget tree manipulation and traversal.
use crate::{
    context::TreeCtx, environment::Environment, Element, ElementId, Event, EventCtx, Geometry, HitTestResult,
    LayoutCtx, LayoutParams, PaintCtx,
};
use bitflags::bitflags;
use std::any::{Any, TypeId};

mod align;
mod background;
pub mod button;
pub mod clickable;
mod flex;
pub mod frame;
pub mod grid;
pub mod null;
pub mod overlay;
mod relative;
pub mod shape;
pub mod text;

use crate::composable;
use kurbo::{Point, Rect};

/// Widget prelude.
pub mod prelude {
    pub use crate::{
        composable, debug_util::DebugWriter, widget::Axis, ChangeFlags, Element, ElementId, Environment, Event,
        EventCtx, EventKind, Geometry, HitTestResult, LayoutCtx, LayoutParams, PaintCtx, Point, Rect, RouteEventCtx,
        Signal, Size, State, TreeCtx, Widget,
    };
}

////////////////////////////////////////////////////////////////////////////////////////////////////
use crate::{drawing::Paint, widget::overlay::ZOrder};

use crate::debug_util::DebugWriter;
pub use background::Background;
pub use clickable::Clickable;
pub use flex::{VBox, VBoxElement};
pub use frame::Frame;
pub use grid::{Grid, GridTemplate};
pub use null::Null;
pub use overlay::Overlay;
pub use shape::Shape;
pub use text::Text;
/*struct TreeNode {
    /// Parent node.
    parent: Option<WidgetId>,
    /// Child widgets of this node (direct descendants).
    children: Vec<WidgetId>,
}

pub(crate) struct Tree {
    pub(crate) nodes: SlotMap<WidgetId, TreeNode>,
}

impl Tree {
    pub(crate) fn new() -> (Tree, WidgetId) {
        let mut nodes = SlotMap::with_key();
        let root = nodes.insert(TreeNode {
            parent: None,
            children: vec![],
        });
        (Tree { nodes }, root)
    }
}*/

////////////////////////////////////////////////////////////////////////////////////////////////////

bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct ChangeFlags: u32 {
        const NONE = 0;
        /// Any structural change (child added / removed).
        const STRUCTURE = (1 << 0);
        /// The size of the element has changed.
        const SIZE = (1 << 1);
        /// The positioning information of the element has changed (alignment).
        /// TODO remove this, layout doesn't contain positioning information anymore
        #[deprecated]
        const POSITIONING = (1<<2);
        /// Geometry has changed (SIZE | POSITIONING)
        const GEOMETRY = Self::SIZE.bits() | Self::POSITIONING.bits();
        /// Element must be repainted.
        const PAINT = (1<<3);

        /// Child geometry may have changed.
        const CHILD_GEOMETRY = (1<<4);

        /// (Layout) constraints have changed.
        const LAYOUT_CONSTRAINTS = (1<<5);
        /// (Layout) child positions within the parent may have changed.
        const LAYOUT_CHILD_POSITIONS = (1<<7);

        /// The baseline of the element has changed.
        const BASELINE_CHANGED = (1<<8);

        // FIXME: the POSITIONING and SIZE flags are useless since if any of these changes we must call `layout`
        // on the child anyway to retrieve the latest size or alignment.
        // Technically, we could optimize the case where the child knows that only the positioning info has changed and not
        // its size, so that the parent can

        const ALL = 0xFFFF;
    }
}

/// New widget trait
pub trait Widget {
    type Element: Element;

    // Returns this widget's ID, if it has one.
    //fn id(&self) -> WidgetId;

    /// Creates the associated widget node.
    fn build(self, cx: &mut TreeCtx, id: ElementId) -> Self::Element;

    /// Updates an existing widget node.
    ///
    /// # Return value
    ///
    /// A set of change flags:
    /// - GEOMETRY: the geometry of the element might have changed
    fn update(self, cx: &mut TreeCtx, element: &mut Self::Element) -> ChangeFlags;
}

/// Type-erased widget.
pub trait AnyWidget {
    /// Returns the produced element type ID.
    fn element_type_id(&self) -> TypeId;

    /// Creates the associated widget node.
    fn build(self: Box<Self>, cx: &mut TreeCtx, id: ElementId) -> Box<dyn Element>;

    /// Updates an existing widget node.
    fn update(self: Box<Self>, cx: &mut TreeCtx, element: &mut Box<dyn Element>) -> ChangeFlags;
}

impl<W, T> AnyWidget for W
where
    W: Widget<Element = T>,
    T: Element,
{
    fn element_type_id(&self) -> TypeId {
        TypeId::of::<T>()
    }

    fn build(self: Box<Self>, cx: &mut TreeCtx, element_id: ElementId) -> Box<dyn Element> {
        Box::new(Widget::build(*self, cx, element_id))
    }

    fn update(self: Box<Self>, cx: &mut TreeCtx, element: &mut Box<dyn Element>) -> ChangeFlags {
        if let Some(element) = element.as_any_mut().downcast_mut::<T>() {
            cx.update(*self, element)
        } else {
            // not the same type, discard and rebuild
            // FIXME ID change?
            *element = Box::new(cx.build(*self));
            ChangeFlags::STRUCTURE
        }
    }
}

impl Widget for Box<dyn AnyWidget> {
    type Element = Box<dyn Element>;

    fn build(self, cx: &mut TreeCtx, element_id: ElementId) -> Self::Element {
        AnyWidget::build(self, cx, element_id)
    }

    fn update(self, cx: &mut TreeCtx, element: &mut Self::Element) -> ChangeFlags {
        AnyWidget::update(self, cx, element)
    }
}

/*/// Container for a widget.
pub struct WidgetNode<T> {
    content: T,
    id: WidgetId,
}

impl<T> WidgetNode<T> {
    /// Creates a new WidgetNode.
    #[composable]
    pub fn new(content: T) -> WidgetNode<T> {
        WidgetNode {
            content,
            id: WidgetId::here(),
        }
    }
}

// Would like to be T: ?Sized, but this would make Self::Element: ?Sized, and build() wouldn't work
impl<T: Widget> Widget for WidgetNode<T> {
    type Element = ElementNode<T::Element>;

    fn build(self, cx: &mut TreeCtx, env: &Environment) -> Self::Element {
        ElementNode::new(self.content.build(cx, env))
    }

    fn update(self, cx: &mut TreeCtx, element: &mut Self::Element, env: &Environment) -> ChangeFlags {
        element.update(cx, self, env)
    }
}
*/

////////////////////////////////////////////////////////////////////////////////////////////////////

/*
pub trait HasLayoutProperties<T> {
    type Widget: Widget;

    fn into_widget(self) -> (Self::Widget, T);
}

pub struct Attached<W, T> {
    pub widget: W,
    pub props: T,
}

impl<W, T> HasLayoutProperties<T> for Attached<W, T>
where
    W: Widget,
{
    type Widget = W;

    fn into_widget(self) -> (Self::Widget, T) {
        (self.widget, self.props)
    }
}

impl<W, T> HasLayoutProperties<T> for W
where
    W: Widget,
    T: Default,
{
    type Widget = W;

    fn into_widget(self) -> (Self::Widget, T) {
        (self, T::default())
    }
}
*/

/// Axis.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum Axis {
    Horizontal,
    Vertical,
}

/// Extension methods on widgets.
pub trait WidgetExt: Widget + Sized + 'static {
    /// Sets the background paint of the widget.
    #[must_use]
    #[composable]
    fn background(self, paint: impl Into<Paint>) -> Overlay<Self, Background> {
        Overlay::new(self, Background::new(paint.into()), ZOrder::Below)
    }

    #[must_use]
    #[composable]
    fn clickable(self) -> Clickable<Self> {
        Clickable::new(self)
    }
}

impl<W: Widget + 'static> WidgetExt for W {}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct StatefulElement<T, E> {
    state: T,
    inner: E,
}

pub struct Stateful<Init, F> {
    init: Init,
    inner: F,
}

impl<Init, F> Stateful<Init, F> {
    pub fn new<T, W>(init: Init, inner: F) -> Stateful<Init, F>
    where
        Init: FnOnce() -> T,
        F: FnOnce(&mut TreeCtx) -> W,
        W: Widget,
    {
        Stateful { init, inner }
    }
}

impl<Init, T, F, W> Widget for Stateful<Init, F>
where
    Init: FnOnce() -> T,
    F: FnOnce(&mut TreeCtx) -> W,
    W: Widget,
    T: Any + Default,
{
    type Element = StatefulElement<T, W::Element>;

    fn build(self, cx: &mut TreeCtx, _id: ElementId) -> Self::Element {
        eprintln!("Stateful::build");
        let mut state = (self.init)();
        let inner = cx.with_state(&mut state, self.inner);
        let inner = cx.build(inner);
        StatefulElement { state, inner }
    }

    fn update(self, cx: &mut TreeCtx, element: &mut Self::Element) -> ChangeFlags {
        eprintln!("Stateful::update");
        let inner = cx.with_state(&mut element.state, self.inner);
        cx.update(inner, &mut element.inner)
    }
}

impl<T: 'static, E: Element> Element for StatefulElement<T, E> {
    fn id(&self) -> ElementId {
        self.inner.id()
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, params: &LayoutParams) -> Geometry {
        self.inner.layout(ctx, params)
    }

    fn event(&mut self, ctx: &mut EventCtx, event: &mut Event) -> ChangeFlags {
        self.inner.event(ctx, event)
    }

    fn natural_size(&mut self, axis: Axis, params: &LayoutParams) -> f64 {
        self.inner.natural_size(axis, params)
    }

    fn natural_baseline(&mut self, params: &LayoutParams) -> f64 {
        self.inner.natural_baseline(params)
    }

    fn hit_test(&self, ctx: &mut HitTestResult, position: Point) -> bool {
        self.inner.hit_test(ctx, position)
    }

    fn paint(&mut self, ctx: &mut PaintCtx) {
        self.inner.paint(ctx)
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn debug(&self, w: &mut DebugWriter) {
        w.type_name("StatefulElement");
        w.child("inner", &self.inner);
    }
}
