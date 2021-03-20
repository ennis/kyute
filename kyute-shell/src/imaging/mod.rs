//! Image file I/O (loading and decoding).

use crate::{
    drawing::{Bitmap, DrawContext},
    error::{check_hr, Result},
    platform::Platform,
};
use std::{
    path::{Path, PathBuf},
    ptr,
};
use thiserror::Error;
use winapi::{
    shared::winerror::HRESULT,
    um::{d2d1::*, d2d1_1::*, wincodec::*, winnt::GENERIC_READ},
};
use wio::{com::ComPtr, wide::ToWide};

#[derive(Error, Debug)]
pub enum ImagingError {
    #[error("could not decode image `{path:?}`: {hr}")]
    DecoderError { path: PathBuf, hr: HRESULT },
}

fn load_bitmap_from_file_internal(
    platform: &Platform,
    draw_ctx: &DrawContext,
    path: &Path,
) -> Result<Bitmap> {
    let wic = &platform.0.wic_factory;
    unsafe {
        let mut decoder: *mut IWICBitmapDecoder = ptr::null_mut();
        let wide_uri = path.to_wide_null();
        let hr = wic.CreateDecoderFromFilename(
            wide_uri.as_ptr(),
            ptr::null_mut(),
            GENERIC_READ,
            WICDecodeMetadataCacheOnLoad,
            &mut decoder,
        );
        check_hr(hr)?;
        let decoder = ComPtr::from_raw(decoder);

        let mut source: *mut IWICBitmapFrameDecode = ptr::null_mut();
        let hr = decoder.GetFrame(0, &mut source);
        check_hr(hr)?;
        let source = ComPtr::from_raw(source);

        let mut converter: *mut IWICFormatConverter = ptr::null_mut();
        let hr = wic.CreateFormatConverter(&mut converter);
        check_hr(hr)?;
        let converter = ComPtr::from_raw(converter);

        let hr = converter.Initialize(
            source.as_raw().cast(),
            &GUID_WICPixelFormat32bppPBGRA,
            WICBitmapDitherTypeNone,
            ptr::null_mut(),
            0.0,
            WICBitmapPaletteTypeMedianCut,
        );
        check_hr(hr)?;

        let mut bitmap: *mut ID2D1Bitmap1 = ptr::null_mut();
        let hr = draw_ctx.ctx.CreateBitmapFromWicBitmap(
            converter.as_raw().cast(),
            ptr::null_mut(),
            &mut bitmap,
        );
        check_hr(hr)?;
        let bitmap = ComPtr::from_raw(bitmap);

        Ok(Bitmap(bitmap))
    }
}

/// Loads a bitmap from a file for use with the specified draw context.
pub fn load_bitmap_from_file<P: AsRef<Path>>(
    platform: &Platform,
    draw_ctx: &DrawContext,
    path: P,
) -> Result<Bitmap> {
    load_bitmap_from_file_internal(platform, draw_ctx, path.as_ref())
}
