//! built-in widgets.
mod align;
mod button;
mod clickable;
mod constrained;
//mod container;
pub mod drop_down;
mod flex;
pub mod grid;
mod image;
mod label;
mod layout_wrapper;
mod menu;
mod null;
mod padding;
mod separator;
mod slider;
mod text;
mod text_edit;
//mod text_v1;
mod border;
mod canvas;
//mod color_picker;
//mod layer_widget;
mod env_override;
mod formatter;
mod frame;
mod popup;
mod scroll_area;
//mod selectable;
mod checkbox;
mod cursor;
mod debug;
mod drag_drop;
mod drawable;
mod font_size;
pub mod form;
mod group_box;
mod overlay;
mod placeholder;
mod placement;
mod shape;
mod stateful;
mod stepper;
mod styled_box;
pub mod table;
mod text_input;
mod thumb;
mod titled_pane;
mod widget_pod;

//pub use align::Align;
//pub use baseline::Baseline;
pub use border::Border;
pub use button::Button;
pub use canvas::{Canvas, Viewport};
pub use checkbox::{Checkbox, CheckboxField};
pub use clickable::Clickable;
pub use debug::{Debug, DebugFlags, DebugName};
pub use drawable::Drawable;
//pub use color_picker::{ColorPaletteItem, ColorPicker, ColorPickerMode, ColorPickerParams, HsvColorSquare};
//pub use constrained::ConstrainedBox;
pub use drop_down::DropDown;
pub use env_override::EnvOverride;
pub use flex::{CrossAxisAlignment, Flex, MainAxisAlignment, MainAxisSize};
pub use form::Form;
pub use formatter::{DisplayFormatter, FloatingPointNumberFormatter, Formatter, ValidationResult};
pub use frame::Frame;
pub use grid::Grid;
pub use image::{Image, Scaling};
pub use label::Label;
pub use placement::Adjacent;
//pub use layer_widget::LayerWidget;
pub use layout_wrapper::LayoutInspector;
pub use menu::{Action, ContextMenu, Menu, MenuItem, Shortcut};
pub use null::Null;
pub use padding::Padding;
pub use popup::Popup;
pub use scroll_area::ScrollArea;
pub use slider::SliderBase;
pub use stepper::Stepper;
pub use styled_box::StyledBox;
pub use table::{TableSelection, TableView, TableViewParams};
pub use text::Text;
pub use text_edit::{BaseTextEdit, TextEdit, TextField};
//pub use text_input::{StepperTextInput, TextInput};
pub use drag_drop::DropTarget;
pub use group_box::GroupBox;
pub use overlay::{Overlay, ZOrder};
pub use placeholder::Placeholder;
pub use shape::Shape;
pub use stateful::{Retained, RetainedWidget};
pub use thumb::{DragController, Thumb};
pub use titled_pane::TitledPane;
pub use widget_pod::WidgetPod;

use crate::{
    composable,
    core::DebugNode,
    drawing::PaintCtx,
    layout::Alignment,
    style,
    style::Style,
    theme,
    widget::{
        align::{HorizontalAlignment, VerticalAlignment},
        constrained::{Fill, FixedHeight, FixedWidth, MaxHeight, MaxWidth, MinHeight, MinWidth},
        cursor::CursorIcon,
        font_size::FontSize,
    },
    Color, EnvKey, EnvValue, Environment, Event, EventCtx, Geometry, LayoutCtx, LayoutParams, Length,
    LengthOrPercentage, UnitExt, Widget, WidgetId,
};
use kyute_shell::{winit, TypedData};
use std::{
    convert::TryInto,
    fmt,
    ops::{Deref, DerefMut},
    sync::Arc,
};

// TODO move somewhere else
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Orientation {
    Horizontal,
    Vertical,
}

