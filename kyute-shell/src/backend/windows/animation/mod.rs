//! Composition layers - DirectComposition
use crate::application::Application;
use graal::{platform::windows::DeviceExtWindows, vk};
use kyute_common::{counter::Counter, SizeI, Transform};
use skia_safe::runtime_effect::uniform::Type::Int;
use std::{
    cell::{Cell, RefCell, RefMut},
    ffi::c_void,
    mem::ManuallyDrop,
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
            Direct3D12::{
                ID3D12CommandList, ID3D12Fence, ID3D12GraphicsCommandList, ID3D12Resource,
                D3D12_COMMAND_LIST_TYPE_DIRECT, D3D12_CPU_DESCRIPTOR_HANDLE, D3D12_FENCE_FLAG_SHARED,
                D3D12_RESOURCE_BARRIER, D3D12_RESOURCE_BARRIER_0, D3D12_RESOURCE_BARRIER_FLAG_NONE,
                D3D12_RESOURCE_BARRIER_TYPE_TRANSITION, D3D12_RESOURCE_STATE_PRESENT,
                D3D12_RESOURCE_STATE_RENDER_TARGET, D3D12_RESOURCE_TRANSITION_BARRIER,
            },
            DirectComposition::{IDCompositionVisual2, IDCompositionVisual3},
            Dxgi::{
                Common::{
                    DXGI_ALPHA_MODE_IGNORE, DXGI_ALPHA_MODE_PREMULTIPLIED, DXGI_FORMAT_R10G10B10A2_UNORM,
                    DXGI_FORMAT_R16G16B16A16_FLOAT, DXGI_FORMAT_R8G8B8A8_UNORM, DXGI_FORMAT_R8G8B8A8_UNORM_SRGB,
                    DXGI_SAMPLE_DESC,
                },
                IDXGISwapChain3, DXGI_SCALING_STRETCH, DXGI_SWAP_CHAIN_DESC1,
                DXGI_SWAP_CHAIN_FLAG_FRAME_LATENCY_WAITABLE_OBJECT, DXGI_SWAP_EFFECT_FLIP_DISCARD,
                DXGI_SWAP_EFFECT_FLIP_SEQUENTIAL, DXGI_USAGE_RENDER_TARGET_OUTPUT, DXGI_USAGE_SHARED,
            },
        },
        System::SystemServices::GENERIC_ALL,
    },
};

////////////////////////////////////////////////////////////////////////////////////////////////////
// Swap chain
////////////////////////////////////////////////////////////////////////////////////////////////////

//const COMPOSITION_SWAP_CHAIN_COUNTER: Counter = Counter::new();

struct InteropImage {
    /// Shared handle to DXGI swap chain buffer.
    image_shared_handle: HANDLE,
    /// Imported DXGI swap chain buffer.
    image: graal::ImageInfo,
    /// Command list containing a single transition barrier for the swap chain buffer.
    ///
    /// Submitted to the D3D12 Queue before Vulkan work, needed to force an implicit synchronization with
    /// presentation.
    barrier_command_list: ID3D12GraphicsCommandList,
}

/// A wrapper around a DXGI swap chain, whose buffers are shared with vulkan images.
struct CompositionSwapChain {
    /// Backing swap chain.
    swap_chain: IDXGISwapChain3,
    /// Imported vulkan images for the swap chain buffers.
    interop_images: Vec<InteropImage>,
    /// Size of the swap chain.
    size: SizeI,
}

