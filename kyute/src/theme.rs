/// Environment keys that control the visual aspect (theme) of common widgets.
use kyute_shell::drawing::{
    Color, ColorInterpolationMode, DrawContext, ExtendMode, GradientStopCollection, IntoBrush,
    Offset,
};
use palette::{Alpha, LinSrgb, LinSrgba, Shade, Srgb, Srgba};
use crate::{SideOffsets, EnvKey};
use kyute_shell::text::TextFormat;
use std::sync::Arc;
use crate::style::StyleSet;

pub const FONT_SIZE: EnvKey<f64> = EnvKey::new("kyute.theme.font_size"); // [14.0];
pub const FONT_NAME: EnvKey<String> = EnvKey::new("kyute.theme.font_name");
pub const MIN_BUTTON_WIDTH : EnvKey<f64> = EnvKey::new("kyute.theme.min_button_width"); // [30.0];
pub const MIN_BUTTON_HEIGHT : EnvKey<f64> = EnvKey::new("kyute.theme.min_button_height"); // [14.0];
pub const FRAME_BG_SUNKEN_COLOR : EnvKey<Color> = EnvKey::new("kyute.theme.frame_bg_sunken_color"); // [Color::new(0.227, 0.227, 0.227, 1.0)];
pub const FRAME_BG_NORMAL_COLOR : EnvKey<Color> = EnvKey::new("kyute.theme.frame_bg_normal_color"); // [Color::new(0.326, 0.326, 0.326, 1.0)];
pub const FRAME_BG_RAISED_COLOR : EnvKey<Color> = EnvKey::new("kyute.theme.frame_bg_raised_color"); // [Color::new(0.424, 0.424, 0.424, 1.0)];
pub const FRAME_FOCUS_COLOR : EnvKey<Color> = EnvKey::new("kyute.theme.frame_focus_color"); // [Color::new(0.6, 0.6, 0.9, 1.0)];
pub const FRAME_BORDER_COLOR : EnvKey<Color> = EnvKey::new("kyute.theme.frame_border_color"); // [Color::new(0.13,0.13,0.13,1.0)];
pub const FRAME_OUTER_HIGHLIGHT_OPACITY : EnvKey<f64> = EnvKey::new("kyute.theme.frame_outer_highlight_opacity"); // [0.15];
pub const FRAME_EDGE_DARKENING_INTENSITY : EnvKey<f64> = EnvKey::new("kyute.theme.frame_edge_darkening_intensity"); // [0.5];
pub const BUTTON_TOP_HIGHLIGHT_INTENSITY : EnvKey<f64> = EnvKey::new("kyute.theme.button_top_highlight_intensity"); // [0.2];
pub const BUTTON_BACKGROUND_TOP_COLOR : EnvKey<Color> = EnvKey::new("kyute.theme.button_background_top_color"); // [Color::new(0.45, 0.45, 0.45, 1.0)];
pub const BUTTON_BACKGROUND_BOTTOM_COLOR : EnvKey<Color> = EnvKey::new("kyute.theme.button_background_bottom_color"); // [Color::new(0.40, 0.40, 0.40, 1.0)];
pub const BUTTON_BORDER_BOTTOM_COLOR : EnvKey<Color> = EnvKey::new("kyute.theme.button_border_bottom_color"); // [Color::new(0.1, 0.1, 0.1, 1.0)];
pub const BUTTON_BORDER_TOP_COLOR : EnvKey<Color> = EnvKey::new("kyute.theme.button_border_top_color"); // [Color::new(0.18, 0.18, 0.18, 1.0)];
pub const BUTTON_BORDER_RADIUS : EnvKey<f64> = EnvKey::new("kyute.theme.button_border_radius"); // [2.0];
pub const BUTTON_LABEL_PADDING : EnvKey<SideOffsets> = EnvKey::new("kyute.theme.button_label_padding"); // [SideOffsets::new(2.0, 5.0, 2.0, 5.0)];
pub const FLEX_SPACING: EnvKey<f64> = EnvKey::new("kyute.theme.flex_spacing"); // [2.0];
pub const SLIDER_PADDING : EnvKey<SideOffsets> = EnvKey::new("kyute.theme.slider_padding"); // [SideOffsets::new_all_same(0.0)];
pub const SLIDER_HEIGHT : EnvKey<f64> = EnvKey::new("kyute.theme.slider_height"); // [14.0];
pub const SLIDER_TRACK_Y : EnvKey<f64> = EnvKey::new("kyute.theme.slider_track_y"); // [9.0];
pub const SLIDER_KNOB_Y : EnvKey<f64> = EnvKey::new("kyute.theme.slider_knob_y"); // [7.0];
pub const SLIDER_KNOB_WIDTH : EnvKey<f64> = EnvKey::new("kyute.theme.slider_knob_width"); // [11.0];
pub const SLIDER_KNOB_HEIGHT : EnvKey<f64> = EnvKey::new("kyute.theme.slider_knob_height"); // [11.0];
pub const SLIDER_TRACK_HEIGHT : EnvKey<f64> = EnvKey::new("kyute.theme.slider_track_height"); // [4.0];
pub const TEXT_EDIT_FONT_SIZE: EnvKey<f64> = EnvKey::new("kyute.theme.text_edit_font_size"); // [12.0];
pub const TEXT_EDIT_FONT_NAME: EnvKey<String> = EnvKey::new("kyute.theme.text_edit_font_name"); // ["Segoe UI"];
pub const TEXT_EDIT_PADDING: EnvKey<SideOffsets> = EnvKey::new("kyute.theme.text_edit_padding"); // [SideOffsets::new_all_same(2.0)];
pub const TEXT_EDIT_CARET_COLOR: EnvKey<Color> = EnvKey::new("kyute.theme.text_edit_caret_color"); // [Color::new(1.0,1.0,1.0,1.0)];
pub const TEXT_EDIT_BORDER_COLOR : EnvKey<Color> = EnvKey::new("kyute.theme.text_edit_border_color"); // [Color::new(0.0,0.0,0.0,1.0)];
pub const TEXT_EDIT_BACKGROUND_COLOR : EnvKey<Color> = EnvKey::new("kyute.theme.text_edit_background_color"); // [Color::new(1.0,1.0,1.0,1.0)];
pub const TEXT_EDIT_BACKGROUND_STYLE : EnvKey<StyleSet> = EnvKey::new("kyute.theme.text_edit_background_style");
pub const TEXT_EDIT_BORDER_WIDTH : EnvKey<f64> = EnvKey::new("kyute.theme.text_edit_border_width"); // [1.0];
pub const TEXT_COLOR : EnvKey<Color> = EnvKey::new("kyute.theme.text_color"); // [Color::new(0.96,0.96,0.96,1.0)];
pub const SELECTED_TEXT_BACKGROUND_COLOR : EnvKey<Color> = EnvKey::new("kyute.theme.selected_text_background_color"); // [Color::new(0.6,0.6,0.8,1.0)];
pub const SELECTED_TEXT_COLOR : EnvKey<Color> = EnvKey::new("kyute.theme.selected_text_color"); // [Color::new(1.0,1.0,1.0,1.0)];

