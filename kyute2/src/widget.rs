//! Widget tree manipulation and traversal.
use crate::{
    context::TreeCtx, Alignment, BoxConstraints, Element, ElementId, Event, EventCtx, Geometry, HitTestResult,
    LayoutCtx, PaintCtx, UnitExt,
};
use bitflags::bitflags;
use kurbo::Size;
use std::{
    any::{Any, TypeId},
    marker::PhantomData,
    ops::DerefMut,
};

pub mod align;
pub mod background;
pub mod button;
pub mod clickable;
pub mod constrained;
pub mod decoration;
mod flex;
pub mod frame;
pub mod grid;
pub mod null;
pub mod overlay;
pub mod padding;
mod relative;
pub mod shape;
pub mod text;

/// Widget prelude.
pub mod prelude {
    pub use crate::{
        debug_util::DebugWriter, widget::Axis, BoxConstraints, ChangeFlags, Element, ElementId, Event, EventCtx,
        EventKind, Geometry, HitTestResult, LayoutCtx, PaintCtx, Point, Rect, Size, State, TreeCtx, Widget,
    };
}

////////////////////////////////////////////////////////////////////////////////////////////////////
use crate::{context::State, debug_util::DebugWriter, drawing::Paint, widget::overlay::ZOrder, Insets, Point, Rect};

pub use align::Align;
pub use background::Background;
pub use button::button;
pub use clickable::Clickable;
pub use constrained::Constrained;
pub use decoration::{BorderStyle, DecoratedBox, RoundedRectBorder, ShapeBorder, ShapeDecoration};
pub use flex::{VBox, VBoxElement};
pub use frame::Frame;
pub use grid::{Grid, GridTemplate};
pub use null::Null;
pub use overlay::Overlay;
pub use padding::Padding;
pub use text::Text;

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

/// Widget types.
///
/// Widgets can be seen as a "diff"  against an [`Element`] in the UI tree
/// (a widget represents a change on an element).
///
/// Updates to the UI tree of a window are represented as a tree of `Widget`s.
///
/// See the crate documentation for more information.
pub trait Widget {
    /// The type of element produced by this widget.
    type Element: Element;

    /// Creates the associated element.
    ///
    /// This is called to create the element when a corresponding element cannot be found in the UI tree.
    fn build(self, cx: &mut TreeCtx, id: ElementId) -> Self::Element;

    /// Updates an existing element.
    ///
    /// This is called when a corresponding element was found in the UI tree.
    /// The function should then update the element with the latest changes, and communicate
    /// what has changed by returning [`ChangeFlags`].
    ///
    /// # Return value
    ///
    /// A set of change flags:
    /// - GEOMETRY: the geometry of the element might have changed
    /// - TODO
    fn update(self, cx: &mut TreeCtx, element: &mut Self::Element) -> ChangeFlags;
}

/// Object-safe version of the [`Widget`] trait.
///
/// The methods are modified to operate on `dyn Element` rather than a concrete `Element` type.
pub trait AnyWidget {
    /// Returns the type of element produced by this widget as a type ID.
    fn element_type_id(&self) -> TypeId;

    /// Creates the associated element.
    ///
    /// See [`Widget::build`].
    fn build(self: Box<Self>, cx: &mut TreeCtx, id: ElementId) -> Box<dyn Element>;

    /// Updates an existing element.
    ///
    /// See [`Widget::update`].
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
        // Don't forget `deref_mut()` here since `Box<dyn Element>` implements Element as well,
        // which has a different implementation of `as_any_mut()`.
        if let Some(element) = element.deref_mut().as_any_mut().downcast_mut::<T>() {
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

////////////////////////////////////////////////////////////////////////////////////////////////////

/// Axis.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum Axis {
    Horizontal,
    Vertical,
}

/// Widget state ambient value.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct WidgetState {
    pub disabled: bool,
    pub hovered: bool,
    pub active: bool,
    pub focused: bool,
}

impl Default for WidgetState {
    fn default() -> Self {
        WidgetState {
            disabled: false,
            hovered: false,
            active: false,
            focused: false,
        }
    }
}

/// Extension methods on widgets.
pub trait WidgetExt: Widget + Sized + 'static {
    /// Sets the background paint of the widget.
    #[must_use]
    fn background(self, paint: impl Into<Paint>) -> Overlay<Self, Background> {
        Overlay::new(self, Background::new(paint.into()), ZOrder::Below)
    }

    #[must_use]
    fn clickable(self) -> Clickable<Self> {
        Clickable::new(self)
    }

    #[must_use]
    fn provide_with<T, F>(self, f: F) -> ProvideWith<T, F, Self> {
        ProvideWith::new(f, self)
    }

    #[must_use]
    fn provide<T>(self, value: T) -> Provide<T, Self> {
        Provide::new(value, self)
    }

    /// Disables or enables the widget.
    #[must_use]
    fn disabled<T>(self, disabled: bool) -> ProvideWith<WidgetState, fn(WidgetState) -> WidgetState, Self> {
        // FIXME I'd like to pass a closure but I can't name the closure type, and impl Fn in return position in traits is not stable yet
        let f = if disabled {
            |prev| WidgetState { disabled: true, ..prev }
        } else {
            |prev| WidgetState {
                disabled: false,
                ..prev
            }
        };
        self.provide_with(f)
    }

    /*/// Adds a frame with decorations around the widget.
    #[must_use]
    fn decorate<B>(self, shape_decoration: ShapeDecoration<B>) -> Frame<Self, B> {
        Frame::new(100.percent(), 100.percent(), self).decoration(shape_decoration)
    }*/

    #[must_use]
    fn padding(self, padding: impl Into<Insets>) -> Padding<Self> {
        Padding::new(padding.into(), self)
    }

    #[must_use]
    fn align(self, x: Alignment, y: Alignment) -> Align<Self> {
        Align::new(x, y, self)
    }

    #[must_use]
    fn decorate<Border: ShapeBorder>(self, decoration: ShapeDecoration<Border>) -> DecoratedBox<Border, Self> {
        DecoratedBox::new(decoration, self)
    }

    #[must_use]
    fn min_size(self, size: Size) -> Constrained<Self> {
        Constrained::new(
            BoxConstraints {
                min: size,
                max: Size::new(f64::INFINITY, f64::INFINITY),
            },
            self,
        )
    }
}