impl Orientation {
    pub fn cross_orientation(self) -> Orientation {
        match self {
            Orientation::Horizontal => Orientation::Vertical,
            Orientation::Vertical => Orientation::Horizontal,
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// Modifiers
////////////////////////////////////////////////////////////////////////////////////////////////////

/// Widget modifiers.
pub trait Modifier {
    fn layout<W: Widget>(
        &self,
        ctx: &mut LayoutCtx,
        widget: &W,
        constraints: &LayoutParams,
        env: &Environment,
    ) -> Geometry;

    fn debug_node(&self) -> DebugNode {
        DebugNode::new("Modifier")
    }
}

/// Wraps a widget with a layout modifier.
#[derive(Clone)]
pub struct Modified<M, W>(pub M, pub W);

impl<M, W> Modified<M, W> {
    pub fn inner(&self) -> &W {
        &self.1
    }

    pub fn inner_mut(&mut self) -> &mut W {
        &mut self.1
    }
}

impl<M, W> Widget for Modified<M, W>
where
    W: Widget,
    M: Modifier,
{
    fn widget_id(&self) -> Option<WidgetId> {
        self.1.widget_id()
    }

    fn layout(&self, ctx: &mut LayoutCtx, constraints: &LayoutParams, env: &Environment) -> Geometry {
        self.0.layout(ctx, &self.1, constraints, env)
    }

    fn event(&self, ctx: &mut EventCtx, event: &mut Event, env: &Environment) {
        self.1.route_event(ctx, event, env)
    }

    fn paint(&self, ctx: &mut PaintCtx) {
        self.1.paint(ctx)
    }

    fn debug_node(&self) -> DebugNode {
        self.0.debug_node()
    }
}

/*impl<W> WidgetWrapper for Modified<(), W>
where
    W: WidgetWrapper,
{
    type Inner = W::Inner;

    fn inner(&self) -> &Self::Inner {
        &self.1
    }

    fn inner_mut(&mut self) -> &mut Self::Inner {
        &mut self.1
    }
}*/

/*
impl<M, W> Deref for Modified<M, W> {
    type Target = W;

    fn deref(&self) -> &Self::Target {
        &self.1
    }
}

impl<M, W> DerefMut for Modified<M, W> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.1
    }
}*/

/// Modifier that overrides an environment value.
pub struct EnvironmentOverride<T> {
    key: EnvKey<T>,
    value: T,
}

impl<T> EnvironmentOverride<T> {
    pub fn new(key: EnvKey<T>, value: T) -> EnvironmentOverride<T> {
        EnvironmentOverride { key, value }
    }
}

impl<T: EnvValue> Modifier for EnvironmentOverride<T> {
    fn layout<W: Widget>(
        &self,
        ctx: &mut LayoutCtx,
        widget: &W,
        constraints: &LayoutParams,
        env: &Environment,
    ) -> Geometry {
        let mut env = env.clone();
        env.set(&self.key, self.value.clone());
        widget.layout(ctx, constraints, &env)
    }

    fn debug_node(&self) -> DebugNode {
        DebugNode::new(format!("override {}", self.key.name()))
    }
}

/// Modifier that applies a set of environment values on top of the current one.
pub struct EnvironmentOverlay {
    overlay: Environment,
}

impl EnvironmentOverlay {
    pub fn new(overlay: Environment) -> EnvironmentOverlay {
        EnvironmentOverlay { overlay }
    }
}

impl Modifier for EnvironmentOverlay {
    fn layout<W: Widget>(
        &self,
        ctx: &mut LayoutCtx,
        widget: &W,
        constraints: &LayoutParams,
        env: &Environment,
    ) -> Geometry {
        let mut env = env.merged(self.overlay.clone());
        widget.layout(ctx, constraints, &env)
    }

    fn debug_node(&self) -> DebugNode {
        DebugNode::new(format!("environment overlay"))
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// WidgetExt
////////////////////////////////////////////////////////////////////////////////////////////////////

pub type Above<A: Widget, B: Widget> = impl Widget;
pub type Below<A: Widget, B: Widget> = impl Widget;
pub type RightOf<A: Widget, B: Widget> = impl Widget;
pub type LeftOf<A: Widget, B: Widget> = impl Widget;

/// Extension methods on widgets.
pub trait WidgetExt: Widget + Sized + 'static {
    /// Adds a border around the widget.
    #[must_use]
    fn border(self, width: impl Into<Length>, shape: impl Into<style::Shape>) -> Border<Self> {
        let width = width.into();
        let shape = shape.into();
        Border::new(
            style::Border {
                widths: [width; 4],
                color: Default::default(),
                line_style: Default::default(),
            },
            shape.into(),
            self,
        )
    }

    /// Changes the cursor icon when the mouse hovers the inner widget.
    #[must_use]
    fn cursor_icon(self, icon: winit::window::CursorIcon) -> CursorIcon<Self> {
        CursorIcon::new(self, icon)
    }

    /// Assigns a debug name to a widget.
    #[must_use]
    fn debug_name(self, name: impl Into<String>) -> DebugName<Self> {
        DebugName::new(self, name.into())
    }

    /// Wraps this widget in a debugging wrapper.
    #[must_use]
    fn debug(self, flags: DebugFlags) -> Debug<Self> {
        Debug::new(self, flags)
    }

    /// Wraps this widget in a type that implements WidgetWrapper.
    #[must_use]
    fn wrap(self) -> Modified<(), Self> {
        Modified((), self)
    }

    /// Sets the background paint of the widget.
    #[must_use]
    fn background(self, image: impl TryInto<style::Image>) -> Overlay<Self, Shape> {
        let image = image.try_into().unwrap_or_else(|_| {
            warn!("invalid CSS image value");
            style::Image::default()
        });
        Overlay::new(self, Shape::new(style::Shape::rectangle(), image), ZOrder::Below)
    }

    /// Sets the background paint of the widget.
    #[must_use]
    fn rounded_background(self, image: impl TryInto<style::Image>, radius: impl Into<Length>) -> Overlay<Self, Shape> {
        let image = image.try_into().unwrap_or_else(|_| {
            warn!("invalid CSS image value");
            style::Image::default()
        });
        Overlay::new(
            self,
            Shape::new(
                style::Shape::RoundedRect {
                    radii: [radius.into(); 4],
                },
                image,
            ),
            ZOrder::Below,
        )
    }

    /// Sets the minimum width of the widget.
    #[must_use]
    fn frame(self, width: impl Into<LengthOrPercentage>, height: impl Into<LengthOrPercentage>) -> Frame<Self> {
        Frame::new(width.into(), height.into(), self)
    }

    #[must_use]
    fn top(self, length: impl Into<Length>) -> Modified<VerticalAlignment, Modified<Padding, Self>> {
        self.padding_top(length).vertical_alignment(Alignment::START)
    }

    #[must_use]
    fn bottom(self, length: impl Into<Length>) -> Modified<VerticalAlignment, Modified<Padding, Self>> {
        self.padding_bottom(length).vertical_alignment(Alignment::END)
    }

    #[must_use]
    fn left(self, length: impl Into<Length>) -> Modified<HorizontalAlignment, Modified<Padding, Self>> {
        self.padding_left(length).horizontal_alignment(Alignment::START)
    }

    #[must_use]
    fn right(self, length: impl Into<Length>) -> Modified<HorizontalAlignment, Modified<Padding, Self>> {
        self.padding_right(length).horizontal_alignment(Alignment::END)
    }

    #[must_use]
    fn min_width(self, min_width: impl Into<Length>) -> Modified<MinWidth, Self> {
        Modified(MinWidth(min_width.into()), self)
    }

    #[must_use]
    fn min_height(self, min_height: impl Into<Length>) -> Modified<MinHeight, Self> {
        Modified(MinHeight(min_height.into()), self)
    }

    #[must_use]
    fn max_width(self, max_width: impl Into<Length>) -> Modified<MaxWidth, Self> {
        Modified(MaxWidth(max_width.into()), self)
    }

    #[must_use]
    fn max_height(self, max_height: impl Into<Length>) -> Modified<MaxHeight, Self> {
        Modified(MaxHeight(max_height.into()), self)
    }

    #[must_use]
    fn fix_width(self, width: impl Into<Length>) -> Modified<FixedWidth, Self> {
        Modified(FixedWidth(width.into()), self)
    }

    #[must_use]
    fn fix_height(self, height: impl Into<Length>) -> Modified<FixedHeight, Self> {
        Modified(FixedHeight(height.into()), self)
    }

    /// Wraps the widget in a `ConstrainedBox` that fills the available space in the parent widget.
    #[must_use]
    fn fill(self) -> Modified<Fill, Self> {
        Modified(Fill, self)
    }

    #[must_use]
    fn horizontal_alignment(self, alignment: Alignment) -> Modified<HorizontalAlignment, Self> {
        Modified(HorizontalAlignment(alignment), self)
    }

    #[must_use]
    fn vertical_alignment(self, alignment: Alignment) -> Modified<VerticalAlignment, Self> {
        Modified(VerticalAlignment(alignment), self)
    }

    /// Places this widget to the right of the the other.
    #[must_use]
    fn right_of<B: Widget + 'static>(self, other: B, vertical_alignment: Alignment) -> RightOf<Self, B> {
        // == "align left edge of self with right edge of other"
        Adjacent::new(
            self.horizontal_alignment(Alignment::START)
                .vertical_alignment(vertical_alignment),
            other
                .horizontal_alignment(Alignment::END)
                .vertical_alignment(vertical_alignment),
        )
    }

    /// Places this widget to the left of the the other.
    #[must_use]
    fn left_of<B: Widget + 'static>(self, other: B, vertical_alignment: Alignment) -> LeftOf<Self, B> {
        // == "align right edge of self with left edge of other"
        Adjacent::new(
            self.horizontal_alignment(Alignment::END)
                .vertical_alignment(vertical_alignment),
            other
                .horizontal_alignment(Alignment::START)
                .vertical_alignment(vertical_alignment),
        )
    }

    /// Places this widget above the other (vertically).
    #[must_use]
    fn above<B: Widget + 'static>(self, other: B, horizontal_alignment: Alignment) -> Above<Self, B> {
        // == "align bottom edge of self with top edge of other"
        Adjacent::new(
            self.vertical_alignment(Alignment::END)
                .horizontal_alignment(horizontal_alignment),
            other
                .vertical_alignment(Alignment::START)
                .horizontal_alignment(horizontal_alignment),
        )
    }

    /// Places this widget below the other (vertically).
    #[must_use]
    fn below<B: Widget + 'static>(self, other: B, horizontal_alignment: Alignment) -> Below<Self, B> {
        // == "align left edge of self with right edge of other"
        Adjacent::new(
            self.vertical_alignment(Alignment::START)
                .horizontal_alignment(horizontal_alignment),
            other
                .vertical_alignment(Alignment::END)
                .horizontal_alignment(horizontal_alignment),
        )
    }

