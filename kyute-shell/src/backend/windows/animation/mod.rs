//! Composition layers - DirectComposition
use crate::application::Application;
use graal::{platform::windows::DeviceExtWindows, vk};
use kyute_common::{counter::Counter, SizeI, Transform};
use std::{
    cell::{Cell, RefCell, RefMut},
    ptr,
    sync::Arc,
};
use tracing::trace;
use windows::{
    core::{Interface, PCWSTR},
    Foundation::Numerics::Matrix3x2,
    Win32::{
        Foundation::{CloseHandle, HANDLE},
        Graphics::{
            Direct3D12::ID3D12Resource,
            DirectComposition::{IDCompositionVisual2, IDCompositionVisual3},
            Dxgi::{
                Common::{
                    DXGI_ALPHA_MODE_IGNORE, DXGI_ALPHA_MODE_PREMULTIPLIED, DXGI_FORMAT_R8G8B8A8_UNORM, DXGI_SAMPLE_DESC,
                },
                IDXGISwapChain3, DXGI_SCALING_STRETCH, DXGI_SWAP_CHAIN_DESC1, DXGI_SWAP_EFFECT_FLIP_DISCARD,
                DXGI_SWAP_EFFECT_FLIP_SEQUENTIAL, DXGI_USAGE_RENDER_TARGET_OUTPUT,
            },
        },
        System::SystemServices::GENERIC_ALL,
    },
};

////////////////////////////////////////////////////////////////////////////////////////////////////
// Swap chain
////////////////////////////////////////////////////////////////////////////////////////////////////

//const COMPOSITION_SWAP_CHAIN_COUNTER: Counter = Counter::new();

/// A wrapper around a DXGI swap chain, whose buffers are shared with vulkan images.
struct CompositionSwapChain {
    /// Backing swap chain.
    swap_chain: IDXGISwapChain3,
    /// Imported vulkan images for the swap chain buffers.
    interop_images: Vec<(HANDLE, graal::ImageInfo)>,
    /// Size of the swap chain.
    size: SizeI,
}

impl Drop for CompositionSwapChain {
    fn drop(&mut self) {
        // release the buffers
        self.release_interop();
    }
}

impl CompositionSwapChain {
    /// Creates imported vulkan images for the swap chain buffers.
    fn create_interop(&mut self) {
        assert!(self.interop_images.is_empty());

        let app = Application::instance();
        let d3d12_device = &app.backend.d3d12_device.0;
        let gr_device = app.gpu_device();

        // --- wrap swap chain buffers as vulkan images ---
        for i in 0..2 {
            // obtain the ID3D12Resource of each swap chain buffer and create a shared handle for them
            let dx_buffer: ID3D12Resource = unsafe {
                self.swap_chain
                    .GetBuffer::<ID3D12Resource>(i)
                    .expect("GetBuffer failed")
            };
            let shared_handle = unsafe {
                d3d12_device
                    .CreateSharedHandle(
                        &dx_buffer,
                        ptr::null(),
                        GENERIC_ALL,
                        None,
                        /*PCWSTR(
                            format!(
                                "kyute_shell::animation::CompositionSurface@{}:{}",
                                COMPOSITION_SWAP_CHAIN_COUNTER.next(),
                                i
                            )
                            .to_wide()
                            .as_ptr(),
                        ),*/
                    )
                    .expect("CreateSharedHandle failed")
            };

            // create a vulkan image with memory imported from the shared handle
            let imported_image = unsafe {
                gr_device.create_imported_image_win32(
                    "composition surface",
                    &graal::ImageResourceCreateInfo {
                        image_type: vk::ImageType::TYPE_2D,
                        usage: vk::ImageUsageFlags::COLOR_ATTACHMENT
                            | vk::ImageUsageFlags::TRANSFER_DST
                            | graal::vk::ImageUsageFlags::TRANSFER_SRC,
                        format: vk::Format::R8G8B8A8_UNORM,
                        extent: vk::Extent3D {
                            width: self.size.width as u32,
                            height: self.size.height as u32,
                            depth: 1,
                        },
                        mip_levels: 1,
                        array_layers: 1,
                        samples: 1,
                        tiling: Default::default(),
                    },
                    vk::MemoryPropertyFlags::default(),
                    vk::MemoryPropertyFlags::default(),
                    vk::ExternalMemoryHandleTypeFlags::D3D12_RESOURCE_KHR,
                    shared_handle.0 as vk::HANDLE,
                    None,
                )
            };

            self.interop_images.push((shared_handle, imported_image));
        }
    }

