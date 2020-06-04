use crate::layout::{Bounds, Offset, Point, Size};
use crate::renderer::colors::DEFAULT_COLORS;
use crate::renderer::{ButtonState, Colors, LineSegment, CheckBoxOptions};
use crate::widget::textedit::Selection;
use crate::{BoxConstraints, Measurements, PaintCtx};
use euclid::default::Rect;
use kyute_shell::drawing::brush::IntoBrush;
use kyute_shell::drawing::{Brush, Color, DrawContext, DrawTextOptions, RectExt, SolidColorBrush};
use kyute_shell::platform::Platform;
use kyute_shell::text::{TextFormat, TextFormatBuilder, TextLayout};
use palette::Srgba;
use std::ops::Range;
use std::rc::Rc;
use bitflags::_core::any::Any;

/// Metrics of a button frame renderered by [`Renderer::draw_button`].
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct ButtonStyle {
    pub min_size: Size,
    /// Padding between the inside of the border and the contents of the button (does not include border size).
    pub padding: f64,
}

/// Metrics of a text edit frame.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct TextEditStyle {
    /// Minimum border-box size of a text edit box
    pub min_size: Size,
    /// Padding between the inside of the border and the text (does not include border size).
    pub padding: f64,
}

/// Metrics of a slider frame.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct SliderStyle {
    pub min_size: Size,
    pub padding: f64,
    pub min_knob_size: f64,
}

