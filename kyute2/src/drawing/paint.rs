//! Description of paints.
use kurbo::Vec2;
use skia_safe as sk;
use skia_safe::gradient_shader::GradientShaderColors;
use tracing::warn;

use crate::{
    drawing::{Image, ToSkia},
    Color, Data, Rect,
};

/// Image repeat mode.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum RepeatMode {
    Repeat,
    NoRepeat,
}

/// Data passed to uniforms.
#[derive(Clone, Debug)]
pub struct UniformData(pub sk::Data);

impl Data for UniformData {
    fn same(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

#[macro_export]
macro_rules! make_uniform_data {
    ( [$effect:ident] $($name:ident : $typ:ty = $value:expr),*) => {
        unsafe {
            let total_size = $effect.uniform_size();
            let mut data: Vec<u8> = Vec::with_capacity(total_size);
            let ptr = data.as_mut_ptr();

            $(
            {
                let (u_offset, u_size) = $effect
                    .uniforms()
                    .iter()
                    .find_map(|u| {
                        if u.name() == std::stringify!($name) {
                            Some((u.offset(), u.size_in_bytes()))
                        } else {
                            None
                        }
                    })
                    .expect("could not find uniform");

                let v : $typ = $value;
                assert_eq!(std::mem::size_of::<$typ>(), u_size);
                std::ptr::write(ptr.add(u_offset).cast::<$typ>(), v);
            }
            )*

            data.set_len(total_size);
            $crate::drawing::UniformData(skia_safe::Data::new_copy(&data))
        }
    };
}

#[macro_export]
macro_rules! shader {
    ($source:literal) => {{
        thread_local! {
            static SHADER: std::cell::OnceCell<$crate::skia::RuntimeEffect> = OnceCell::new();
        }
        SHADER.with(|cell| {
            cell.get_or_init(|| {
                $crate::skia::RuntimeEffect::make_for_shader($source, None).expect("failed to compile shader")
            })
            .clone()
        })

        /*static SHADER: std::sync::OnceLock<$crate::ThreadBound<$crate::skia::RuntimeEffect>> =
            std::sync::OnceLock::new();
        SHADER
            .get_or_init(|| {
                $crate::skia::RuntimeEffect::make_for_shader($source, None).expect("failed to compile shader")
            })
            .get_ref()
            .expect("shader accessed from another thread")*/
    }};
}

#[macro_export]
macro_rules! shader_paint {
    ($source:literal, $($name:ident : $typ:ty = $value:expr),*) => {
        {
            let shader = $crate::shader!($source).clone();
            let uniforms = $crate::make_uniform_data!([shader] $($name : $typ = $value),*);
            $crate::drawing::Paint::Shader { effect: shader, uniforms }
        }
    };
}

/*
fn compare_runtime_effects(left: &sk::RuntimeEffect, right: &sk::RuntimeEffect) -> bool {
    // FIXME: skia_safe doesn't let us access the native pointer for some reason,
    // so force our way though
    //left.native() as *const _ == right.native() as *const _
    unsafe {
        let ptr_a: *const c_void = mem::transmute_copy(left);
        let ptr_b: *const c_void = mem::transmute_copy(right);
        ptr_a == ptr_b
    }
}
*/

/// Paint.
#[derive(Clone, Debug)]
//#[serde(tag = "type")]
pub enum Paint {
    //#[serde(rename = "color")]
    Color(Color),
    //#[serde(rename = "linear-gradient")]
    LinearGradient(LinearGradient),
    //#[serde(rename = "image")]
    Image {
        // FIXME: can't deserialize here
        image: Image,
        repeat_x: RepeatMode,
        repeat_y: RepeatMode,
    },
    // TODO: shader effects
    Shader {
        // GOD FCKING DAMMIT MAKE THIS THREAD-SAFE ALREADY
        effect: sk::RuntimeEffect,
        uniforms: UniformData,
    },
}

impl PartialEq for Paint {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Paint::Color(a), Paint::Color(b)) => a == b,
            (Paint::LinearGradient(a), Paint::LinearGradient(b)) => a == b,
            (Paint::Image { .. }, Paint::Image { .. }) => {
                // TODO
                false
            }
            (Paint::Shader { .. }, Paint::Shader { .. }) => {
                // TODO
                false
            }
            _ => false,
        }
    }
}

// Nope, not thread safe.
//impl_env_value!(Paint);

// TODO: move gradient stuff to a specific module

impl Default for Paint {
    fn default() -> Self {
        Paint::Color(Color::new(0.0, 0.0, 0.0, 0.0))
    }
}

impl Paint {
    pub fn is_transparent(&self) -> bool {
        if let Paint::Color(color) = self {
            color.alpha() == 0.0
        } else {
            false
        }
    }

