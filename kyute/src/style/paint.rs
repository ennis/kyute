//! Description of paints.
use crate::{
    style::{Angle, ValueRef},
    Color, EnvKey, Environment, Offset, Rect,
};
use std::fmt;
use kyute_shell::drawing::ToSkia;
use kyute_shell::skia as sk;
use kyute_shell::skia::gradient_shader::GradientShaderColors;

/// Represents a gradient stop.
#[derive(Clone, Debug, serde::Deserialize)]
pub struct GradientStop {
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pos: Option<f64>,
    color: ValueRef<Color>,
}

/// Paint.
#[derive(Clone, Debug, serde::Deserialize)]
#[serde(tag = "type")]
pub enum Paint {
    #[serde(rename = "color")]
    SolidColor { color: ValueRef<Color> },
    #[serde(rename = "linear-gradient")]
    LinearGradient(LinearGradient),
    #[serde(rename = "image")]
    Image {
        // TODO
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
                                positions[i + j] =
                                    (prev + (next - prev) * j as f64 / skip as f64) as f32;
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
            Paint::Image { .. } => {
                todo!("images")
            }
        }
    }
}

impl From<Color> for Paint {
    fn from(color: Color) -> Self {
        Paint::SolidColor {
            color: ValueRef::Inline(color),
        }
    }
}

impl From<EnvKey<Color>> for Paint {
    fn from(color: EnvKey<Color>) -> Self {
        Paint::SolidColor {
            color: ValueRef::Env(color),
        }
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
    pub fn stop(mut self, color: impl Into<ValueRef<Color>>, pos: impl Into<Option<f64>>) -> Self {
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