/*
/// GUI renderer.
///
/// The [`Renderer`] and the [`Painter`] types are in charge of rendering the primitive elements
/// of a user interface, such as labels, button frames, panel backgrounds, slider knows,
/// scroll bar handles, etc.
///
/// Note that it is the renderer that decides the appearance of those primitive elements:
/// as such, it is more akin to a "style" or "theme" than a low-level drawing API.
/// The renderer specifies the metrics of the primitive elements, such as their minimum or preferred
/// sizes, which you can query with the [`widget_metrics`](Renderer::widget_metrics) method.
///
/// The [`Renderer`] contains the renderer state common to all windows.
/// The related [`Painter`] type represents a painting context of the renderer on a particular
/// window.
///
/// Bikeshedding:
/// First, this type should really be a trait. Possible names:
///  - Theme
///  - Style
///  - Skin
///  - Look / LookAndFeel
///
#[derive(Clone)]
pub struct DefaultTheme {
    /// The text format used for labels on buttons and such.
    label_text_format: TextFormat,
    data: ThemeData,
}

impl DefaultTheme {
    /// Creates the default UI theme
    pub fn new(platform: &Platform) -> DefaultTheme {
        let data = DEFAULT_THEME_DATA;

        let label_text_format = TextFormatBuilder::new(platform)
            .family("Segoe UI")
            .size(14.0)
            .build()
            .unwrap(); // TODO try something else on failure

        Theme {
            data,
            label_text_format,
        }
    }

    fn draw_box(
        &self,
        context: &mut PaintCtx,
        bounds: Bounds,
        fill: &dyn Brush,
        border: &dyn Brush,
        border_width: f64,
    ) {
        // box background
        context.fill_rectangle(bounds.stroke_inset(border_width), &fill);
        // border
        if context.is_hovering() || context.is_focused() {
            context.draw_rectangle(bounds.stroke_inset(border_width), &border, border_width);
        }
    }

    /// Returns the padding for a content area.
    fn content_padding(&self, content_area: ContentArea) -> EdgeInsets {
        let w = match content_area {
            content_areas::BUTTON => self.data.button_style.padding + self.data.frame_border_width,
            content_areas::TEXT_EDIT => self.data.text_edit_style.padding + self.data.frame_border_width,
            _ => 0.0,
        };
        EdgeInsets::all(w )
    }
}

impl Theme for DefaultTheme {

    fn draw_primitive(&self, context: &mut PaintCtx, primitive: Primitive, bounds: Bounds, options: Option<&dyn Any>)
    {
        match primitive {
            primitives::BUTTON_FRAME => {
                let bg_brush = self.data.colors.button.into_brush(context);
                let border_brush = self.data.colors.border.into_brush(context);
                ctx.fill_rectangle(bounds, &bg_brush);
                let stroke_size = 1.0;
                ctx.draw_rectangle(
                    bounds.inflate(-0.5 * stroke_size, -0.5 * stroke_size),
                    &border_brush,
                    1.0,
                );
            },
            primitives::CHECK_BOX => {
                let options : &CheckBoxOptions = options.unwrap().downcast_ref().unwrap();
                unimplemented!()
            },
            primitives::RADIO_BUTTON => {
                let options : &RadioButtonOptions = options.unwrap().downcast_ref().unwrap();
                unimplemented!()
            },
            primitives::SLIDER_FRAME => {
                let fill = self.data.colors.frame_bg.into_brush(context);
                let border = self.data.colors.border.into_brush(context);
                self.draw_box(ctx, bounds, &fill, &border, 1.0);
            },
            primitives::TEXT_EDIT_FRAME => {},
            _ => {}
        }
    }

    fn content_constraints(&self, content_area: ContentArea, outer_constraints: &BoxConstraints) -> BoxConstraints {
        outer_constraints.deflate(&self.content_padding(content_area))
    }

    fn measure_with_contents(
        &self,
        content_area: ContentArea,
        content_measurements: Measurements,
    ) -> (Measurements, Offset)
    {
        // add padding to the content size
        let insets = self.content_padding(content_area);
        let measurements = Measurements {
            size: Size::new(content_measurements.size.width + insets.left + insets.right,
             content_measurements.size.height + insets.bottom + insets.top),
            baseline: content_measurements.baseline.map(|b| b + insets.top)
        };

        let offset = Offset::new(insets.left, insets.top);
        (measurements, offset)
    }


    /*/// Returns the bounds of the content area for the specified frame given a size.
    pub fn content_bounds(&self, class: FrameClass, size: Size) -> Bounds {
        let insets = self.content_padding(class);
        Bounds::new(Point::new(insets.left, insets.top), Size::new(
            size.width - insets.left - insets.right,
            size.height - insets.top - insets.bottom
        ))
    }*/

    /// Returns the line defining a slider track.
    fn slider_track(&self, size: Size) -> LineSegment {
        // account for border and padding
        let w =
            size.width - 2.0 * self.data.frame_border_width - 2.0 * self.data.slider_style.padding;
        let y = size.height / 2.0;
        LineSegment {
            start: Point::new(
                self.data.frame_border_width + self.data.slider_style.padding,
                y,
            ),
            end: Point::new(w, y),
        }
    }

    /// Layouts a frame given the specified constraints and the content size.
    ///
    /// Returns the measurements of the frame, and the offset to the content box.
    fn measure_frame(
        &self,
        frame: FrameClass,
        constraints: &BoxConstraints,
        content_measurements: Measurements,
    ) -> (Measurements, Offset) {
        //let offset = self.
        unimplemented!()
    }

    /// Draws a frame in the specified bounds
    fn draw_text_box_frame(&self, ctx: &mut PaintCtx, bounds: Bounds) {
    }

    fn draw_slider_frame(&self, ctx: &mut PaintCtx, bounds: Bounds) {
        let fill = self.data.colors.frame_bg.into_brush(ctx);
        let border = self.data.colors.border.into_brush(ctx);
        self.draw_box(ctx, bounds, &fill, &border, 1.0);
    }

    /// Draws editable text (text, selection highlight, and caret)
    fn draw_editable_text(
        &self,
        ctx: &mut PaintCtx,
        bounds: Bounds,
        text_layout: &mut TextLayout,
        selection: Selection,
    ) {
        let text_brush = SolidColorBrush::new(ctx, self.data.colors.text.into());
        let caret_brush = SolidColorBrush::new(ctx, self.data.colors.text.into());
        let selected_bg_brush = SolidColorBrush::new(ctx, self.data.colors.text_selected_bg.into());
        let selected_text_brush = SolidColorBrush::new(ctx, self.data.colors.text.into());

        ctx.save();
        ctx.transform(&bounds.origin.to_vector().to_transform());

        // text color
        text_layout.set_drawing_effect(&text_brush, ..);
        if !selection.is_empty() {
            // FIXME slightly changes the layout when the selection straddles a kerning pair?
            text_layout.set_drawing_effect(&selected_text_brush, selection.min()..selection.max());
        }

        // selection highlight
        if !selection.is_empty() {
            let selected_areas = text_layout
                .hit_test_text_range(selection.min()..selection.max(), &bounds.origin)
                .unwrap();
            for sa in selected_areas {
                ctx.fill_rectangle(sa.bounds.round_out(), &selected_bg_brush);
            }
        }

        // text
        ctx.draw_text_layout(
            bounds.origin,
            &text_layout,
            &text_brush,
            DrawTextOptions::ENABLE_COLOR_FONT,
        );

        // caret
        if ctx.is_focused() {
            let caret_hit_test = text_layout.hit_test_text_position(selection.end).unwrap();

            //dbg!(caret_hit_test);
            ctx.fill_rectangle(
                Rect::new(
                    caret_hit_test.point.floor(),
                    Size::new(1.0, caret_hit_test.metrics.bounds.size.height),
                ),
                &caret_brush,
            );
        }

        ctx.restore();
    }


}*/

