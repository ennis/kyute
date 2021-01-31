use crate::env::Key;
use crate::{Environment, Rect, SideOffsets, Size};
/// Environment keys that control the visual aspect (theme) of common widgets.
use kyute_shell::drawing::{
    Color, ColorInterpolationMode, DrawContext, ExtendMode, GradientStopCollection, IntoBrush,
    Offset,
};
use palette::{Alpha, LinSrgb, LinSrgba, Shade, Srgb, Srgba};

impl_keys!(
/// Default font size.
FontSize: f64 [14.0];
/// Default font family
#[cfg(windows)]
FontName: &'a str ["Segoe UI"];
/// Minimum button width
MinButtonWidth : f64 [30.0];
/// Minimum button height
MinButtonHeight : f64 [14.0];

//------ Frame shades
FrameBgSunkenColor : Color [Color::new(0.227, 0.227, 0.227, 1.0)];
FrameBgNormalColor : Color [Color::new(0.326, 0.326, 0.326, 1.0)];
FrameBgRaisedColor : Color [Color::new(0.424, 0.424, 0.424, 1.0)];
FrameFocusColor : Color [Color::new(0.6, 0.6, 0.9, 1.0)];
FrameBorderColor : Color [Color::new(0.13,0.13,0.13,1.0)];  // ~ darker FrameBgSunkenColor
FrameOuterHighlightOpacity : f64 [0.15];
FrameEdgeDarkeningIntensity : f64 [0.5];

//------
ButtonTopHighlightIntensity : f64 [0.2];
ButtonBackgroundTopColor : Color [Color::new(0.45, 0.45, 0.45, 1.0)];        // ~ FrameBgRaisedColor
ButtonBackgroundBottomColor : Color [Color::new(0.40, 0.40, 0.40, 1.0)];     // ~ FrameBgRaisedColor
ButtonBorderBottomColor : Color [Color::new(0.1, 0.1, 0.1, 1.0)]; // ~ FrameBorderColor
ButtonBorderTopColor : Color [Color::new(0.18, 0.18, 0.18, 1.0)]; // ~ FrameBorderColor
ButtonBorderRadius : f64 [2.0];
ButtonLabelPadding : SideOffsets [SideOffsets::new(2.0, 5.0, 2.0, 5.0)];

//------
FlexSpacing: f64 [2.0];

/// Label padding.
SliderPadding : SideOffsets [SideOffsets::new_all_same(0.0)];
SliderHeight : f64 [14.0];
SliderTrackY : f64 [9.0];
SliderKnobY : f64 [7.0];
SliderKnobWidth : f64 [11.0];
SliderKnobHeight : f64 [11.0];
SliderTrackHeight : f64 [4.0];

///
TextEditFontSize: f64 [12.0];
TextEditFontName: &'a str ["Segoe UI"];
TextEditPadding: SideOffsets [SideOffsets::new_all_same(2.0)];
TextEditCaretColor: Color [Color::new(1.0,1.0,1.0,1.0)];
TextEditBorderColor : Color [Color::new(0.0,0.0,0.0,1.0)];
TextEditBackgroundColor : Color [Color::new(1.0,1.0,1.0,1.0)];
TextEditBorderWidth : f64 [1.0];

TextColor : Color [Color::new(0.96,0.96,0.96,1.0)];
SelectedTextBackgroundColor : Color [Color::new(0.6,0.6,0.8,1.0)];
SelectedTextColor : Color [Color::new(1.0,1.0,1.0,1.0)];
);

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
