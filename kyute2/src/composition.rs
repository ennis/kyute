//! System compositor interface
use crate::{backend, Size};
use glazier::raw_window_handle::RawWindowHandle;
use skia_safe as sk;
use slotmap::{SecondaryMap, SlotMap};
use std::cell::RefCell;

////////////////////////////////////////////////////////////////////////////////////////////////////

slotmap::new_key_type! {
    /// Unique identifier for a compositor layer.
    pub struct LayerID;
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Copy, Clone, Debug)]
struct LayerInfo {}

#[derive(Copy, Clone, Debug)]
struct TreeInfo {
    parent: Option<LayerID>,
    prev_sibling: Option<LayerID>,
    next_sibling: Option<LayerID>,
}

#[derive(Copy, Clone, Debug, Default)]
struct ContainerInfo {
    first_child: Option<LayerID>,
    last_child: Option<LayerID>,
}

#[derive(Copy, Clone, Debug)]
struct EffectInfo {
    opacity: f32,
}

#[derive(Copy, Clone, Debug)]
struct TransformInfo {
    transform: kurbo::Affine,
}

#[derive(Copy, Clone, Debug)]
struct ClipLayer {
    bounds: kurbo::Rect,
}

#[derive(Copy, Clone, Debug)]
struct SurfaceInfo {}

////////////////////////////////////////////////////////////////////////////////////////////////////

/// A drawable surface
pub struct DrawableSurface {
    backend: backend::composition::DrawableSurface,
}

impl DrawableSurface {
    /// Returns the underlying skia surface.
    pub fn surface(&self) -> sk::Surface {
        self.backend.surface()
    }
}

/// Pixel format of a drawable surface.
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug, Hash)]
pub enum ColorType {
    Alpha8,
    RGBA8888,
    BGRA8888,
    RGBA1010102,
    BGRA1010102,
    RGB101010x,
    BGR101010x,
    BGR101010xXR,
    Gray8,
    RGBAF16,
    RGBAF32,
    A16Float,
    A16UNorm,
    R16G16B16A16UNorm,
    SRGBA8888,
    R8UNorm,
}

