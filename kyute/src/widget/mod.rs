//! built-in widgets.
mod align;
mod baseline;
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
mod font_size;
mod overlay;
mod shape;
mod stepper;
mod styled_box;
mod table;
mod text_input;
mod thumb;
mod titled_pane;
mod widget_pod;

use std::convert::TryInto;
//pub use align::Align;
//pub use baseline::Baseline;
pub use border::Border;
pub use button::Button;
pub use canvas::{Canvas, Viewport};
pub use clickable::Clickable;
//pub use color_picker::{ColorPaletteItem, ColorPicker, ColorPickerMode, ColorPickerParams, HsvColorSquare};
//pub use constrained::ConstrainedBox;
pub use drop_down::DropDown;
pub use env_override::EnvOverride;
pub use flex::{CrossAxisAlignment, Flex, MainAxisAlignment, MainAxisSize};
pub use formatter::{DisplayFormatter, FloatingPointNumberFormatter, Formatter, ValidationResult};
pub use frame::Frame;
pub use grid::Grid;
pub use image::{Image, Scaling};
pub use kyute_macros::WidgetWrapper;
pub use label::Label;
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
pub use table::{ColumnHeaders, TableRow, TableSelection, TableView, TableViewParams};
pub use text::Text;
pub use text_edit::BaseTextEdit;
//pub use text_input::{StepperTextInput, TextInput};
pub use overlay::{Overlay, ZOrder};
pub use shape::Shape;
pub use thumb::{DragController, Thumb};
pub use titled_pane::TitledPane;
pub use widget_pod::WidgetPod;

