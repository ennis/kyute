use crate::backend;
use kyute_common::{Rect, SizeI, Transform};
use std::any::Any;

/// A compositing layer.
#[derive(Clone)]
pub struct Layer(pub(crate) backend::Layer);

impl Layer {
    pub fn new() -> Layer {
        Layer(backend::Layer::new())
    }

    /// Acquires a surface that can be drawn onto.
    pub fn acquire_surface(&self) -> Surface {
        Surface(self.0.acquire_surface())
    }

    /// Sets the transform of this layer.
    pub fn set_transform(&self, transform: &Transform) {
        self.0.set_transform(transform)
    }

    /// Adds a child layer.
    pub fn add_child(&self, layer: &Layer) {
        self.0.add_child(&layer.0)
    }

    /// Removes all child layers.
    pub fn remove_all_children(&self) {
        self.0.remove_all_children()
    }

    /// Removes the specified layer.
    pub fn remove_child(&self, layer: &Layer) {
        self.0.remove_child(&layer.0)
    }

    /// Sets the pixel size of this layer.
    ///
    /// # Panics
    ///
    /// Any surface object returned by `acquire_surface` must be dropped prior to calling this function
    /// or it will panic.
    pub fn set_size(&self, size: SizeI) {
        self.0.set_size(size);
    }

    /// Returns the pixel size of this layer.
    pub fn size(&self) -> SizeI {
        self.0.size()
    }
}

/// Drawing surface returned by `Layer::acquire_surface`.
pub struct Surface(backend::Surface);

impl Surface {
    pub fn image_info(&self) -> graal::ImageInfo {
        self.0.image_info()
    }

    pub fn size(&self) -> SizeI {
        self.0.size()
    }
}