impl ColorType {
    pub fn to_skia_color_type(&self) -> sk::ColorType {
        match *self {
            //ColorType::Unknown => sk::ColorType::Unknown,
            ColorType::Alpha8 => sk::ColorType::Alpha8,
            //ColorType::RGB565 => sk::ColorType::RGB565,
            //ColorType::ARGB4444 => sk::ColorType::ARGB4444,
            ColorType::RGBA8888 => sk::ColorType::RGBA8888,
            //ColorType::RGB888x => sk::ColorType::RGB888x,
            ColorType::BGRA8888 => sk::ColorType::BGRA8888,
            ColorType::RGBA1010102 => sk::ColorType::RGBA1010102,
            ColorType::BGRA1010102 => sk::ColorType::BGRA1010102,
            ColorType::RGB101010x => sk::ColorType::RGB101010x,
            ColorType::BGR101010x => sk::ColorType::BGR101010x,
            ColorType::BGR101010xXR => sk::ColorType::BGR101010xXR,
            ColorType::Gray8 => sk::ColorType::Gray8,
            //ColorType::RGBAF16Norm => sk::ColorType::RGBAF16Norm,
            ColorType::RGBAF16 => sk::ColorType::RGBAF16,
            ColorType::RGBAF32 => sk::ColorType::RGBAF32,
            //ColorType::R8G8UNorm => sk::ColorType::R8G8UNorm,
            ColorType::A16Float => sk::ColorType::A16Float,
            //ColorType::R16G16Float => sk::ColorType::R16G16Float,
            ColorType::A16UNorm => sk::ColorType::A16UNorm,
            //ColorType::R16G16UNorm => sk::ColorType::R16G16UNorm,
            ColorType::R16G16B16A16UNorm => sk::ColorType::R16G16B16A16UNorm,
            ColorType::SRGBA8888 => sk::ColorType::SRGBA8888,
            ColorType::R8UNorm => sk::ColorType::R8UNorm,
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
struct CompositorInner {
    backend: backend::composition::Compositor,
    layers: SlotMap<LayerID, LayerInfo>,
    tree: SecondaryMap<LayerID, TreeInfo>,
    transforms: SecondaryMap<LayerID, TransformInfo>,
    containers: SecondaryMap<LayerID, ContainerInfo>,
    effects: SecondaryMap<LayerID, EffectInfo>,
    surfaces: SecondaryMap<LayerID, SurfaceInfo>,
}

////////////////////////////////////////////////////////////////////////////////////////////////////

/// A connection to the system compositor.
pub struct Compositor {
    inner: RefCell<CompositorInner>,
}

impl Compositor {
    pub(crate) fn new(app_backend: &backend::AppBackend) -> Compositor {
        let backend = backend::composition::Compositor::new(app_backend);
        let inner = CompositorInner {
            backend,
            layers: Default::default(),
            tree: Default::default(),
            transforms: Default::default(),
            containers: Default::default(),
            effects: Default::default(),
            surfaces: Default::default(),
        };
        Compositor {
            inner: RefCell::new(inner),
        }
    }

    /// Creates a container layer.
    pub fn create_container_layer(&self) -> LayerID {
        let mut inner = self.inner.borrow_mut();
        let id = inner.layers.insert(LayerInfo {});
        inner.containers.insert(id, ContainerInfo::default());
        inner.backend.create_container_layer(id);
        id
    }

    /// Inserts a layer into the composition tree.
    pub fn insert_layer(&self, parent: LayerID, new_child: LayerID, reference: Option<LayerID>) {
        let mut inner = self.inner.borrow_mut();
        assert!(
            inner.containers.contains_key(parent),
            "parent should be a container layer"
        );
        if let Some(before) = reference {
            assert!(
                inner.tree.contains_key(before) && inner.tree[before].parent == Some(parent),
                "reference should be a child of parent"
            );
        }

        let new_prev_sibling = match reference {
            Some(before) => inner.tree[before].prev_sibling,
            None => inner.containers[parent].last_child,
        };

        inner.tree.insert(
            new_child,
            TreeInfo {
                parent: Some(parent),
                prev_sibling: new_prev_sibling,
                next_sibling: reference,
            },
        );

        match reference {
            Some(before) => inner.tree[before].next_sibling = Some(new_child),
            None => inner.containers[parent].last_child = Some(new_child),
        }

        if inner.tree[new_child].prev_sibling.is_none() {
            inner.containers[parent].first_child = Some(new_child)
        }

        inner.backend.insert_layer(parent, new_child, reference);
    }

    /// Creates a drawable surface layer.
    ///
    /// Use `acquire_drawing_surface` to obtain a drawable surface with a Skia context from the layer.
    ///
    /// # Argument
    ///
    /// * size Size of the surface in pixels
    /// * format Pixel format
    pub fn create_surface_layer(&self, size: Size, format: ColorType) -> LayerID {
        let mut inner = self.inner.borrow_mut();
        let id = inner.layers.insert(LayerInfo {});
        inner.surfaces.insert(id, SurfaceInfo {});
        inner.backend.create_surface_layer(id, size, format);
        id
    }

    /// Resizes a surface layer.
    pub fn set_surface_layer_size(&self, layer: LayerID, size: Size) {
        let mut inner = self.inner.borrow_mut();
        inner.backend.set_surface_layer_size(layer, size);
    }

    /// Binds a layer to a native window.
    pub unsafe fn bind_layer(&self, layer: LayerID, window: RawWindowHandle) {
        let mut inner = self.inner.borrow_mut();
        inner.backend.bind_layer(layer, window);
    }

    /// Removes a layer from the tree.
    pub fn remove_layer(&self, old_child: LayerID) {
        let mut inner = self.inner.borrow_mut();
        let old_tree = inner
            .tree
            .remove(old_child)
            .expect("layer should be inserted in the composition tree");
        match old_tree.parent {
            None => {}
            Some(parent_layer) => {
                inner.backend.remove_layer(old_child, parent_layer);

                match old_tree.prev_sibling {
                    None => inner.containers[parent_layer].first_child = old_tree.next_sibling,
                    Some(prev_sibling) => inner.tree[prev_sibling].next_sibling = old_tree.next_sibling,
                }
                match old_tree.next_sibling {
                    None => inner.containers[parent_layer].last_child = old_tree.prev_sibling,
                    Some(next_sibling) => inner.tree[next_sibling].prev_sibling = old_tree.prev_sibling,
                }
            }
        }
    }

    /// Waits for the specified surface to be ready for presentation.
    ///
    /// TODO explain
    pub fn wait_for_surface(&self, surface: LayerID) {
        let mut inner = self.inner.borrow_mut();
        inner.backend.wait_for_surface(surface)
    }

    /// Creates a skia drawing context to paint on the specified surface layer.
    ///
    /// Only one drawing context can be active at a time.
    pub fn acquire_drawing_surface(&self, surface_layer: LayerID) -> DrawableSurface {
        let mut inner = self.inner.borrow_mut();
        DrawableSurface {
            backend: inner.backend.acquire_drawing_surface(surface_layer),
        }
    }

    /// Releases the drawing surface for the specified surface layer.
    pub fn release_drawing_surface(&self, surface_layer: LayerID, drawing_surface: DrawableSurface) {
        let mut inner = self.inner.borrow_mut();
        inner
            .backend
            .release_drawing_surface(surface_layer, drawing_surface.backend)
    }

    // Returns a platform-specific surface object that can be used to paint onto the surface layer.
    //pub fn get_native_surface(&mut self, surface_layer_id: LayerID) -> &backend::CompositionSurface {}

    pub fn destroy_layer(&self, layer: LayerID) {
        //todo!()
        let mut inner = self.inner.borrow_mut();
        inner.backend.destroy_layer(layer);
    }
}
