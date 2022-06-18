#[derive(Clone, Debug)]
pub struct UniformData(pub(crate) sk::Data);

impl Data for UniformData {
    fn same(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

#[macro_export]
macro_rules! make_uniform_data {
    ( [$effect:ident] $($name:ident : $typ:ty = $value:expr;)*) => {
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
            $crate::style::UniformData(skia_safe::Data::new_copy(&data))
        }
    };
}

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

/// Paint.
#[derive(Clone, Debug, Data)]
//#[serde(tag = "type")]
pub enum Paint {
    //#[serde(rename = "color")]
    SolidColor(Color),
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
        #[data(same_fn = "compare_runtime_effects")]
        effect: sk::RuntimeEffect,
        uniforms: UniformData,
    },
}

impl_env_value!(Paint);

impl Default for Paint {
    fn default() -> Self {
        Paint::SolidColor(Color::new(0.0, 0.0, 0.0, 0.0))
    }
}

impl Paint {
    pub fn is_transparent(&self) -> bool {
        if let Paint::SolidColor(color) = self {
            color.alpha() == 0.0
        } else {
            false
        }
    }

    /// Converts this object to a skia `SkPaint`.
    pub fn to_sk_paint(&self, bounds: Rect) -> sk::Paint {
        match self {
            Paint::SolidColor(color) => {
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

    pub fn image(uri: &str, repeat_x: RepeatMode, repeat_y: RepeatMode) -> Paint {
        // TODO: call outside of composition?
        let image_cache = cache::environment().get(IMAGE_CACHE).unwrap();
        if let Ok(image) = image_cache.load(uri) {
            Paint::Image {
                image,
                repeat_x,
                repeat_y,
            }
        } else {
            Paint::SolidColor(Default::default())
        }
    }
}

impl From<Color> for Paint {
    fn from(color: Color) -> Self {
        Paint::SolidColor(color)
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