    /// Waits for the 3D device to be idle and destroys the vulkan images previously created with `create_interop()`.
    fn release_interop(&mut self) {
        let app = Application::instance();

        // before releasing the buffers, we must make sure that the swap chain is not in use
        // TODO we don't bother with setting up fences around the swap chain, we just wait for all commands to complete.
        // We could use fences to avoid unnecessary waiting, but not sure that it's worth the complication.
        app.backend.wait_for_command_completion();

        // destroy the vulkan imported images
        let device = Application::instance().gpu_device();
        for &(handle, img) in self.interop_images.iter() {
            unsafe {
                CloseHandle(handle);
            }
            device.destroy_image(img.id)
        }
        self.interop_images.clear();
    }

    /// Creates a new composition surface with the given size.
    fn new(size: SizeI) -> CompositionSwapChain {
        let app = Application::instance();

        let width = size.width as u32;
        let height = size.height as u32;

        assert_ne!(width, 0, "composition surface cannot be zero-sized");
        assert_ne!(height, 0, "composition surface cannot be zero-sized");

        // --- create swap chain ---
        let swap_chain_desc = DXGI_SWAP_CHAIN_DESC1 {
            Width: width,
            Height: height,
            Format: DXGI_FORMAT_R8G8B8A8_UNORM,
            Stereo: false.into(),
            SampleDesc: DXGI_SAMPLE_DESC { Count: 1, Quality: 0 },
            BufferUsage: DXGI_USAGE_RENDER_TARGET_OUTPUT,
            BufferCount: 2,
            Scaling: DXGI_SCALING_STRETCH,
            SwapEffect: DXGI_SWAP_EFFECT_FLIP_DISCARD,
            AlphaMode: DXGI_ALPHA_MODE_IGNORE,
            Flags: 0,
        };
        let swap_chain: IDXGISwapChain3 = unsafe {
            app.backend
                .dxgi_factory
                .0
                .CreateSwapChainForComposition(&app.backend.d3d12_command_queue.0, &swap_chain_desc, None)
                .expect("CreateSwapChainForComposition failed")
                .cast::<IDXGISwapChain3>()
                .unwrap()
        };

        let mut swap_chain = CompositionSwapChain {
            swap_chain,
            interop_images: Vec::new(),
            size,
        };
        swap_chain.create_interop();
        swap_chain
    }

    /// Resizes the surface.
    fn set_size(&mut self, new_size: SizeI) {
        if new_size == self.size {
            return;
        }

        self.release_interop();
        self.size = new_size;
        unsafe {
            self.swap_chain
                .ResizeBuffers(
                    2,
                    new_size.width as u32,
                    new_size.height as u32,
                    DXGI_FORMAT_R8G8B8A8_UNORM,
                    0,
                )
                .expect("IDXGISwapChain::ResizeBuffers failed");
        }
        self.create_interop();
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// Swap chain surface
////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct Surface {
    swap_chain: IDXGISwapChain3,
    layer: Arc<LayerImpl>,
    buffer: graal::ImageInfo,
}

impl Surface {
    pub fn image_info(&self) -> graal::ImageInfo {
        self.buffer
    }