    /// Centers the widget in the available space.
    #[must_use]
    fn centered(self) -> Modified<VerticalAlignment, Modified<HorizontalAlignment, Self>> {
        self.horizontal_alignment(Alignment::CENTER)
            .vertical_alignment(Alignment::CENTER)
    }

    // Aligns the widget in the available space.
    //#[must_use]
    //fn aligned(self, alignment: Alignment) -> Align<Self> {
    //    Align::new(alignment, self)
    //}

    // Adds padding around the widget.
    #[must_use]
    fn padding_left(self, left: impl Into<Length>) -> Modified<Padding, Self> {
        Modified(Padding::new(0.dip(), 0.dip(), 0.dip(), left), self)
    }

    /// Adds padding around the widget.
    #[must_use]
    fn padding_right(self, right: impl Into<Length>) -> Modified<Padding, Self> {
        Modified(Padding::new(0.dip(), right, 0.dip(), 0.dip()), self)
    }

    /// Adds padding around the widget.
    #[must_use]
    fn padding_top(self, top: impl Into<Length>) -> Modified<Padding, Self> {
        Modified(Padding::new(top, 0.dip(), 0.dip(), 0.dip()), self)
    }

    /// Adds padding around the widget.
    #[must_use]
    fn padding_bottom(self, bottom: impl Into<Length>) -> Modified<Padding, Self> {
        Modified(Padding::new(0.dip(), 0.dip(), bottom, 0.dip()), self)
    }