use crate::{
    core::DebugNode,
    drawing::PaintCtx,
    layout::Alignment,
    style,
    style::Style,
    widget::{
        align::{HorizontalAlignment, VerticalAlignment},
        constrained::{Fill, FixedHeight, FixedWidth, MaxHeight, MaxWidth, MinHeight, MinWidth},
        font_size::FontSize,
    },
    Environment, Event, EventCtx, Layout, LayoutConstraints, LayoutCtx, Length, LengthOrPercentage, UnitExt, Widget,
    WidgetId,
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

/// Widget modifiers.
pub trait Modifier {
    fn layout<W: Widget>(
        &self,
        ctx: &mut LayoutCtx,
        widget: &W,
        constraints: &LayoutConstraints,
        env: &Environment,
    ) -> Layout;

    fn debug_node(&self) -> DebugNode {
        DebugNode::new("Modifier")
    }
}

/// Wraps a widget with a layout modifier.
#[derive(Clone)]
pub struct Modified<M, W>(pub M, pub W);

impl<M, W> WidgetWrapper for Modified<M, W>
where
    W: Widget,
    M: Modifier,
{
    type Inner = W;

    fn inner(&self) -> &Self::Inner {
        &self.1
    }

    fn inner_mut(&mut self) -> &mut Self::Inner {
        &mut self.1
    }

    fn layout(&self, ctx: &mut LayoutCtx, constraints: &LayoutConstraints, env: &Environment) -> Layout {
        self.0.layout(ctx, &self.1, constraints, env)
    }

    fn debug_node(&self) -> DebugNode {
        self.0.debug_node()
    }
}

/// Widgets that have only one child and wish to defer to this child's `Widget` implementation.
pub trait WidgetWrapper {
    type Inner: Widget + ?Sized;

    fn inner(&self) -> &Self::Inner;
    fn inner_mut(&mut self) -> &mut Self::Inner;

    fn widget_id(&self) -> Option<WidgetId> {
        self.inner().widget_id()
    }

    fn event(&self, ctx: &mut EventCtx, event: &mut Event, env: &Environment) {
        self.inner().event(ctx, event, env)
    }

    fn route_event(&self, ctx: &mut EventCtx, event: &mut Event, env: &Environment) {
        self.inner().route_event(ctx, event, env)
    }

    fn layout(&self, ctx: &mut LayoutCtx, constraints: &LayoutConstraints, env: &Environment) -> Layout {
        self.inner().layout(ctx, constraints, env)
    }

    fn paint(&self, ctx: &mut PaintCtx) {
        self.inner().paint(ctx)
    }

    fn debug_node(&self) -> DebugNode {
        self.inner().debug_node()
    }
}

impl<T: WidgetWrapper> Widget for T {
    fn widget_id(&self) -> Option<WidgetId> {
        WidgetWrapper::widget_id(self)
    }

    fn layout(&self, ctx: &mut LayoutCtx, constraints: &LayoutConstraints, env: &Environment) -> Layout {
        WidgetWrapper::layout(self, ctx, constraints, env)
    }

    fn event(&self, ctx: &mut EventCtx, event: &mut Event, env: &Environment) {
        WidgetWrapper::event(self, ctx, event, env)
    }

    fn paint(&self, ctx: &mut PaintCtx) {
        WidgetWrapper::paint(self, ctx)
    }

    fn route_event(&self, ctx: &mut EventCtx, event: &mut Event, env: &Environment) {
        WidgetWrapper::route_event(self, ctx, event, env)
    }

    fn debug_node(&self) -> DebugNode {
        WidgetWrapper::debug_node(self)
    }
}

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

    /// Sets the background paint of the widget.
    #[must_use]
    fn background(self, image: impl TryInto<style::Image>, shape: impl Into<style::Shape>) -> Overlay<Self, Shape> {
        let image = image.try_into().unwrap_or_else(|_| {
            warn!("invalid CSS image value");
            style::Image::default()
        });
        Overlay::new(self, Shape::new(shape.into(), image), ZOrder::Below)
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

    /*/// Wraps the widget in a `ConstrainedBox` that constrains the width of the widget.
    #[must_use]
    fn fix_width(self, width: impl Into<Length>) -> ConstrainedBox<Self> {
        let width = width.into();
        ConstrainedBox::new(self).min_width(width).max_width(width)
    }

    /// Wraps the widget in a `ConstrainedBox` that constrains the height of the widget.
    #[must_use]
    fn fix_height(self, height: impl Into<Length>) -> ConstrainedBox<Self> {
        let height = height.into();
        ConstrainedBox::new(self).min_height(height).max_height(height)
    }

    /// Wraps the widget in a `ConstrainedBox` that constrains the size of the widget.
    #[must_use]
    fn fix_size(self, width: impl Into<Length>, height: impl Into<Length>) -> ConstrainedBox<Self> {
        let width = width.into();
        let height = height.into();
        ConstrainedBox::new(self)
            .min_width(width)
            .max_width(width)
            .min_height(height)
            .max_height(height)
    }*/

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
    fn padding(
        self,
        top: impl Into<Length>,
        right: impl Into<Length>,
        bottom: impl Into<Length>,
        left: impl Into<Length>,
    ) -> Modified<Padding, Self> {
        Modified(Padding::new(top, right, bottom, left), self)
    }

    /// Sets the font size.
    #[must_use]
    fn font_size(self, size: impl Into<Length>) -> Modified<FontSize, Self> {
        Modified(FontSize(size.into()), self)
    }

    #[must_use]
    fn style(self, style: impl TryInto<Style>) -> StyledBox<Self> {
        StyledBox::new(self, style)
    }
}

impl<W: Widget + 'static> WidgetExt for W {}

/// Prelude containing commonly used items for writing custom widgets.
pub mod prelude {
    #[doc(hidden)]
    pub use crate::{
        cache::Signal, composable, drawing::PaintCtx, widget::WidgetExt, widget::WidgetPod, widget::WidgetWrapper,
        Alignment, BoxConstraints, Environment, Event, EventCtx, Layout, LayoutCache, LayoutConstraints, LayoutCtx,
        Length, Measurements, Offset, Orientation, Point, Rect, Size, Transform, UnitExt, Widget, WidgetId,
    };
}
