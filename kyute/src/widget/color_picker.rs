use crate::{
    drawing::ToSkia,
    make_uniform_data, style,
    style::{BoxStyle, Paint, PaintCtxExt},
    theme, tweak,
    widget::{
        grid,
        grid::{AlignItems, GridLayoutExt, TrackSizePolicy},
        prelude::*,
        slider::{SliderBase, SliderTrack},
        Border, Clickable, Container, Grid, GridLength, Null, Stepper, Text, TextInput, ValidationResult,
        WidgetWrapper,
    },
    Color, GpuFrameCtx, PointerEventKind, UnitExt, WidgetExt,
};
use anyhow::Error;
use euclid::SideOffsets2D;
use kyute_common::{Length, SideOffsets};
use kyute_shell::text::FormattedText;
use lazy_static::lazy_static;
use palette::{FromColor, Hsv, Hsva, LinSrgba, Mix, RgbHue, Srgb, Srgba};
use skia_safe as sk;
use threadbound::ThreadBound;

////////////////////////////////////////////////////////////////////////////////////////////////////
// Shaders
////////////////////////////////////////////////////////////////////////////////////////////////////

// FIXME we assume that the target is linear sRGB, thus the conversion in the shader.
// I can't find a way to tell skia that this shader is outputting values in nonlinear sRGB.
// SkImage has `reinterpretColorSpace`, but for some reason `SkRuntimeEffect::makeImage` takes
// a `GrRecordingContext` and I don't have that close by.
//
// Also, the color space passed to `sk::Paint::new()` seems to only affect the color passed in parameter,
// and not the shader.
//
// TODO: recent versions of skia have "fromLinearSrgb/toLinearSrgb", use that when it hits skia_safe
const HSV_COLOR_SQUARE_SKSL: &str = r#"
uniform float hue;
uniform float2 size;

