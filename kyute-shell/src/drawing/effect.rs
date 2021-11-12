/*unsafe fn get_output_image(effect: &ComPtr<ID2D1Effect>) -> ComPtr<ID2D1Image> {
    let mut output_image = ptr::null_mut();
    effect.GetOutput(&mut output_image);
    ComPtr::from_raw(output_image)
}

#[derive(Clone)]
pub struct FloodImage {
    effect: ComPtr<ID2D1Effect>,
    output_image: ComPtr<ID2D1Image>,
}

impl FloodImage {
    pub fn new(ctx: &DrawContext, fill_color: Color) -> FloodImage {
        unsafe {
            let mut effect = ptr::null_mut();
            check_hr(ctx.ctx.CreateEffect(&CLSID_D2D1Flood, &mut effect))
                .expect("CreateEffect failed");
            let effect = ComPtr::from_raw(effect);
            let (r, g, b, a) = fill_color.into_components();
            let color_v = D2D_VECTOR_4F {
                x: r,
                y: g,
                z: b,
                w: a,
            };
            effect.SetValue(
                D2D1_FLOOD_PROP_COLOR,
                D2D1_PROPERTY_TYPE_VECTOR4,
                &color_v as *const _ as *const u8,
                mem::size_of::<D2D_VECTOR_4F>() as u32,
            );
            let output_image = get_output_image(&effect);
            FloodImage {
                effect,
                output_image,
            }
        }
    }
}

impl Image for FloodImage {
    fn as_raw_image(&self) -> *mut ID2D1Image {
        self.output_image.as_raw()
    }
}*/

/*
#[derive(Clone)]
pub struct HsvToRgb {
    effect: ComPtr<ID2D1Effect>,
    output_image: ComPtr<ID2D1Image>,
}

const D2D1_HUETORGB_PROP_INPUT_COLOR_SPACE: u32 = 0;
const D2D1_HUETORGB_INPUT_COLOR_SPACE_HUE_SATURATION_VALUE: u32 = 0;

impl HsvToRgb {
    pub fn new<I: Image>(ctx: &DrawContext, input: I) -> HsvToRgb {
        unsafe {
            let mut effect = ptr::null_mut();
            check_hr(ctx.ctx.CreateEffect(&CLSID_D2D1HueToRgb, &mut effect))
                .expect("CreateEffect failed");
            let effect = ComPtr::from_raw(effect);
            let input_img = input.as_raw_image();
            effect.SetInput(0, input_img);
            effect.SetValue(
                D2D1_HUETORGB_PROP_INPUT_COLOR_SPACE,
                D2D1_HUETORGB_INPUT_COLOR_SPACE_HUE_SATURATION_VALUE,
            );
            let output_image = get_output_image(&effect);
            HsvToRgb {
                effect,
                output_image,
            }
        }
    }
}

impl Image for HsvToRgb {
    fn as_raw_image(&self) -> *mut ID2D1Image {
        self.output_image.as_raw()
    }
}


#[derive(Clone)]
pub struct ColorMatrixEffect {

}*/
