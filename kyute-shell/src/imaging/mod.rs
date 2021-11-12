//! Image file I/O (loading and decoding).

use crate::{
    bindings::Windows::Win32::{
        SystemServices::GENERIC_READ,
        WindowsImagingComponent::{
            GUID_WICPixelFormat32bppPBGRA, WICBitmapDitherType, WICBitmapPaletteType,
            WICDecodeOptions,
        },
    },
    drawing::{Bitmap, DrawContext},
    error::Result,
    platform::Platform,
};
use std::{path::Path, ptr};
use windows::Interface;

/*#[derive(Error, Debug)]
pub enum ImagingError {
    #[error("could not decode image `{path:?}`: {hr}")]
    DecoderError { path: PathBuf, hr: HRESULT },
}*/

fn load_bitmap_from_file_internal(draw_ctx: &DrawContext, path: &Path) -> Result<Bitmap> {
    let platform = Platform::instance();
    let wic = &platform.0.wic_factory;
    unsafe {
        let mut decoder = None;
        let decoder = wic
            .CreateDecoderFromFilename(
                path.to_str().unwrap(),
                ptr::null_mut(),
                GENERIC_READ,
                WICDecodeOptions::WICDecodeMetadataCacheOnLoad,
                &mut decoder,
            )
            .and_some(decoder)?;

        let mut source = None;
        let source = decoder.GetFrame(0, &mut source).and_some(source)?;

        let mut converter = None;
        let converter = wic
            .CreateFormatConverter(&mut converter)
            .and_some(converter)?;

        converter
            .Initialize(
                &source,
                &GUID_WICPixelFormat32bppPBGRA as *const _ as *mut _, // *mut GUID? is it an oversight?
                WICBitmapDitherType::WICBitmapDitherTypeNone,
                None,
                0.0,
                WICBitmapPaletteType::WICBitmapPaletteTypeMedianCut,
            )
            .ok()?;

        let mut bitmap = None;
        let bitmap = draw_ctx
            .ctx
            .CreateBitmapFromWicBitmap(&converter, ptr::null(), &mut bitmap)
            .and_some(bitmap)?;

        Ok(Bitmap(bitmap.cast().unwrap()))
    }
}

/// Loads a bitmap from a file for use with the specified draw context.
pub fn load_bitmap_from_file<P: AsRef<Path>>(draw_ctx: &DrawContext, path: P) -> Result<Bitmap> {
    load_bitmap_from_file_internal(draw_ctx, path.as_ref())
}
