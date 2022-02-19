//! Wrapper around skia images.
use crate::{asset::Asset, drawing::ToSkia};
use skia_safe as sk;
use std::{io, io::Read};

#[derive(Clone, Debug)]
pub struct Image(skia_safe::Image);


impl ToSkia for Image {
    type Target = skia_safe::Image;

    fn to_skia(&self) -> Self::Target {
        self.0.clone()
    }
}

impl Asset for Image {
    type LoadError = io::Error;

    fn load(reader: &mut dyn Read) -> Result<Self, Self::LoadError> {
        let mut data = vec![];
        reader.read_to_end(&mut data);
        Self::load_from_bytes(&data)
    }

    fn load_from_bytes(bytes: &[u8]) -> Result<Self, Self::LoadError> {
        unsafe {
            // There used to be a public `DecodeToRaster` API that could take a void* but it was removed because it was "unused"
            // (how the fuck can you just declare that a _public API_ is "unused"?)
            let sk_data = skia_safe::Data::new_bytes(&bytes);
            let sk_image = skia_safe::Image::from_encoded(sk_data)
                .unwrap()
                .new_raster_image() // must call to force decoding and release
                .unwrap();
            Ok(Image(sk_image))
        }
    }
}