pub const DEFAULT_TEXT_FORMAT : EnvKey<TextFormat> = EnvKey::new("kyute.theme.text_format"); // [Color::new(1.0,1.0,1.0,1.0)];
pub const BUTTON_STYLE : EnvKey<StyleSet> = EnvKey::new("kyute.theme.button_style");
pub const SLIDER_KNOB_STYLE : EnvKey<StyleSet> = EnvKey::new("kyute.theme.slider_knob_style");
pub const SLIDER_TRACK_STYLE : EnvKey<StyleSet> = EnvKey::new("kyute.theme.slider_track_style");

/*
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum FrameType {
    PanelBackground, // border
    Button,          // border + outer highlight
    TextEdit,        // border + sunken +
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct FrameStyle {
    pub hovered: bool,
    pub disabled: bool,
    pub focused: bool,
    pub pressed: bool,
}

fn make_gradient(ctx: &mut DrawContext, a: LinSrgba, b: LinSrgba) -> GradientStopCollection {
    GradientStopCollection::new(
        ctx,
        ColorInterpolationMode::GammaCorrect,
        ExtendMode::Clamp,
        &[(0.0, Srgba::from_linear(a)), (1.0, Srgba::from_linear(b))],
    )
}

fn draw_outer_highlight(
    ctx: &mut DrawContext,
    focused: bool,
    bounds: Bounds,
    radius: f64,
    env: &Environment,
) {
    let frame_highlight_opacity = env.get(FrameOuterHighlightOpacity);

    if focused {
        let brush = env.get(FrameFocusColor).into_brush(ctx);
        ctx.draw_rounded_rectangle(bounds.inflate(0.5, 0.5), radius, radius, &brush, 1.0);
    } else {
        let brush = make_vertical_gradient_brush(
            ctx,
            bounds.size.height,
            0.8 * bounds.size.height,
            LinSrgba::new(1.0, 1.0, 1.0, 1.0),
            LinSrgba::new(1.0, 1.0, 1.0, 0.0),
            frame_highlight_opacity,
        );
        ctx.draw_rounded_rectangle(bounds.inflate(0.5, 0.5), radius, radius, &brush, 1.0);
    }
}

pub fn draw_button_frame(
    ctx: &mut DrawContext,
    style: &FrameStyle,
    bounds: Bounds,
    env: &Environment,
) {
    let raised: LinSrgba = env.get(FrameBgRaisedColor).into_linear();
    let sunken: LinSrgba = env.get(FrameBgSunkenColor).into_linear();
    let radius = env.get(ButtonBorderRadius);

    // ---- draw background ----
    let mut bg_base = raised;
    if style.hovered {
        bg_base = bg_base.lighten(0.2);
    }
    let bg_low = bg_base.darken(0.05);
    let bg_high = bg_base.lighten(0.05);
    let bg_brush = make_vertical_gradient_brush(ctx, bounds.size.height, 0.0, bg_low, bg_high, 1.0);
    ctx.fill_rounded_rectangle(bounds, radius, radius, &bg_brush);

    // ---- top highlight ----
    let top_highlight_brush = Color::new(1.0, 1.0, 1.0, 0.3).into_brush(ctx);
    ctx.fill_rectangle(
        Bounds::new(
            bounds.origin + Offset::new(1.0, 1.0),
            Size::new(bounds.size.width - 1.0, 1.0),
        ),
        &top_highlight_brush,
    );

    // ---- draw border ----
    let border_rect = bounds.inflate(-0.5, -0.5);
    let mut border_base = sunken.darken(0.023);
    //let mut border_low = border_base.darken(0.01);
    //let mut border_high = border_base.lighten(0.01);
    let brush = border_base.into_brush(ctx);

    /*let brush = make_vertical_gradient_brush(ctx, bounds.size.height, 0.0,
    border_low, border_high,
    1.0);*/
    ctx.draw_rounded_rectangle(bounds.inflate(-0.5, -0.5), radius, radius, &brush, 1.0);

    // ---- outer highlight ----
    draw_outer_highlight(ctx, style.focused, bounds, radius, env);
}

pub fn draw_text_box_frame(
    ctx: &mut DrawContext,
    style: &FrameStyle,
    bounds: Bounds,
    env: &Environment,
) {
    let sunken: LinSrgba = env.get(FrameBgSunkenColor).into_linear();

    // ---- draw background ----
    let mut bg_base = sunken;
    if style.hovered {
        bg_base = bg_base.lighten(0.04);
    }
    let bg_brush = bg_base.into_brush(ctx);
    ctx.fill_rectangle(bounds, &bg_brush);
    // ---- draw border ----
    let mut border_base = sunken.darken(0.023);
    //let mut border_low = border_base.darken(0.01);
    //let mut border_high = border_base.lighten(0.01);
    let brush = border_base.into_brush(ctx);
    /*let brush = make_vertical_gradient_brush(ctx, bounds.size.height, 0.0,
    border_low, border_high,
    1.0);*/
    ctx.draw_rectangle(bounds.inflate(-0.5, -0.5), &brush, 1.0);

    // ---- outer highlight ----
    draw_outer_highlight(ctx, false, bounds, 0.0, env);
}
*/
