use lazy_static::lazy_static;
use std::ops::Deref;
use windows::{
    core::Interface,
    Win32::Graphics::DirectWrite::{DWriteCreateFactory, IDWriteFactory7, DWRITE_FACTORY_TYPE_SHARED},
};

#[derive(Clone)]
pub(crate) struct DWriteFactory(pub(crate) IDWriteFactory7);
unsafe impl Sync for DWriteFactory {} // ok to send &I across threads
unsafe impl Send for DWriteFactory {} // ok to send I across threads
impl Deref for DWriteFactory {
    type Target = IDWriteFactory7;
    fn deref(&self) -> &IDWriteFactory7 {
        &self.0
    }
}

lazy_static! {
    static ref DWRITE_FACTORY: DWriteFactory = create_dwrite_factory();
}

fn create_dwrite_factory() -> DWriteFactory {
    unsafe {
        let factory = DWriteCreateFactory(DWRITE_FACTORY_TYPE_SHARED, &IDWriteFactory7::IID)
            .unwrap()
            .cast::<IDWriteFactory7>()
            .unwrap();
        DWriteFactory(factory)
    }
}

pub(crate) fn dwrite_factory() -> &'static DWriteFactory {
    &*DWRITE_FACTORY
}