    /// Adds padding around the widget.
    #[must_use]
    fn padding(self, padding: impl Into<Length>) -> Modified<Padding, Self> {
        let padding = padding.into();
        Modified(Padding::new(padding, padding, padding, padding), self)
    }

    /// Adds padding around the widget.
    #[must_use]
    fn padding_trbl(
        self,
        top: impl Into<Length>,
        right: impl Into<Length>,
        bottom: impl Into<Length>,
        left: impl Into<Length>,
    ) -> Modified<Padding, Self> {
        Modified(Padding::new(top, right, bottom, left), self)
    }

    #[must_use]
    #[composable]
    fn style(self, style: impl TryInto<Style>) -> StyledBox<Self> {
        StyledBox::new(self, style)
    }

    /// Makes this widget clickable.
    ///
    /// See `Clickable`.
    #[must_use]
    #[composable]
    fn clickable(self) -> Clickable<Self> {
        Clickable::new(self)
    }

    /// Overrides an environment value.
    #[must_use]
    fn env_override<T: EnvValue>(self, key: EnvKey<T>, value: T) -> Modified<EnvironmentOverride<T>, Self> {
        Modified(EnvironmentOverride::new(key, value), self)
    }

    /// Sets the font size.
    #[must_use]
    fn font_size(self, size: impl Into<Length>) -> Modified<FontSize, Self> {
        Modified(FontSize(size.into()), self)
    }