float3 hsv2rgb(float3 c) {
    float4 K = float4(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
    float3 p = abs(fract(c.xxx + K.xyz) * 6.0 - K.www);
    return c.z * mix(K.xxx, clamp(p - K.xxx, 0.0, 1.0), c.y);
}

half4 main(float2 fragcoord) {
    float2 pos = fragcoord / size;
    float3 rgb = hsv2rgb(float3(hue, pos.x, 1.0-pos.y));
    rgb = pow(rgb, float3(2.2));
    return half4(rgb, 1.0);
}
"#;

const COLOR_BAR_PAINT_SKSL: &str = r#"
uniform float4 from;
uniform float4 to;
uniform float2 size;
uniform int cbSize;
uniform int encoding;
layout(color) uniform float3 cbColor;

float3 checkerboard(float2 fragcoord) {
    float2 p = floor(fragcoord / float(cbSize));
    return mix(float3(1.0), cbColor, mod(p.x + p.y, 2.0));
}

float3 hsv2rgb(float3 c) {
    float4 K = float4(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
    float3 p = abs(fract(c.xxx + K.xyz) * 6.0 - K.www);
    return c.z * mix(K.xxx, clamp(p - K.xxx, 0.0, 1.0), c.y);
}

float4 main(float2 fragcoord) {
    float2 pos = fragcoord / size;
    float4 color = mix(from, to, pos.x);

    if (encoding == 1) {
        color.rgb = hsv2rgb(color.rgb);
    }

    if (cbSize > 0) {
        color.rgb = mix(checkerboard(fragcoord), color.rgb, color.a);
        color.a = 1.0;
    }
    return color;
}
"#;

const COLOR_SWATCH_PAINT_SKSL: &str = r#"
layout(color) uniform float4 color;
layout(color) uniform float3 cbColor;
uniform int cbSize;

float3 checkerboard(float2 fragcoord) {
    float2 p = floor(fragcoord / float(cbSize));
    return mix(float3(1.0), cbColor, mod(p.x + p.y, 2.0));
}

float4 main(float2 fragcoord) {
    float4 final = color;
    if (cbSize > 0) {
        final.rgb = mix(checkerboard(fragcoord), final.rgb, final.a);
        final.a = 1.0;
    }
    return final;
}
"#;

lazy_static! {
    static ref COLOR_BAR_PAINT_EFFECT: ThreadBound<sk::RuntimeEffect> =
        ThreadBound::new(sk::RuntimeEffect::make_for_shader(COLOR_BAR_PAINT_SKSL, None).unwrap());
    static ref COLOR_SWATCH_PAINT_EFFECT: ThreadBound<sk::RuntimeEffect> =
        ThreadBound::new(sk::RuntimeEffect::make_for_shader(COLOR_SWATCH_PAINT_SKSL, None).unwrap());
    static ref HSV_COLOR_SQUARE_EFFECT: ThreadBound<sk::RuntimeEffect> =
        ThreadBound::new(sk::RuntimeEffect::make_for_shader(HSV_COLOR_SQUARE_SKSL, None).unwrap());
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// Helpers
////////////////////////////////////////////////////////////////////////////////////////////////////

/// Formats & parses a color in hex notation in a text edit (e.g. #RRGGBB)
struct HexColorFormatter;

impl crate::widget::text_edit::Formatter<Color> for HexColorFormatter {
    fn format(&self, value: &Color) -> FormattedText {
        value.to_hex().into()
    }

    fn format_partial_input(&self, text: &str) -> FormattedText {
        text.into()
    }

    fn validate_partial_input(&self, text: &str) -> ValidationResult {
        if Color::try_from_hex(text).is_ok() {
            ValidationResult::Valid
        } else {
            ValidationResult::Incomplete
        }
    }

    fn parse(&self, text: &str) -> Result<Color, Error> {
        Ok(Color::try_from_hex(text)?)
    }
}

#[repr(i32)]
#[derive(Copy, Clone, Debug)]
enum ColorEncoding {
    Rgb = 0,
    Hsv = 1,
}

fn make_color_bar_paint(
    encoding: ColorEncoding,
    from: [f32; 4],
    to: [f32; 4],
    size: Size,
    checkerboard_size: i32,
    checkerboard_color: Color,
) -> Paint {
    let effect = COLOR_BAR_PAINT_EFFECT.get_ref().unwrap();
    let (cbr, cbg, cbb, cba) = checkerboard_color.to_rgba();
    let uniforms = make_uniform_data!([effect]
        from:     [f32; 4] = from;
        to:       [f32; 4] = to;
        size:     [f32; 2] = [size.width as f32, size.height as f32];
        cbSize:   i32      = checkerboard_size;
        cbColor:  [f32; 3] = [cbr, cbg, cbb];
        encoding: i32      = encoding as i32;
    );

    Paint::Shader {
        effect: effect.clone(),
        uniforms,
    }
}

fn make_color_swatch_paint(color: Color, checkerboard_size: i32, checkerboard_color: Color) -> Paint {
    let effect = COLOR_SWATCH_PAINT_EFFECT.get_ref().unwrap();
    let (cbr, cbg, cbb, cba) = checkerboard_color.to_rgba();
    let (r, g, b, a) = color.to_rgba();
    let uniforms = make_uniform_data!([effect]
        color:    [f32; 4] = [r,g,b,a];
        cbSize:   i32      = checkerboard_size;
        cbColor:  [f32; 3] = [cbr, cbg, cbb];
    );
    Paint::Shader {
        effect: effect.clone(),
        uniforms,
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// ColorPicker
////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct ColorPaletteItem<'a> {
    name: &'a str,
    value: Color,
}

#[derive(Copy, Clone, Debug)]
pub enum ColorPickerMode {
    RgbSliders,
    HsvSliders,
    HsvWheel,
}

/// Color picker parameters.
pub struct ColorPickerParams<'a> {
    /// Enable alpha slider.
    pub enable_alpha: bool,
    /// Color palette.
    pub palette: Option<&'a [ColorPaletteItem<'a>]>,
    pub enable_hex_input: bool,
}

#[derive(WidgetWrapper)]
pub struct ColorPicker {
    grid: Grid,
    color_changed: Option<Color>,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum ColorComponent {
    R,
    G,
    B,
    A,
}

/*#[composable]
fn color_component_slider(
    component: ColorComponent,
    color: &mut Color,
) -> (impl grid::Insertable, impl grid::Insertable, impl grid::Insertable) {
    let label;
    let slider;
    let text_input;

    match component {
        ColorComponent::R => {
            label = "R";
            slider = ColorSlider::rgb(r, LinSrgba::new(0.0, g, b, 1.0), LinSrgba::new(1.0, g, b, 1.0))
                .on_value_changed(|v| r = v);
            text_input = TextInput::number(r as f64).on_value_changed(|v| r = v as f32);
        }
        ColorComponent::G => {
            label = "G";
            slider = ColorSlider::rgb(g, LinSrgba::new(r, 0.0, b, 1.0), LinSrgba::new(r, 1.0, b, 1.0))
                .on_value_changed(|v| g = v);
            text_input = TextInput::number(g as f64).on_value_changed(|v| g = v as f32);
        }
        ColorComponent::B => {
            label = "B";
            slider = ColorSlider::rgb(b, LinSrgba::new(r, g, 0.0, 1.0), LinSrgba::new(r, g, 1.0, 1.0))
                .on_value_changed(|v| b = v);
            text_input = TextInput::number(b as f64).on_value_changed(|v| b = v as f32);
        }
        ColorComponent::A => {
            label = "A";
            slider = ColorSlider::rgb(a, LinSrgba::new(r, g, b, 0.0), LinSrgba::new(r, g, b, 1.0))
                .on_value_changed(|v| a = v);
            text_input = TextInput::number(a as f64).on_value_changed(|v| a = v as f32);
        }
    };

    let label = Text::new(label).aligned(Alignment::CENTER_RIGHT).grid_row("label");
    (label, slider, text_input)
}*/

impl ColorPicker {
    #[composable]
    pub fn new(color: Color, params: &ColorPickerParams) -> ColorPicker {
        let mut grid = Grid::new();

        grid = Grid::with_template(tweak!("auto / [label] 20 [slider] 300 50 80 / 1 4"));
        grid.set_align_items(AlignItems::Center);

        let mut new_color = color;
        let (mut r, mut g, mut b, mut a) = color.0.into_linear().into_components();

        grid.insert((
            ////////////////////////////////////
            Text::new("R").aligned(Alignment::CENTER_RIGHT).grid_column(0),
            ColorSlider::rgb(r, LinSrgba::new(0.0, g, b, 1.0), LinSrgba::new(1.0, g, b, 1.0))
                .on_value_changed(|v| r = v),
            TextInput::number(r as f64).on_value_changed(|v| r = v as f32),
            ////////////////////////////////////
            Text::new("G").aligned(Alignment::CENTER_RIGHT).grid_column(0),
            ColorSlider::rgb(g, LinSrgba::new(r, 0.0, b, 1.0), LinSrgba::new(r, 1.0, b, 1.0))
                .on_value_changed(|v| g = v),
            TextInput::number(g as f64).on_value_changed(|v| g = v as f32),
            ////////////////////////////////////
            Text::new("B").aligned(Alignment::CENTER_RIGHT).grid_column(0),
            ColorSlider::rgb(b, LinSrgba::new(r, g, 0.0, 1.0), LinSrgba::new(r, g, 1.0, 1.0))
                .on_value_changed(|v| b = v),
            TextInput::number(b as f64).on_value_changed(|v| b = v as f32),
        ));

        if params.enable_alpha {
            grid.insert((
                Text::new("A").aligned(Alignment::CENTER_RIGHT).grid_column(0),
                ColorSlider::rgb(a, LinSrgba::new(r, g, b, 0.0), LinSrgba::new(r, g, b, 1.0))
                    .on_value_changed(|v| a = v),
                TextInput::number(a as f64).on_value_changed(|v| a = v as f32),
            ));
        }

        grid.insert(
            Stepper::new(((a - 0.5) * 20.0) as i32, -10i32, 10i32, 1)
                .on_value_changed(|v| a = (v as f32 + 10.0) / 20.0)
                .grid_area((4..5, 0..3)),
        );

        new_color = Color(LinSrgba::new(r, g, b, a).into_encoding());

        grid.insert(ColorSwatch::new(100.percent(), 100.percent(), new_color).grid_area((0..4, 3..4)));

        if params.enable_hex_input {
            let hex_input = TextInput::new(new_color, HexColorFormatter).on_value_changed(|c| new_color = c);
            grid.insert(hex_input.grid_area((4..5, 3..4)));
        }

        let color_changed = if new_color != color { Some(new_color) } else { None };
        ColorPicker { grid, color_changed }
    }

    pub fn on_color_changed(self, f: impl FnOnce(Color)) -> Self {
        if let Some(color) = self.color_changed {
            f(color)
        }
        self
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// HsvColorSquare
////////////////////////////////////////////////////////////////////////////////////////////////////

///
pub struct HsvColorSquare {
    hue: f32,
}

impl HsvColorSquare {
    pub fn new(hue: f32) -> HsvColorSquare {
        HsvColorSquare { hue }
    }
}

impl Widget for HsvColorSquare {
    fn widget_id(&self) -> Option<WidgetId> {
        None
    }

    fn layout(&self, _ctx: &mut LayoutCtx, constraints: BoxConstraints, _env: &Environment) -> Measurements {
        Measurements::new(constraints.max)
    }

    fn event(&self, _ctx: &mut EventCtx, _event: &mut Event, _env: &Environment) {}

    fn paint(&self, ctx: &mut PaintCtx) {
        let mut bounds = ctx.bounds;
        let size = bounds.size;
        let effect = HSV_COLOR_SQUARE_EFFECT.get_ref().unwrap();
        let uniforms = make_uniform_data!([effect]
            hue: f32 = self.hue;
            size: [f32; 2] = [size.width as f32, size.height as f32];
        );
        let paint = Paint::Shader {
            effect: effect.clone(),
            uniforms,
        };
        let style = BoxStyle::new().fill(paint);
        ctx.draw_styled_box(bounds, &style);
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// HsvColorSquare
////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(WidgetWrapper)]
pub struct ColorSwatch {
    inner: Container<Null>,
}

impl ColorSwatch {
    #[composable]
    pub fn new(width: Length, height: Length, color: Color) -> ColorSwatch {
        let inner = Container::new(Null).fixed_width(width).fixed_height(height).box_style(
            BoxStyle::new()
                .fill(make_color_swatch_paint(color, 8, theme::palette::GREY_400))
                .border(style::Border::around(2.px()).paint(Color::from_hex("#000000")))
                .border(style::Border::around(1.px()).paint(Color::from_hex("#DDDDDD"))),
        );
        ColorSwatch { inner }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// ColorBar
////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Copy, Clone, Debug)]
pub enum ColorBarBounds {
    Hsv { from: Hsva, to: Hsva },
    Rgb { from: LinSrgba, to: LinSrgba },
}

impl ColorBarBounds {
    pub fn sample(&self, factor: f32) -> Color {
        match self {
            ColorBarBounds::Hsv { from, to } => {
                let hue =
                    RgbHue::from_degrees((1.0 - factor) * from.hue.to_raw_degrees() + factor * to.hue.to_raw_degrees());
                let saturation = (1.0 - factor) * from.saturation + factor * to.saturation;
                let value = (1.0 - factor) * from.value + factor * to.value;
                let alpha = (1.0 - factor) * from.alpha + factor * to.alpha;
                Color(Srgba::from_color(Hsva {
                    color: Hsv::new(hue, saturation, value),
                    alpha,
                }))
            }
            ColorBarBounds::Rgb { from, to } => Color(from.mix(&to, factor).into_encoding()),
        }
    }
}

// TODO eventually replace by a more generic "ColorGradientBar"
pub struct ColorBar {
    color_bounds: ColorBarBounds,
}

impl ColorBar {
    pub fn new(color_bounds: ColorBarBounds) -> ColorBar {
        ColorBar { color_bounds }
    }

    pub fn rgb(from: LinSrgba, to: LinSrgba) -> ColorBar {
        ColorBar::new(ColorBarBounds::Rgb { from, to })
    }

    pub fn hsv(from: Hsva, to: Hsva) -> ColorBar {
        ColorBar::new(ColorBarBounds::Hsv { from, to })
    }
}

//const COLOR_BAR_HEIGHT_DIP: f64 = 12.0;
const COLOR_BAR_KNOB_SIZE: f64 = 12.0;
const COLOR_BAR_SLIDER_SIZE_DIP: f64 = 8.0;

impl Widget for ColorBar {
    fn widget_id(&self) -> Option<WidgetId> {
        None
    }

    fn layout(&self, _ctx: &mut LayoutCtx, constraints: BoxConstraints, _env: &Environment) -> Measurements {
        let size = Size::new(
            constraints.finite_max_width().unwrap_or(300.0),
            COLOR_BAR_SLIDER_SIZE_DIP,
        );
        Measurements::new(size)
    }

    fn event(&self, _ctx: &mut EventCtx, _event: &mut Event, _env: &Environment) {}

    fn paint(&self, ctx: &mut PaintCtx) {
        let bounds = ctx.bounds;
        // paint bar
        let size = bounds.size;
        let paint = match self.color_bounds {
            ColorBarBounds::Hsv { from, to } => {
                let (from_hue, from_sat, from_val, from_alpha) = from.into_components();
                let (to_hue, to_sat, to_val, to_alpha) = to.into_components();
                make_color_bar_paint(
                    ColorEncoding::Hsv,
                    [from_hue.to_raw_degrees() / 360.0, from_sat, from_val, from_alpha],
                    [to_hue.to_raw_degrees() / 360.0, to_sat, to_val, to_alpha],
                    size,
                    (8.0 * ctx.scale_factor).round() as i32,
                    Color::new(0.7, 0.7, 0.7, 1.0),
                )
            }
            ColorBarBounds::Rgb { from, to } => {
                let (from_r, from_g, from_b, from_a) = from.into_components();
                let (to_r, to_g, to_b, to_a) = to.into_components();
                make_color_bar_paint(
                    ColorEncoding::Rgb,
                    [from_r, from_g, from_b, from_a],
                    [to_r, to_g, to_b, to_a],
                    size,
                    (8.0 * ctx.scale_factor).round() as i32,
                    Color::new(0.7, 0.7, 0.7, 1.0),
                )
            }
        };

        let style = BoxStyle::new()
            .radius(0.5 * COLOR_BAR_SLIDER_SIZE_DIP.dip())
            .fill(paint)
            .border(style::Border::outside(1.px()).paint(theme::palette::GREY_500));
        ctx.draw_styled_box(
            bounds.inner_rect(SideOffsets2D::new_all_same(1.0 / ctx.scale_factor)),
            &style,
        );
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// ColorSlider
////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(WidgetWrapper)]
pub struct ColorSlider {
    slider: SliderBase,
}

impl ColorSlider {
    #[composable]
    pub fn new(val: f32, bounds: ColorBarBounds) -> ColorSlider {
        let color = bounds.sample(val);
        let slider = SliderBase::new(
            val as f64,
            ColorBar::new(bounds),
            Container::new(Null)
                .fixed_width(COLOR_BAR_KNOB_SIZE.dip())
                .fixed_height(COLOR_BAR_KNOB_SIZE.dip())
                .box_style(
                    BoxStyle::new()
                        .radius(12.dip())
                        .fill(color)
                        .border(style::Border::inside(1.px()).paint(theme::palette::GREY_50)),
                ),
        );
        ColorSlider { slider }
    }

    pub fn on_value_changed(self, f: impl FnOnce(f32)) -> Self {
        if let Some(val) = self.slider.position_changed() {
            f(val as f32)
        }
        self
    }

    #[composable]
    pub fn rgb(val: f32, from: LinSrgba, to: LinSrgba) -> ColorSlider {
        ColorSlider::new(val, ColorBarBounds::Rgb { from, to })
    }

    #[composable]
    pub fn hsv(val: f32, from: Hsva, to: Hsva) -> ColorSlider {
        ColorSlider::new(val, ColorBarBounds::Hsv { from, to })
    }
}