impl CompositionSwapChain {
    /// Creates a new composition surface with the given size.
    fn new(size: SizeI) -> CompositionSwapChain {
        eprintln!("new composition swap chain");
        let app = Application::instance();

        let width = size.width as u32;
        let height = size.height as u32;

        assert_ne!(width, 0, "composition surface cannot be zero-sized");
        assert_ne!(height, 0, "composition surface cannot be zero-sized");

        // --- create swap chain ---
        let swap_chain_desc = DXGI_SWAP_CHAIN_DESC1 {
            Width: width,
            Height: height,
            Format: DXGI_FORMAT_R16G16B16A16_FLOAT,
            Stereo: false.into(),
            SampleDesc: DXGI_SAMPLE_DESC { Count: 1, Quality: 0 },
            BufferUsage: DXGI_USAGE_RENDER_TARGET_OUTPUT,
            BufferCount: 2,
            Scaling: DXGI_SCALING_STRETCH,
            SwapEffect: DXGI_SWAP_EFFECT_FLIP_DISCARD,
            AlphaMode: DXGI_ALPHA_MODE_PREMULTIPLIED,
            Flags: DXGI_SWAP_CHAIN_FLAG_FRAME_LATENCY_WAITABLE_OBJECT.0 as u32,
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

    /// Creates imported vulkan images for the swap chain buffers.
    fn create_interop(&mut self) {
        assert!(self.interop_images.is_empty());

        let app = Application::instance();
        let d3d12_device = &app.backend.d3d12_device.0;
        let gr_device = app.gpu_device();

        unsafe {
            // --- wrap swap chain buffers as vulkan images ---
            for i in 0..2 {
                // obtain the ID3D12Resource of each swap chain buffer and create a shared handle for them
                let swap_chain_buffer: ID3D12Resource = self
                    .swap_chain
                    .GetBuffer::<ID3D12Resource>(i)
                    .expect("GetBuffer failed");
                let shared_handle = d3d12_device
                    .CreateSharedHandle(&swap_chain_buffer, ptr::null(), GENERIC_ALL, None)
                    .expect("CreateSharedHandle failed");

                // create a vulkan image with memory imported from the shared handle
                let imported_image = gr_device.create_imported_image_win32(
                    "composition surface",
                    &graal::ImageResourceCreateInfo {
                        image_type: vk::ImageType::TYPE_2D,
                        usage: vk::ImageUsageFlags::COLOR_ATTACHMENT
                            | vk::ImageUsageFlags::TRANSFER_DST
                            | graal::vk::ImageUsageFlags::TRANSFER_SRC,
                        format: vk::Format::R16G16B16A16_SFLOAT,
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
                );

                // initial command list

                // initial sync - D3D12 signal
                //let command_queue = &app.backend.d3d12_command_queue;
                // dummy rendering
                let command_list: ID3D12GraphicsCommandList = d3d12_device
                    .CreateCommandList(
                        0,
                        D3D12_COMMAND_LIST_TYPE_DIRECT,
                        app.backend.d3d12_command_allocator.get_ref().unwrap(),
                        None,
                    )
                    .unwrap();

                /*// FIXME manually drop shit, see https://github.com/microsoft/windows-rs/issues/1410
                let mut barrier = [D3D12_RESOURCE_BARRIER {
                    Type: D3D12_RESOURCE_BARRIER_TYPE_TRANSITION,
                    Flags: D3D12_RESOURCE_BARRIER_FLAG_NONE,
                    Anonymous: D3D12_RESOURCE_BARRIER_0 {
                        Transition: ManuallyDrop::new(D3D12_RESOURCE_TRANSITION_BARRIER {
                            pResource: Some(dx_buffer.clone()),
                            Subresource: 0,
                            StateBefore: D3D12_RESOURCE_STATE_PRESENT,
                            StateAfter: D3D12_RESOURCE_STATE_RENDER_TARGET,
                        }),
                    },
                }];
                command_list.ResourceBarrier(&barrier);
                ManuallyDrop::drop(&mut barrier[0].Anonymous.Transition);*/

                command_list.DiscardResource(&swap_chain_buffer, ptr::null());
                command_list.Close();

                let interop_image = InteropImage {
                    image_shared_handle: shared_handle,
                    image: imported_image,
                    barrier_command_list: command_list,
                };

                self.interop_images.push(interop_image);
            }
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
        for interop_image in self.interop_images.iter() {
            unsafe {
                CloseHandle(interop_image.image_shared_handle);
            }
            device.destroy_image(interop_image.image.id);
        }
        self.interop_images.clear();
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
                    DXGI_FORMAT_R16G16B16A16_FLOAT,
                    DXGI_SWAP_CHAIN_FLAG_FRAME_LATENCY_WAITABLE_OBJECT.0 as u32,
                )
                .expect("IDXGISwapChain::ResizeBuffers failed");
        }
        self.create_interop();
    }
}

impl Drop for CompositionSwapChain {
    fn drop(&mut self) {
        // release the buffers
        self.release_interop();
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// Swap chain surface
////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct Surface {
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
        // FIXME The swap chain is presented when this surface object is dropped. We may want to have a more explicit method for that.
        // FIXME there are artifacts; not sure where they come from, try to use a "staging" image instead of directly sharing the swapchain buffers
        unsafe { self.layer.present_and_release_surface(&self.buffer) }
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
    /// XXX why is it not created immediately?
    swap_chain: RefCell<Option<CompositionSwapChain>>,

    /// Presentation fence value
    presentation_fence_value: Cell<u64>,

    /// Vulkan side of the presentation fence
    /// TODO: not sure we need one fence per buffer, maybe a single fence
    presentation_fence_semaphore: vk::Semaphore,
    /// D3D12 side of the presentation fence
    presentation_fence: ID3D12Fence,
    presentation_fence_shared_handle: HANDLE,

    /// Whether there's an instance of `Surface` drawing to a buffer of the swap chain.
    surface_acquired: Cell<bool>,
}

impl LayerImpl {
    fn new() -> LayerImpl {
        let app = Application::instance();
        let comp_device = app.backend.composition_device.get_ref().unwrap();
        let visual: IDCompositionVisual2 = unsafe { comp_device.CreateVisual().expect("CreateVisual failed") };
        let visual: IDCompositionVisual3 = visual.cast().expect("cast to IDCompositionVisual3 failed");

        let d3d12 = &app.backend.d3d12_device;

        // Create & share a D3D12 fence for VK/DXGI sync
        let presentation_fence = unsafe { d3d12.CreateFence(0, D3D12_FENCE_FLAG_SHARED).unwrap() };
        let presentation_fence_shared_handle = unsafe {
            d3d12
                .CreateSharedHandle(&presentation_fence, ptr::null(), GENERIC_ALL, None)
                .unwrap()
        };
        let presentation_fence_semaphore = unsafe {
            app.gpu_device().create_imported_semaphore_win32(
                vk::SemaphoreImportFlags::empty(),
                vk::ExternalSemaphoreHandleTypeFlags::D3D12_FENCE,
                presentation_fence_shared_handle.0 as *mut c_void,
                None,
            )
        };

        LayerImpl {
            visual,
            size: Default::default(),
            swap_chain: RefCell::new(None),
            presentation_fence_value: Cell::new(1),
            presentation_fence_semaphore,
            presentation_fence,
            presentation_fence_shared_handle,
            surface_acquired: Default::default(),
        }
    }

    fn ensure_swap_chain(&self) -> RefMut<CompositionSwapChain> {
        // FIXME cancerous borrow_mut code
        let mut swap_chain = self.swap_chain.borrow_mut();
        {
            let swap_chain = &mut *swap_chain;
            if swap_chain.is_none() {
                let sc = CompositionSwapChain::new(self.size.get());
                unsafe {
                    self.visual.SetContent(&sc.swap_chain).expect("SetContent failed");
                }
                *swap_chain = Some(sc);
            }
        }
        RefMut::map(swap_chain, |s| s.as_mut().unwrap())
    }

    fn acquire_surface(&self) -> graal::ImageInfo {
        assert!(!self.surface_acquired.get());

        let app = Application::instance();

        let swap_chain = self.ensure_swap_chain();
        let buf_index = unsafe { swap_chain.swap_chain.GetCurrentBackBufferIndex() };
        let interop_image = &swap_chain.interop_images[buf_index as usize];

        let fence_value = self.presentation_fence_value.get();
        self.presentation_fence_value.set(fence_value + 1);

        // initial sync - D3D12 signal
        let command_queue = &app.backend.d3d12_command_queue;
        unsafe {
            // dummy rendering
            command_queue
                .0
                .ExecuteCommandLists(&[Some(interop_image.barrier_command_list.clone().into())]);
            command_queue.0.Signal(&self.presentation_fence, fence_value).unwrap();
        }

        // initial sync - vulkan wait
        {
            let mut gpu_ctx = app.lock_gpu_context();
            let mut frame = graal::Frame::new();
            frame.add_pass(unsafe {
                graal::PassBuilder::new()
                    .external_semaphore_wait(
                        self.presentation_fence_semaphore,
                        vk::PipelineStageFlags::ALL_COMMANDS,
                        graal::SemaphoreWaitKind::D3D12Fence(fence_value),
                    )
                    .name("DXGI-to-Vulkan sync")
            });
            gpu_ctx.submit_frame(&mut (), frame, &Default::default());
        }

        self.surface_acquired.set(true);
        interop_image.image
    }

    /// Presents and releases a surface.
    ///
    /// Called by Surface::drop.
    unsafe fn present_and_release_surface(&self, _buffer: &graal::ImageInfo) {
        let _span = trace_span!("present_and_release_surface").entered();
        trace!("surface present");

        let app = Application::instance();

        let fence_value = self.presentation_fence_value.get();
        self.presentation_fence_value.set(fence_value + 1);

        {
            let mut gpu_ctx = app.lock_gpu_context();
            let mut frame = graal::Frame::new();
            // FIXME we signal the fence on the graphics queue, but work affecting the image might have been scheduled on another in case of async compute.
            frame.add_pass(
                graal::PassBuilder::new()
                    .external_semaphore_signal(
                        self.presentation_fence_semaphore,
                        graal::SemaphoreSignalKind::D3D12Fence(fence_value),
                    )
                    .name("Vulkan-to-DXGI sync"),
            );
            gpu_ctx.submit_frame(&mut (), frame, &Default::default());
        }

        app.backend
            .d3d12_command_queue
            .Wait(&self.presentation_fence, fence_value)
            .unwrap();

        self.ensure_swap_chain()
            .swap_chain
            .Present(1, 0)
            .ok()
            .expect("Present failed");
        self.surface_acquired.set(false);
    }
}

/// A layer in the compositor tree.
#[derive(Clone)]
pub struct Layer(pub(crate) Arc<LayerImpl>);

impl Layer {
    /// Creates a new layer.
    pub fn new() -> Layer {
        Layer(Arc::new(LayerImpl::new()))
    }

    /// Returns the DirectComposition visual associated to this layer.
    pub(crate) fn visual(&self) -> &IDCompositionVisual3 {
        &self.0.visual
    }

    /// Returns a surface for drawing on this layer.
    ///
    /// The returned `Surface` object must be dropped before `acquire_surface` is called again, otherwise
    /// the function will panic.
    pub fn acquire_surface(&self) -> Surface {
        let buffer = self.0.acquire_surface();
        Surface {
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
