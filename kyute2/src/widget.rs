//! Widget tree manipulation and traversal.
mod align;
mod button;
pub mod clickable;
pub mod constrained;
pub mod decoration;
pub mod frame;
pub mod null;
mod padding;
mod stateful;
pub mod text;
mod transform;

use std::{
    any::{Any, TypeId},
    marker::PhantomData,
    ops::DerefMut,
    time::Duration,
};

use bitflags::bitflags;
use kurbo::Affine;

pub use button::button;
pub use clickable::Clickable;
pub use constrained::Constrained;
pub use decoration::{BorderStyle, Decoration, RoundedRectBorder, ShapeBorder, ShapeDecoration};
pub use frame::Frame;
pub use null::Null;
pub use padding::Padding;
pub use text::Text;
pub use transform::TransformNode;

/*pub use align::Align;
pub use background::Background;
pub use button::button;
pub use clickable::Clickable;
pub use constrained::Constrained;*/
//pub use flex::{Flex, FlexElement};

/*
pub use grid::{Grid, GridTemplate};
pub use null::Null;
pub use overlay::Overlay;
pub use padding::Padding;
pub use text::Text;*/

use crate::{
    context::TreeCtx, Alignment, BoxConstraints, Event, Geometry, HitTestResult, LayoutCtx, PaintCtx, WidgetId,
};

use crate::utils::{WidgetSet, WidgetSlice};
use crate::{
    context::ContextDataHandle,
    //debug_util::DebugWriter,
    drawing::Paint,
    Insets,
    Point,
};

/*pub mod align;
pub mod background;
pub mod button;
pub mod clickable;
pub mod constrained;
pub mod decoration;
//mod flex;
pub mod frame;
pub mod grid;
pub mod null;
pub mod overlay;
pub mod padding;
mod relative;
pub mod shape;
pub mod text;*/

/// Widget prelude.
pub mod prelude {
    pub use crate::{
        widget::Axis, BoxConstraints, ChangeFlags, ContextDataHandle, Event, EventKind, Geometry, HitTestResult,
        LayoutCtx, PaintCtx, Point, Rect, Size, TreeCtx, Widget, WidgetId,
    };
}

////////////////////////////////////////////////////////////////////////////////////////////////////

bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct ChangeFlags: u32 {
        const NONE = 0;
        /// Any structural change (child added / removed).
        const STRUCTURE = (1 << 0);
        /// The geometry of the element has changed.
        const GEOMETRY = (1 << 1);
        // Geometry has changed (SIZE | POSITIONING)
        //const GEOMETRY = Self::SIZE.bits() | Self::POSITIONING.bits();
        /// Element must be repainted.
        const PAINT = (1<<3);
        /// The app logic may need to be re-run.
        const APP_LOGIC = (1<<4);

        /// Child geometry may have changed.
        const CHILD_GEOMETRY = (1<<5);
        /// (Layout) constraints have changed.
        const LAYOUT_CONSTRAINTS = (1<<6);
        /// (Layout) child positions within the parent may have changed.
        const LAYOUT_CHILD_POSITIONS = (1<<7);
        /// The baseline of the element has changed.
        const BASELINE_CHANGED = (1<<8);

        const LAYOUT_FLAGS = Self::CHILD_GEOMETRY.bits()
            | Self::LAYOUT_CONSTRAINTS.bits()
            | Self::LAYOUT_CHILD_POSITIONS.bits()
            | Self::BASELINE_CHANGED.bits();

        const ALL = 0xFFFF;
    }
}

//pub type WidgetPaths = WidgetSet<WidgetId>;
//pub type WidgetPathsRef<'a> = PathSubset<'a, WidgetId>;
pub type WidgetVisitor<'a> = dyn FnMut(&mut TreeCtx, &mut dyn Widget) + 'a;

/// Widget types.
///
/// See the crate documentation for more information.
pub trait Widget {
    /// Return this widget's ID.
    fn id(&self) -> WidgetId;

    /// Visits a child widget.
    ///
    /// Container widgets should reimplement this.
    fn visit_child(&mut self, cx: &mut TreeCtx, id: WidgetId, visitor: &mut WidgetVisitor) {}

    /// Updates this widget.
    ///
    /// This is called when something has determined that this widget needs to update itself somehow.
    /// This can be because:
    /// - a state dependency of this widget has changed
    /// - the widget was just inserted into the widget tree
    ///
    /// # Guidelines
    ///
    /// You shouldn't have to manually call `update()` on child widgets. Instead, request an update by
    /// calling `cx.request_update(widgetpaths)`.
    fn update(&mut self, cx: &mut TreeCtx) -> ChangeFlags;

    /// Event handling.
    fn event(&mut self, cx: &mut TreeCtx, event: &mut Event) -> ChangeFlags;

    /// Hit-testing.
    fn hit_test(&self, result: &mut HitTestResult, position: Point) -> bool;

    /// Layout.
    fn layout(&mut self, cx: &mut LayoutCtx, bc: &BoxConstraints) -> Geometry;

    /// Raw window events.
    fn window_event(&mut self, _cx: &mut TreeCtx, _event: &winit::event::WindowEvent, _time: Duration) -> ChangeFlags {
        ChangeFlags::NONE
    }

    fn paint(&mut self, cx: &mut PaintCtx);
}

/////////////////////////////////////////////////////////////////////////
// Dummy widget to flesh out the Widget trait
pub struct Container {
    widgets: Vec<Box<dyn Widget>>,
}