    /// Sets the color of the text within this widget.
    #[must_use]
    fn text_color(self, color: impl Into<Color>) -> Modified<EnvironmentOverride<Color>, Self> {
        self.env_override(theme::TEXT_COLOR, color.into())
    }

    /// Wraps this widget in a `WidgetPod`.
    ///
    /// The resulting `WidgetPod` is not backed by a surface or layer.
    #[must_use]
    #[composable]
    fn pod(self) -> WidgetPod<Self> {
        WidgetPod::new(self)
    }

    /// Wraps this widget in an `Arc<WidgetPod>`.
    ///
    /// This is typically used with a `composable(cached)` function to get a cacheable object for a widget.
    ///
    /// The resulting `WidgetPod` is not backed by a surface or layer.
    #[must_use]
    #[composable]
    fn arc_pod(self) -> Arc<WidgetPod<Self>> {
        Arc::new(WidgetPod::new(self))
    }

    /// Wraps this widget in an `Arc<WidgetPod>` and erases the widget type.
    ///
    /// The resulting `WidgetPod` is not backed by a surface or layer.
    #[must_use]
    #[composable]
    fn arc_dyn_pod(self) -> Arc<WidgetPod> {
        Arc::new(WidgetPod::new(self))
    }

    /// Applies the dark theme on the child widgets.
    #[must_use]
    #[composable]
    fn dark_theme(self) -> Modified<EnvironmentOverlay, Self> {
        Modified(EnvironmentOverlay::new(theme::dark_theme()), self)
    }

    /// Applies the light theme on the child widgets.
    #[must_use]
    #[composable]
    fn light_theme(self) -> Modified<EnvironmentOverlay, Self> {
        Modified(EnvironmentOverlay::new(theme::light_theme()), self)
    }

    /// Applies the selected builtin theme on the widgets.
    #[must_use]
    #[composable]
    fn theme(self, theme: theme::Theme) -> Modified<EnvironmentOverlay, Self> {
        match theme {
            theme::Theme::Light => Modified(EnvironmentOverlay::new(theme::light_theme()), self),
            theme::Theme::Dark => Modified(EnvironmentOverlay::new(theme::dark_theme()), self),
        }
    }

    #[must_use]
    #[composable]
    fn on_drop<F>(self, f: F) -> DropTarget<Self>
    where
        F: FnOnce(&TypedData),
    {
        DropTarget::new(self).on_drop(f)
    }
}

impl<W: Widget + 'static> WidgetExt for W {}

/// Prelude containing commonly used items for writing custom widgets.
pub mod prelude {
    pub use crate::{
        cache::Signal,
        composable,
        drawing::PaintCtx,
        widget::{WidgetExt, WidgetPod},
        Alignment, BoxConstraints, DebugNode, Environment, Event, EventCtx, Geometry, LayoutCache, LayoutCtx,
        LayoutParams, Length, Measurements, Offset, Orientation, Point, Rect, Size, Transform, UnitExt, Widget,
        WidgetId,
    };
}