    pub fn size(&self) -> SizeI {
        self.layer.size.get()
    }
}

impl Drop for Surface {
    fn drop(&mut self) {
        unsafe {
            let _span = trace_span!("composition_surface_present").entered();
            trace!("surface present");
            self.swap_chain.Present(0, 0).ok().expect("Present failed");
            self.layer.surface_acquired.set(false);
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// Composition layer
////////////////////////////////////////////////////////////////////////////////////////////////////

pub(crate) struct LayerImpl {
    /// DirectComposition visual associated to the layer.
    ///
    /// A DirectComposition visual is always associated with a layer, even if it has no contents.
    visual: IDCompositionVisual3,

    /// Size of the visual in pixels. Used to determine the size of composition surfaces if they are needed,
    /// does not mask the result.
    size: Cell<SizeI>,

    /// swap chain
    swap_chain: RefCell<Option<CompositionSwapChain>>,

    /// Whether there's an instance of `Surface` drawing to a buffer of the swap chain.
    surface_acquired: Cell<bool>,
}

/// A layer in the compositor tree.
#[derive(Clone)]
pub struct Layer(pub(crate) Arc<LayerImpl>);

impl Layer {
    /// Creates a new layer.
    pub fn new() -> Layer {
        let app = Application::instance();
        let comp_device = app.backend.composition_device.get_ref().unwrap();
        let visual: IDCompositionVisual2 = unsafe { comp_device.CreateVisual().expect("CreateVisual failed") };
        let visual: IDCompositionVisual3 = visual.cast().expect("cast to IDCompositionVisual3 failed");
        Layer(Arc::new(LayerImpl {
            visual,
            size: Default::default(),
            swap_chain: RefCell::new(None),
            surface_acquired: Default::default(),
        }))
    }

    /// Returns the DirectComposition visual associated to this layer.
    pub(crate) fn visual(&self) -> &IDCompositionVisual3 {
        &self.0.visual
    }

    /// Panics if this layer's size is null.
    fn ensure_swap_chain(&self) -> RefMut<CompositionSwapChain> {
        // FIXME cancerous borrow_mut code
        let mut swap_chain = self.0.swap_chain.borrow_mut();
        {
            let swap_chain = &mut *swap_chain;
            if swap_chain.is_none() {
                let sc = CompositionSwapChain::new(self.0.size.get());
                unsafe {
                    self.0.visual.SetContent(&sc.swap_chain).expect("SetContent failed");
                }
                *swap_chain = Some(sc);
            }
        }
        RefMut::map(swap_chain, |s| s.as_mut().unwrap())
    }

    /// Returns a surface for drawing on this layer.
    ///
    /// The returned `Surface` object must be dropped before `acquire_surface` is called again, otherwise
    /// the function will panic.
    pub fn acquire_surface(&self) -> Surface {
        assert!(!self.0.surface_acquired.get());
        let swap_chain = self.ensure_swap_chain();
        let buf_index = unsafe { swap_chain.swap_chain.GetCurrentBackBufferIndex() };
        let (_, buffer) = swap_chain.interop_images[buf_index as usize];
        self.0.surface_acquired.set(true);
        Surface {
            swap_chain: swap_chain.swap_chain.clone(),
            layer: Arc::clone(&self.0),
            buffer,
        }
    }

    /// Sets the transform of this layer.
    ///
    /// See `crate::animation::Layer::set_transform`
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
            self.0.visual.SetTransform2(&matrix).expect("SetTransform2 failed");
        }
    }

    /// See `crate::animation::Layer::add_child`.
    pub fn add_child(&self, layer: &Layer) {
        unsafe {
            self.0
                .visual
                .AddVisual(&layer.0.visual, true, None)
                .expect("AddVisual failed");
        }
    }

    /// See `crate::animation::Layer::remove_all_children`.
    pub fn remove_all_children(&self) {
        unsafe {
            self.0.visual.RemoveAllVisuals().expect("RemoveAllVisuals failed");
        }
    }

    /// See `crate::animation::Layer::remove_child`.
    pub fn remove_child(&self, layer: &Layer) {
        unsafe {
            self.0
                .visual
                .RemoveVisual(&layer.0.visual)
                .expect("RemoveVisual failed");
        }
    }

    /// Returns the size of this layer.
    pub fn size(&self) -> SizeI {
        self.0.size.get()
    }

    /// See `crate::animation::Layer::set_size`.
    pub fn set_size(&self, new_size: SizeI) {
        assert!(!self.0.surface_acquired.get());
        self.0.size.set(new_size);
        // if there's a swap chain associated to this layer, resize it now.
        let mut swap_chain = self.0.swap_chain.borrow_mut();
        if let Some(swap_chain) = &mut *swap_chain {
            swap_chain.set_size(new_size);
        }
    }
}