impl Widget for Container {
    fn id(&self) -> WidgetId {
        todo!()
    }

    fn visit_child(&mut self, cx: &mut TreeCtx, id: WidgetId, visitor: &mut WidgetVisitor) {
        if let Some(widget) = self.widgets.iter_mut().find(|w| w.id() == id) {
            visitor(cx, widget.deref_mut());
        }
    }

    fn update(&mut self, cx: &mut TreeCtx) -> ChangeFlags {
        todo!()
    }

    fn event(&mut self, cx: &mut TreeCtx, event: &mut Event) -> ChangeFlags {
        todo!()
    }

    fn hit_test(&self, result: &mut HitTestResult, position: Point) -> bool {
        todo!()
    }

    fn layout(&mut self, cx: &mut LayoutCtx, bc: &BoxConstraints) -> Geometry {
        todo!()
    }

    fn paint(&mut self, cx: &mut PaintCtx) {
        todo!()
    }
}

/*
/// Widget wrappers.
///
/// This is a short-hand for widgets that wrap only one another widget (like layout or appearance modifiers).
/// It has the same methods as the `Widget` trait, but most of them have a default implementation that delegates to the wrapped widget.
pub trait WidgetWrapper {
    type Content: Widget;
    fn id(&self) -> WidgetID;
    fn content(&self) -> &Self::Content;
    fn content_mut(&mut self) -> &mut Self::Content;

    fn update(&mut self, cx: &mut TreeCtx) -> ChangeFlags {
        self.content_mut().update(cx)
    }

    fn event(&mut self, cx: &mut TreeCtx, event: &mut Event) -> ChangeFlags {
        self.content_mut().event(cx, event)
    }

    fn layout(&mut self, cx: &mut LayoutCtx, bc: &BoxConstraints) -> Geometry {
        self.content_mut().layout(cx, bc)
    }
}

impl<T: WidgetWrapper> Widget for T {
    fn id(&self) -> WidgetID {
        WidgetWrapper::id(self)
    }

    fn update(&mut self, cx: &mut TreeCtx) -> ChangeFlags {
        WidgetWrapper::update(self, cx)
    }

    fn event(&mut self, cx: &mut TreeCtx, event: &mut Event) -> ChangeFlags {
        WidgetWrapper::event(self, cx, event)
    }

    fn layout(&mut self, cx: &mut LayoutCtx, bc: &BoxConstraints) -> Geometry {
        WidgetWrapper::layout(self, cx, bc)
    }
}

impl<T: Widget + ?Sized> Widget for Box<T> {
    fn id(&self) -> WidgetID {
        Widget::id(&**self)
    }

    fn update(&mut self, cx: &mut TreeCtx) -> ChangeFlags {
        Widget::update(&mut **self, cx)
    }

    fn event(&mut self, cx: &mut TreeCtx, event: &mut Event) -> ChangeFlags {
        Widget::event(&mut **self, cx, event)
    }

    fn layout(&mut self, cx: &mut LayoutCtx, bc: &BoxConstraints) -> Geometry {
        Widget::layout(&mut **self, cx, bc)
    }
}*/

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
    /*/// Sets the background paint of the widget.
    #[must_use]
    fn background(self, paint: impl Into<Paint>) -> Overlay<Self, Background> {
        Overlay::new(self, Background::new(paint.into()), ZOrder::Below)
    }

    /// Shows an overlay on top of the widget.
    #[must_use]
    fn overlay<W: Widget + 'static>(self, overlay: W) -> Overlay<Self, W> {
        Overlay::new(self, overlay, ZOrder::Above)
    }*/

    /// Makes the widget clickable.
    ///
    /// # Example
    ///
    /// TODO
    #[must_use]
    fn clickable(self) -> Clickable<Self> {
        Clickable::new(self)
    }

    /*#[must_use]
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
    fn decorate<D: Decoration>(self, decoration: D) -> DecoratedBox<D, Self> {
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
    }*/
}

impl<W: Widget + 'static> WidgetExt for W {}

////////////////////////////////////////////////////////////////////////////////////////////////////
/*pub struct Provide<T, W> {
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
    fn update(&mut self, cx: &mut TreeCtx) -> ChangeFlags {
        cx.with_state(&mut self.value, |cx, _state_handle| self.inner.update(cx))
    }

    fn event(&mut self, cx: &mut TreeCtx, event: &mut Event) -> ChangeFlags {
        cx.with_state(&mut self.value, |cx, _state_handle| self.inner.event(cx, event))
    }
}*/

/*
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

    fn build(self, cx: &mut TreeCtx, _id: ElementId) -> Self::Element {
        let prev = cx.ambient::<T>().cloned().unwrap_or_default();
        let value = (self.f)(prev);
        cx.with_ambient(&value, move |cx| cx.build(self.inner))
    }

    fn update(self, cx: &mut TreeCtx, element: &mut Self::Element) -> ChangeFlags {
        let prev = cx.ambient::<T>().cloned().unwrap_or_default();
        let value = (self.f)(prev);
        cx.with_ambient(&value, move |cx| cx.update(self.inner, element))
    }
}*/

////////////////////////////////////////////////////////////////////////////////////////////////////

/*pub struct StatefulElement<T, E> {
    state: T,
    inner: E,
}*/

/*
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

(|cx| {
    if cx.get(Disabled) {
        Text::new("Disabled")
    } else {
        Text::new("Enabled")
    }
}).into()

// the closure by itself isn't a widget, but it can be turned into one

*/
