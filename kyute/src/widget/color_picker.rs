use crate::{
    drawing::ToSkia,
    make_uniform_data, style,
    style::{BoxStyle, Paint, PaintCtxExt},
    theme,
    widget::{
        grid::GridTrackDefinition,
        prelude::*,
        slider::{SliderBase, SliderTrack},
        Border, Clickable, Container, Grid, GridLength, Null, WidgetWrapper,
    },
    Color, PointerEventKind, UnitExt, WidgetExt,
};
use euclid::SideOffsets2D;
use kyute_common::SideOffsets;
use lazy_static::lazy_static;
use palette::{FromColor, Hsv, Hsva, Mix, RgbHue, Srgb, Srgba};
use skia_safe as sk;
use skia_safe::canvas::lattice::RectType::Default;
use std::cell::Cell;
use threadbound::ThreadBound;

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

pub struct ColorPickerParams<'a> {
    /// Enable alpha slider.
    pub enable_alpha: bool,
    /// Color palette.
    pub palette: Option<&'a [ColorPaletteItem<'a>]>,
}

#[derive(Clone, WidgetWrapper)]
pub struct ColorPicker {
    grid: Grid,
    color_changed: Signal<Color>,
}

impl ColorPicker {
    #[composable]
    pub fn new(params: &ColorPickerParams) -> ColorPicker {
        let color_changed = Signal::new();
        let mut grid = Grid::new();
        grid.push_row_definition(GridTrackDefinition::new(GridLength::Fixed(150.dip())));
        grid.push_row_definition(GridTrackDefinition::new(GridLength::Fixed(30.dip())));
        grid.push_row_definition(GridTrackDefinition::new(GridLength::Fixed(30.dip())));
        grid.push_row_definition(GridTrackDefinition::new(GridLength::Fixed(30.dip())));
        grid.push_column_definition(GridTrackDefinition::new(GridLength::Fixed(300.dip())));
        let hue_square = Border::new(
            style::Border::around(1.px()).paint(theme::palette::GREY_100),
            HsvColorSquare::new(0.0),
        )
        .padding(2.dip(), 2.dip(), 2.dip(), 2.dip());
        let hue_bar = ColorSlider::hsv(
            Hsva::new(RgbHue::from_degrees(0.0), 1.0, 1.0, 1.0),
            Hsva::new(RgbHue::from_degrees(360.0), 1.0, 1.0, 1.0),
        )
        .padding(2.dip(), 2.dip(), 2.dip(), 2.dip());
        let alpha_bar = ColorSlider::rgb(Color::new(1.0, 0.1, 0.1, 1.0), Color::new(1.0, 0.1, 0.1, 0.0)).padding(
            2.dip(),
            2.dip(),
            2.dip(),
            2.dip(),
        );
        grid.add_item(0, 0, 0, hue_square);
        grid.add_item(1, 0, 0, hue_bar);
        grid.add_item(2, 0, 0, alpha_bar);
        //grid.add_item(2, 0, 0, ColorKnob::new(theme::palette::AMBER_400));
        ColorPicker { grid, color_changed }
    }
}

//--------------------------------------------------------------------------------------------------

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

lazy_static! {
    static ref HSV_COLOR_SQUARE_EFFECT: ThreadBound<sk::RuntimeEffect> =
        ThreadBound::new(sk::RuntimeEffect::make_for_shader(HSV_COLOR_SQUARE_SKSL, None).unwrap());
}

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

    fn event(&self, _ctx: &mut EventCtx, _event: &mut Event, _env: &Environment) {}

    fn layout(&self, _ctx: &mut LayoutCtx, constraints: BoxConstraints, _env: &Environment) -> Measurements {
        Measurements::new(constraints.max)
    }

    fn paint(&self, ctx: &mut PaintCtx, _env: &Environment) {
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

//--------------------------------------------------------------------------------------------------

const COLOR_BAR_PAINT_SKSL: &str = r#"
layout(color) uniform float4 from;
layout(color) uniform float4 to;
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

lazy_static! {
    static ref COLOR_BAR_PAINT_EFFECT: ThreadBound<sk::RuntimeEffect> =
        ThreadBound::new(sk::RuntimeEffect::make_for_shader(COLOR_BAR_PAINT_SKSL, None).unwrap());
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

#[derive(Copy, Clone, Debug)]
pub enum ColorBarBounds {
    Hsv { from: Hsva, to: Hsva },
    Rgb { from: Color, to: Color },
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
            ColorBarBounds::Rgb { from, to } => {
                Color(Srgba::from_color(from.0.into_linear().mix(&to.0.into_linear(), factor)))
            }
        }
    }
}

pub struct ColorBar {
    mode: ColorBarBounds,
}

impl ColorBar {
    pub fn new(mode: ColorBarBounds) -> ColorBar {
        ColorBar { mode }
    }

    pub fn rgb(from: Color, to: Color) -> ColorBar {
        ColorBar {
            mode: ColorBarBounds::Rgb { from, to },
        }
    }

    pub fn hsv(from: Hsva, to: Hsva) -> ColorBar {
        ColorBar {
            mode: ColorBarBounds::Hsv { from, to },
        }
    }
}

//const COLOR_BAR_HEIGHT_DIP: f64 = 12.0;
const COLOR_BAR_KNOB_SIZE: f64 = 12.0;
const COLOR_BAR_SLIDER_SIZE_DIP: f64 = 8.0;

impl Widget for ColorBar {
    fn widget_id(&self) -> Option<WidgetId> {
        None
    }

    fn event(&self, _ctx: &mut EventCtx, _event: &mut Event, _env: &Environment) {}

    fn layout(&self, _ctx: &mut LayoutCtx, constraints: BoxConstraints, _env: &Environment) -> Measurements {
        Measurements::new(Size::new(
            constraints.finite_max_width().unwrap_or(300.0),
            COLOR_BAR_SLIDER_SIZE_DIP,
        ))
    }

    fn paint(&self, ctx: &mut PaintCtx, _env: &Environment) {
        let bounds = ctx.bounds;

        // paint bar
        let size = bounds.size;
        let paint = match self.mode {
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
                let (from_r, from_g, from_b, from_a) = from.to_rgba();
                let (to_r, to_g, to_b, to_a) = to.to_rgba();
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

#[derive(Clone, WidgetWrapper)]
pub struct ColorSlider {
    slider: SliderBase,
}

impl ColorSlider {
    #[composable]
    pub fn new(bounds: ColorBarBounds) -> ColorSlider {
        #[state]
        let mut pos: f64 = 0.0;
        let color = bounds.sample(pos as f32);

        let slider = SliderBase::new(
            pos,
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
        )
        .on_position_changed(|p| pos = p);

        ColorSlider { slider }
    }

    #[composable]
    pub fn rgb(from: Color, to: Color) -> ColorSlider {
        ColorSlider::new(ColorBarBounds::Rgb { from, to })
    }

    #[composable]
    pub fn hsv(from: Hsva, to: Hsva) -> ColorSlider {
        ColorSlider::new(ColorBarBounds::Hsv { from, to })
    }
}