/*
const DEFAULT_THEME_DATA: ThemeData = ThemeData {
    colors: Colors {
        text: [0.95f32, 0.96f32, 0.98f32, 1.00f32],
        text_disabled: [0.36f32, 0.42f32, 0.47f32, 1.00f32],
        window_bg: [0.11f32, 0.15f32, 0.17f32, 1.00f32],
        child_bg: [0.15f32, 0.18f32, 0.22f32, 1.00f32],
        popup_bg: [0.08f32, 0.08f32, 0.08f32, 0.94f32],
        border: [0.08f32, 0.10f32, 0.12f32, 1.00f32],
        border_shadow: [0.00f32, 0.00f32, 0.00f32, 0.00f32],
        frame_bg: [0.20f32, 0.25f32, 0.29f32, 1.00f32],
        frame_bg_hovered: [0.12f32, 0.20f32, 0.28f32, 1.00f32],
        frame_bg_active: [0.09f32, 0.12f32, 0.14f32, 1.00f32],
        title_bg: [0.09f32, 0.12f32, 0.14f32, 0.65f32],
        title_bg_active: [0.08f32, 0.10f32, 0.12f32, 1.00f32],
        title_bg_collapsed: [0.00f32, 0.00f32, 0.00f32, 0.51f32],
        menu_bar_bg: [0.15f32, 0.18f32, 0.22f32, 1.00f32],
        scrollbar_bg: [0.02f32, 0.02f32, 0.02f32, 0.39f32],
        scrollbar_grab: [0.20f32, 0.25f32, 0.29f32, 1.00f32],
        scrollbar_grab_hovered: [0.18f32, 0.22f32, 0.25f32, 1.00f32],
        scrollbar_grab_active: [0.09f32, 0.21f32, 0.31f32, 1.00f32],
        check_mark: [0.28f32, 0.56f32, 1.00f32, 1.00f32],
        slider_grab: [0.28f32, 0.56f32, 1.00f32, 1.00f32],
        slider_grab_active: [0.37f32, 0.61f32, 1.00f32, 1.00f32],
        button: [0.20f32, 0.25f32, 0.29f32, 1.00f32],
        button_hovered: [0.28f32, 0.56f32, 1.00f32, 1.00f32],
        button_active: [0.06f32, 0.53f32, 0.98f32, 1.00f32],
        header: [0.20f32, 0.25f32, 0.29f32, 0.55f32],
        header_hovered: [0.26f32, 0.59f32, 0.98f32, 0.80f32],
        header_active: [0.26f32, 0.59f32, 0.98f32, 1.00f32],
        separator: [0.20f32, 0.25f32, 0.29f32, 1.00f32],
        separator_hovered: [0.10f32, 0.40f32, 0.75f32, 0.78f32],
        separator_active: [0.10f32, 0.40f32, 0.75f32, 1.00f32],
        resize_grip: [0.26f32, 0.59f32, 0.98f32, 0.25f32],
        resize_grip_hovered: [0.26f32, 0.59f32, 0.98f32, 0.67f32],
        resize_grip_active: [0.26f32, 0.59f32, 0.98f32, 0.95f32],
        tab: [0.11f32, 0.15f32, 0.17f32, 1.00f32],
        tab_hovered: [0.26f32, 0.59f32, 0.98f32, 0.80f32],
        tab_active: [0.20f32, 0.25f32, 0.29f32, 1.00f32],
        tab_unfocused: [0.11f32, 0.15f32, 0.17f32, 1.00f32],
        tab_unfocused_active: [0.11f32, 0.15f32, 0.17f32, 1.00f32],
        plot_lines: [0.61f32, 0.61f32, 0.61f32, 1.00f32],
        plot_lines_hovered: [1.00f32, 0.43f32, 0.35f32, 1.00f32],
        plot_histogram: [0.90f32, 0.70f32, 0.00f32, 1.00f32],
        plot_histogram_hovered: [1.00f32, 0.60f32, 0.00f32, 1.00f32],
        text_selected_bg: [0.26f32, 0.59f32, 0.98f32, 0.35f32],
        drag_drop_target: [1.00f32, 1.00f32, 0.00f32, 0.90f32],
        nav_highlight: [0.26f32, 0.59f32, 0.98f32, 1.00f32],
        nav_windowing_highlight: [1.00f32, 1.00f32, 1.00f32, 0.70f32],
        nav_windowing_dim_bg: [0.80f32, 0.80f32, 0.80f32, 0.20f32],
        modal_window_dim_bg: [0.80f32, 0.80f32, 0.80f32, 0.35f32],
        docking_preview: [0.0, 0.0, 0.0, 1.0],
        docking_empty_bg: [0.0, 0.0, 0.0, 1.0],
    },
    button_style: ButtonStyle {
        min_size: Size::new(10.0, 10.0),
        padding: 4.0,
    },
    slider_style: SliderStyle {
        min_size: Size::new(10.0, 10.0),
        padding: 2.0,
        min_knob_size: 0.0,
    },
    text_edit_style: TextEditStyle {
        min_size: Size::new(10.0, 10.0),
        padding: 2.0,
    },
    frame_border_width: 0.0,
};*/