impl<W: Widget + 'static> WidgetExt for W {}

////////////////////////////////////////////////////////////////////////////////////////////////////
pub struct Provide<T, W> {
    value: T,
    inner: W,
}

impl<T, W> Provide<T, W> {
    pub fn new(value: T, inner: W) -> Provide<T, W> {
        Provide { value, inner }
    }
}

impl<T, W> Widget for Provide<T, W>
where
    T: 'static,
    W: Widget,
{
    type Element = W::Element;

    fn build(self, cx: &mut TreeCtx, id: ElementId) -> Self::Element {
        cx.with_ambient(&self.value, move |cx| cx.build(self.inner))
    }

    fn update(self, cx: &mut TreeCtx, element: &mut Self::Element) -> ChangeFlags {
        cx.with_ambient(&self.value, move |cx| cx.update(self.inner, element))
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
pub struct ProvideWith<T, F, W> {
    f: F,
    inner: W,
    _phantom: PhantomData<fn() -> T>,
}

impl<T, F, W> ProvideWith<T, F, W> {
    pub fn new(f: F, inner: W) -> ProvideWith<T, F, W> {
        ProvideWith {
            f,
            inner,
            _phantom: PhantomData,
        }
    }
}

impl<T, F, W> Widget for ProvideWith<T, F, W>
where
    F: FnOnce(T) -> T,
    T: Clone + Default + 'static,
    W: Widget,
{
    type Element = W::Element;

    fn build(self, cx: &mut TreeCtx, id: ElementId) -> Self::Element {
        let prev = cx.ambient::<T>().cloned().unwrap_or_default();
        let value = (self.f)(prev);
        cx.with_ambient(&value, move |cx| cx.build(self.inner))
    }

    fn update(self, cx: &mut TreeCtx, element: &mut Self::Element) -> ChangeFlags {
        let prev = cx.ambient::<T>().cloned().unwrap_or_default();
        let value = (self.f)(prev);
        cx.with_ambient(&value, move |cx| cx.update(self.inner, element))
    }
}

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
        F: FnOnce(&mut TreeCtx, State<T>) -> W,
        W: Widget,
    {
        Stateful { init, inner }
    }
}

impl<Init, T, F, W> Widget for Stateful<Init, F>
where
    Init: FnOnce() -> T,
    F: FnOnce(&mut TreeCtx, State<T>) -> W,
    W: Widget,
    T: Any + Default,
{
    type Element = StatefulElement<T, W::Element>;

    fn build(self, cx: &mut TreeCtx, _id: ElementId) -> Self::Element {
        //eprintln!("Stateful::build");
        let mut state = (self.init)();
        let inner = cx.with_state(&mut state, move |cx, state_handle| {
            let widget = (self.inner)(cx, state_handle);
            cx.build(widget)
        });
        StatefulElement { state, inner }
    }

    fn update(self, cx: &mut TreeCtx, element: &mut Self::Element) -> ChangeFlags {
        //eprintln!("Stateful::update");
        let inner_element = &mut element.inner;
        cx.with_state(&mut element.state, move |cx, state_handle| {
            let widget = (self.inner)(cx, state_handle);
            cx.update(widget, inner_element)
        })
    }
}

impl<T: 'static, E: Element> Element for StatefulElement<T, E> {
    fn id(&self) -> ElementId {
        self.inner.id()
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, params: &BoxConstraints) -> Geometry {
        ctx.layout(&mut self.inner, params)
    }

    fn event(&mut self, ctx: &mut EventCtx, event: &mut Event) -> ChangeFlags {
        self.inner.event(ctx, event)
    }

    fn natural_width(&mut self, height: f64) -> f64 {
        self.inner.natural_width(height)
    }

    fn natural_height(&mut self, width: f64) -> f64 {
        self.inner.natural_height(width)
    }

    fn natural_baseline(&mut self, params: &BoxConstraints) -> f64 {
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
        w.child("", &self.inner);
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

impl<F, W> Widget for F
where
    F: FnOnce(&mut TreeCtx) -> W,
    W: Widget,
{
    type Element = W::Element;

    fn build(self, cx: &mut TreeCtx, _id: ElementId) -> Self::Element {
        let widget = (self)(cx);
        cx.build(widget)
    }

    fn update(self, cx: &mut TreeCtx, element: &mut Self::Element) -> ChangeFlags {
        let widget = (self)(cx);
        cx.update(widget, element)
    }
}