    /// Converts this object to a skia `SkPaint`.
    pub fn to_sk_paint(&self, bounds: Rect) -> sk::Paint {
        match self {
            Paint::Color(color) => {
                let mut paint = sk::Paint::new(color.to_skia(), None);
                paint.set_anti_alias(true);
                paint
            }
            Paint::LinearGradient(linear_gradient) => {
                let c = bounds.center();
                let w = bounds.width();
                let h = bounds.height();

                let angle = linear_gradient.angle;
                let tan_th = angle.tan();
                let (x, y) = if tan_th > h / w {
                    (h / (2.0 * tan_th), 0.5 * h)
                } else {
                    (0.5 * w, 0.5 * w * tan_th)
                };

                let a = c + Vec2::new(-x, y);
                let b = c + Vec2::new(x, -y);
                let a = a.to_skia();
                let b = b.to_skia();

                let mut resolved_gradient = linear_gradient.clone();
                resolved_gradient.resolve_stop_positions();

                let positions: Vec<_> = resolved_gradient
                    .stops
                    .iter()
                    .map(|stop| stop.position.unwrap() as f32)
                    .collect();
                let colors: Vec<_> = resolved_gradient
                    .stops
                    .iter()
                    .map(|stop| stop.color.to_skia())
                    .collect();

                let shader = sk::Shader::linear_gradient(
                    (a, b),
                    GradientShaderColors::ColorsInSpace(&colors, Some(sk::ColorSpace::new_srgb())),
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
                image,
                repeat_x,
                repeat_y,
            } => {
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
            }
            Paint::Shader { effect, uniforms } => {
                let shader = effect
                    .make_shader(&uniforms.0, &[], None)
                    .expect("failed to create shader");
                let mut paint = sk::Paint::default();
                paint.set_shader(shader);
                paint
            }
        }
    }

    pub fn image(_uri: &str, _repeat_x: RepeatMode, _repeat_y: RepeatMode) -> Paint {
        // TODO: call outside of composition?
        todo!()
        /*let image_cache = cache::environment().get(&IMAGE_CACHE).unwrap();
        if let Ok(image) = image_cache.load(uri) {
            Paint::Image {
                image,
                repeat_x,
                repeat_y,
            }
        } else {
            Paint::Color(Default::default())
        }*/
    }
}

impl From<Color> for Paint {
    fn from(color: Color) -> Self {
        Paint::Color(color)
    }
}

/*fn deserialize_angle<'de, D>(d: D) -> Result<Angle, D::Error>
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
}*/

/// Represents a gradient stop.
#[derive(Clone, Debug, PartialEq)]
pub struct ColorStop {
    /// Position of the stop along the gradient segment, normalized between zero and one.
    ///
    /// If `None`, the position is inferred from the position of the surrounding stops.
    pub position: Option<f64>,
    /// Stop color.
    pub color: Color,
}

/// Describes a linear color gradient.
#[derive(Clone, Debug, PartialEq)]
pub struct LinearGradient {
    /// Direction of the gradient line.
    //#[serde(deserialize_with = "deserialize_angle")]
    pub angle: f64,
    /// List of color stops.
    pub stops: Vec<ColorStop>,
}

impl Data for LinearGradient {
    fn same(&self, other: &Self) -> bool {
        self == other
    }
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
    pub fn angle(mut self, angle: f64) -> Self {
        self.angle = angle;
        self
    }

    /// Appends a color stop to this gradient.
    pub fn stop(mut self, color: Color, position: impl Into<Option<f64>>) -> Self {
        self.stops.push(ColorStop {
            color,
            position: position.into(),
        });
        self
    }

    /// Resolves color stop positions.
    ///
    /// See https://www.w3.org/TR/css-images-3/#color-stop-fixup
    pub(crate) fn resolve_stop_positions(&mut self) {
        if self.stops.len() < 2 {
            warn!("invalid gradient (must have at least two stops)");
            return;
        }

        // CSS Images Module Level 3 - 3.4.3. Color Stop “Fixup”
        //
        //      If the first color stop does not have a position, set its position to 0%.
        //      If the last color stop does not have a position, set its position to 100%.
        //
        self.stops.first_mut().unwrap().position.get_or_insert(0.0);
        self.stops.last_mut().unwrap().position.get_or_insert(1.0);

        //
        //      If a color stop or transition hint has a position that is less than the specified position
        //      of any color stop or transition hint before it in the list, set its position to be equal
        //      to the largest specified position of any color stop or transition hint before it.
        //
        let mut cur_pos = self.stops.first().unwrap().position.unwrap();
        for stop in self.stops.iter_mut() {
            if let Some(mut pos) = stop.position {
                if pos < cur_pos {
                    pos = cur_pos;
                }
                cur_pos = pos;
            }
        }

        //
        //      If any color stop still does not have a position, then, for each run of adjacent color stops without positions,
        //      set their positions so that they are evenly spaced between the preceding and following color stops with positions.
        //
        let mut i = 0;
        while i < self.stops.len() {
            if self.stops[i].position.is_none() {
                let mut j = i + 1;
                while self.stops[j].position.is_none() {
                    j += 1;
                }
                let len = j - i + 1;
                let a = self.stops[i - 1].position.unwrap();
                let b = self.stops[j].position.unwrap();
                for k in i..j {
                    self.stops[i].position = Some(a + (b - a) * (k - i + 1) as f64 / len as f64);
                }
                i = j;
            } else {
                i += 1;
            }
        }
    }
}

impl Default for LinearGradient {
    fn default() -> Self {
        Self::new()
    }
}

impl From<LinearGradient> for Paint {
    fn from(g: LinearGradient) -> Self {
        Paint::LinearGradient(g)
    }
}

/*/// From CSS value.
impl TryFrom<&str> for Paint {
    type Error = ();
    fn try_from(css: &str) -> Result<Self, ()> {
        Paint::parse(css).map_err(|_| ())
    }
}*/
