//! Description of paints.
use crate::{
    asset::ASSET_LOADER,
    drawing::{ToSkia, IMAGE_CACHE},
    style::ColorExpr,
    Angle, Color, EnvKey, Environment, Offset, Rect,
};
use skia_safe as sk;
use skia_safe::gradient_shader::GradientShaderColors;
use std::fmt;

/// Represents a gradient stop.
#[derive(Clone, Debug, serde::Deserialize)]
pub struct GradientStop {
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pos: Option<f64>,
    #[serde(flatten)]
    color: ColorExpr,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, serde::Deserialize)]
pub enum RepeatMode {
    Repeat,
    NoRepeat,
}

/// Paint.
#[derive(Clone, Debug, serde::Deserialize)]
#[serde(tag = "type")]
pub enum Paint {
    #[serde(rename = "color")]
    SolidColor { color: ColorExpr },
    #[serde(rename = "linear-gradient")]
    LinearGradient(LinearGradient),
    #[serde(rename = "image")]
    Image {
        uri: String,
        repeat_x: RepeatMode,
        repeat_y: RepeatMode,
    },
}

impl Paint {
    /// Converts this object to a skia `SkPaint`.
    pub fn to_sk_paint(&self, env: &Environment, bounds: Rect) -> sk::Paint {
        match self {
            Paint::SolidColor { color } => {
                let color = color.resolve(env).unwrap();
                let mut paint = sk::Paint::new(color.to_skia(), None);
                paint.set_anti_alias(true);
                paint
            }
            Paint::LinearGradient(linear_gradient) => {
                let c = bounds.center();
                let w = bounds.width();
                let h = bounds.height();

                let angle = linear_gradient.angle;
                let tan_th = angle.radians.tan();
                let (x, y) = if tan_th > h / w {
                    (h / (2.0 * tan_th), 0.5 * h)
                } else {
                    (0.5 * w, 0.5 * w * tan_th)
                };

                let a = c + Offset::new(-x, y);
                let b = c + Offset::new(x, -y);
                let a = sk::Point::new(a.x as sk::scalar, a.y as sk::scalar);
                let b = sk::Point::new(b.x as sk::scalar, b.y as sk::scalar);

                let mut positions = vec![0.0f32; linear_gradient.stops.len()];

                // resolve positions
                {
                    let mut i = 0;
                    let n = positions.len();
                    while i < n {
                        if let Some(pos) = linear_gradient.stops[i].pos {
                            positions[i] = pos as f32;
                            i += 1;
                        } else {
                            let prev = if i > 0 { positions[i - 1] as f64 } else { 0.0 };
                            // find the next non-empty stop position, return the number of stops between the current stop (i) and the next stop with non-empty position.
                            let (skip, next) = {
                                let mut skip = 1;
                                let mut pos = 1.0;
                                while i + skip <= n {
                                    if let Some(p) = linear_gradient.stops[i + skip].pos {
                                        pos = p;
                                        break;
                                    }
                                    skip += 1;
                                }
                                (skip, pos)
                            };

                            for j in 0..skip {
                                positions[i + j] = (prev + (next - prev) * j as f64 / skip as f64) as f32;
                            }
                            i += skip;
                        }
                    }
                }

                let colors: Vec<_> = linear_gradient
                    .stops
                    .iter()
                    .map(|stop| stop.color.resolve(env).unwrap().to_skia())
                    .collect();

                let shader = sk::Shader::linear_gradient(
                    (a, b),
                    GradientShaderColors::ColorsInSpace(&colors, sk::ColorSpace::new_srgb()),
                    &positions[..],
                    sk::TileMode::Clamp,
                    None,
                    None,
                )
                .unwrap();

                let mut paint = sk::Paint::default();
                paint.set_shader(shader);
                paint.set_anti_alias(true);
                paint
            }
            Paint::Image {
                uri,
                repeat_x,
                repeat_y,
            } => {
                let image_cache = env.get(IMAGE_CACHE).unwrap();
                if let Ok(image) = image_cache.load(uri) {
                    let tile_x = match *repeat_x {
                        RepeatMode::Repeat => sk::TileMode::Repeat,
                        RepeatMode::NoRepeat => sk::TileMode::Decal,
                    };
                    let tile_y = match *repeat_y {
                        RepeatMode::Repeat => sk::TileMode::Repeat,
                        RepeatMode::NoRepeat => sk::TileMode::Decal,
                    };
                    let sampling_options = sk::SamplingOptions::new(sk::FilterMode::Linear, sk::MipmapMode::None);
                    let image_shader = image
                        .to_skia()
                        .to_shader((tile_x, tile_y), sampling_options, None)
                        .unwrap();
                    let mut paint = sk::Paint::default();
                    paint.set_shader(image_shader);
                    paint
                } else {
                    sk::Paint::default()
                }
            }
        }
    }
}

impl From<Color> for Paint {
    fn from(color: Color) -> Self {
        Paint::SolidColor { color: color.into() }
    }
}

impl From<EnvKey<Color>> for Paint {
    fn from(color: EnvKey<Color>) -> Self {
        Paint::SolidColor { color: color.into() }
    }
}

fn deserialize_angle<'de, D>(d: D) -> Result<Angle, D::Error>
where
    D: serde::Deserializer<'de>,
{
    struct Visitor;

    impl<'de> serde::de::Visitor<'de> for Visitor {
        type Value = Angle;
        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("floating-point value")
        }
        fn visit_f64<E>(self, value: f64) -> Result<Angle, E>
        where
            E: serde::de::Error,
        {
            Ok(Angle::radians(value))
        }
    }

    d.deserialize_f32(Visitor)
}

/// Describes a linear color gradient.
#[derive(Clone, Debug, serde::Deserialize)]
pub struct LinearGradient {
    #[serde(deserialize_with = "deserialize_angle")]
    angle: Angle,
    stops: Vec<GradientStop>,
}

impl LinearGradient {
    /// Creates a new `LinearGradient`, with no stops.
    pub fn new() -> LinearGradient {
        LinearGradient {
            angle: Default::default(),
            stops: vec![],
        }
    }

    /// Sets the gradient angle.
    pub fn angle(mut self, angle: Angle) -> Self {
        self.angle = angle;
        self
    }

    /// Appends a color stop to this gradient.
    pub fn stop(mut self, color: impl Into<ColorExpr>, pos: impl Into<Option<f64>>) -> Self {
        self.stops.push(GradientStop {
            color: color.into(),
            pos: pos.into(),
        });
        self
    }
}

impl From<LinearGradient> for Paint {
    fn from(a: LinearGradient) -> Self {
        Paint::LinearGradient(a)
    }
}
