use crate::{
    env,
    event::Event,
    layout::{align_boxes, Alignment, BoxConstraints, Measurements},
    renderer::ButtonState,
    style, theme,
    visual::Visual,
    widget::{BoxedWidget, Text, TypedWidget, Widget, WidgetExt},
    Environment, EventCtx, LayoutCtx, PaintCtx, Point, Rect, SideOffsets, Size,
};
use euclid::default::SideOffsets2D;
use generational_indextree::NodeId;
use kyute_shell::drawing::{
    gradient::{ColorInterpolationMode, ExtendMode, GradientStopCollection},
    Color, DrawContext, IntoBrush,
};
use palette::{Blend, LinSrgb, LinSrgba, Srgb};
use std::any::{Any, TypeId};

/// Node visual for a button.
pub struct ButtonVisual {
    bg_color: Color,
    border_color: Color,
    on_click: Option<Box<dyn FnMut(&mut EventCtx)>>,
}

impl Default for ButtonVisual {
    fn default() -> Self {
        ButtonVisual {
            bg_color: Default::default(),
            border_color: Default::default(),
            on_click: None,
        }
    }
}

impl Visual for ButtonVisual {
    fn paint(&mut self, ctx: &mut PaintCtx, env: &Environment) {
        ctx.draw_styled_box("button", style::PaletteIndex(0));
    }

    fn hit_test(&mut self, _point: Point, _bounds: Rect) -> bool {
        false
    }

    fn event(&mut self, ctx: &mut EventCtx, event: &Event) {
        match event {
            Event::PointerDown(p) => {
                if let Some(on_click) = &mut self.on_click {
                    (on_click)(ctx);
                }
                ctx.request_focus();
                ctx.request_redraw();
                ctx.set_handled();
            }
            Event::PointerOver(p) => ctx.request_redraw(),
            Event::PointerOut(p) => ctx.request_redraw(),
            _ => {}
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

/// A clickable push button with a text label.
///
/// This widget is influenced by the following style variables:
/// - [`PADDING`](crate::style::PADDING): padding for the label inside the button.
pub struct Button<'a> {
    label: BoxedWidget<'a>,
    on_click: Option<Box<dyn FnMut(&mut EventCtx)>>,
}

impl<'a> Button<'a> {
    /// Creates a new button with the given text as the label.
    pub fn new(label: impl Into<String>) -> Button<'a> {
        Button {
            label: Text::new(label).boxed(),
            on_click: None,
        }
    }

    pub fn on_click(mut self, on_click: impl FnMut(&mut EventCtx) + 'static) -> Button<'a> {
        self.on_click = Some(Box::new(on_click));
        self
    }
}

//-----------------------------------------------------
// Widget implementation
impl<'a> TypedWidget for Button<'a> {
    type Visual = ButtonVisual;

    fn layout(
        self,
        context: &mut LayoutCtx,
        previous_visual: Option<Box<ButtonVisual>>,
        constraints: &BoxConstraints,
        env: Environment,
    ) -> (Box<ButtonVisual>, Measurements) {
        //let on_click = self.on_click;
        let mut visual = previous_visual.unwrap_or_default();
        visual.on_click = self.on_click;

        //visual.on_click = on_click;
        let min_width = env.get(theme::MinButtonWidth);
        let min_height = env.get(theme::MinButtonHeight);

        // measure the label inside
        let padding: SideOffsets = env.get(theme::ButtonLabelPadding);
        let label_constraints = constraints.deflate(&padding);
        let (label_id, label_measurements) =
            context.emit_child(self.label, &label_constraints, env, None);

        // now measure the button itself
        let mut measurements = Measurements {
            size: label_measurements.size,
            baseline: None,
        };

        // add padding on the sides
        measurements.size += Size::new(padding.horizontal(), padding.vertical());

        // apply minimum size
        measurements.size.width = measurements.size.width.max(min_width);
        measurements.size.height = measurements.size.height.max(min_height);

        // constrain size
        measurements.size = constraints.constrain(measurements.size);

        // center the label inside the button
        let label_offset = align_boxes(Alignment::CENTER, &mut measurements, label_measurements);
        context.set_child_offset(label_id, label_offset);

        (visual, measurements)
    }
}
