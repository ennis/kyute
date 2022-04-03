use crate::{animation::surface::CompositionSurface, application::Application};
use kyute_common::Transform;
use windows::{
    core::Interface,
    Foundation::Numerics::Matrix3x2,
    Win32::Graphics::{
        Direct2D::Common::{D2D_MATRIX_3X2_F, D2D_MATRIX_3X2_F_0, D2D_MATRIX_4X4_F, D2D_MATRIX_4X4_F_0},
        DirectComposition::{IDCompositionVisual2, IDCompositionVisual3},
    },
};

#[derive(Clone)]
pub struct CompositionLayer {
    /// DirectComposition visual associated to the layer.
    pub(crate) visual: IDCompositionVisual3,
}

impl CompositionLayer {
    /// Creates a new layer.
    pub fn new() -> CompositionLayer {
        let app = Application::instance();
        let comp_device = app.composition_device.get_ref().unwrap();
        let visual: IDCompositionVisual2 = unsafe { comp_device.CreateVisual().expect("CreateVisual failed") };
        let visual: IDCompositionVisual3 = visual.cast().expect("cast to IDCompositionVisual3 failed");
        CompositionLayer { visual }
    }

    /// Sets the surface of this layer.
    pub fn set_content(&self, surface: &CompositionSurface) {
        unsafe {
            self.visual
                .SetContent(surface.swap_chain.clone())
                .expect("SetContent failed")
        }
    }

    pub fn set_transform(&self, transform: &Transform) {
        let matrix = Matrix3x2 {
            M11: transform.m11 as f32,
            M12: transform.m12 as f32,
            M21: transform.m21 as f32,
            M22: transform.m22 as f32,
            M31: transform.m31 as f32,
            M32: transform.m32 as f32,
        };
        unsafe {
            self.visual.SetTransform2(&matrix).expect("SetTransform2 failed");
        }
    }

    /// Adds a child layer.
    pub fn add_child(&self, layer: &CompositionLayer) {
        unsafe {
            self.visual
                .AddVisual(layer.visual.clone(), true, None)
                .expect("AddVisual failed");
        }
    }

    /// Removes all child layers.
    pub fn remove_all(&self) {
        unsafe {
            self.visual.RemoveAllVisuals().expect("RemoveAllVisuals failed");
        }
    }

    /// Removes the specified layer.
    pub fn remove_child(&self, layer: &CompositionLayer) {
        unsafe {
            self.visual
                .RemoveVisual(layer.visual.clone())
                .expect("RemoveVisual failed");
        }
    }
}
