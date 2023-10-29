//! Windows compositor implementation details

use crate::{
    backend,
    backend::windows::event::Win32Event,
    composition::{ColorType, LayerID},
    AppGlobals, Size,
};
use raw_window_handle::RawWindowHandle;
use skia_safe as sk;
use slotmap::SecondaryMap;
use windows::{
    core::ComInterface,
    Foundation::Numerics::Vector2,
    Win32::{
        Foundation::{CloseHandle, HANDLE, HWND},
        Graphics::{
            Direct3D12::{ID3D12CommandQueue, ID3D12Device, ID3D12Fence, ID3D12Resource, D3D12_FENCE_FLAG_NONE},
            Dxgi::{
                Common::{
                    DXGI_ALPHA_MODE_IGNORE, DXGI_ALPHA_MODE_PREMULTIPLIED, DXGI_FORMAT_R16G16B16A16_FLOAT,
                    DXGI_SAMPLE_DESC,
                },
                IDXGIFactory3, IDXGISwapChain3, DXGI_SCALING_STRETCH, DXGI_SWAP_CHAIN_DESC1,
                DXGI_SWAP_CHAIN_FLAG_FRAME_LATENCY_WAITABLE_OBJECT, DXGI_SWAP_EFFECT_FLIP_DISCARD,
                DXGI_USAGE_RENDER_TARGET_OUTPUT,
            },
        },
        System::{
            Threading::{CreateEventW, WaitForSingleObject},
            WinRT::Composition::{ICompositorDesktopInterop, ICompositorInterop},
        },
    },
    UI::Composition::{Compositor as WinCompositor, ContainerVisual, Desktop::DesktopWindowTarget, Visual},
};

//mod swap_chain;

////////////////////////////////////////////////////////////////////////////////////////////////////

const SWAP_CHAIN_BUFFER_COUNT: u32 = 2;

/// Windows drawable surface backend
pub(crate) struct DrawableSurface {
    surface: sk::Surface,
}

impl DrawableSurface {
    pub(crate) fn surface(&self) -> sk::Surface {
        self.surface.clone()
    }
}

struct SwapChain {
    inner: IDXGISwapChain3,
    frame_latency_waitable: HANDLE,
}

impl Drop for SwapChain {
    fn drop(&mut self) {
        if !self.frame_latency_waitable.is_invalid() {
            unsafe {
                CloseHandle(self.frame_latency_waitable);
            }
        }
    }
}

/// A windows compositor native layer (a `Visual`).
struct NativeLayer {
    visual: Visual,
    size: Size,
    swap_chain: Option<SwapChain>,
    window_target: Option<DesktopWindowTarget>,
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// Compositor impl
////////////////////////////////////////////////////////////////////////////////////////////////////

/// Windows compositor backend
pub(crate) struct Compositor {
    compositor: WinCompositor,
    dxgi_factory: IDXGIFactory3,
    device: ID3D12Device,
    command_queue: ID3D12CommandQueue,
    completion_fence: ID3D12Fence,
    completion_event: Win32Event,
    completion_fence_value: u64,
    //composition_graphics_device: CompositionGraphicsDevice,
    //composition_device: IDCompositionDesktopDevice,
    visuals: SecondaryMap<LayerID, NativeLayer>,
}

impl Compositor {
    pub(crate) fn new(app_backend: &backend::AppBackend) -> Compositor {
        let compositor = WinCompositor::new().expect("failed to create compositor");
        let dxgi_factory = app_backend.dxgi_factory.0.clone();
        let device = app_backend.d3d12_device.0.clone();
        let command_queue = app_backend.d3d12_command_queue.0.clone();

        let command_completion_fence = unsafe {
            device
                .CreateFence::<ID3D12Fence>(0, D3D12_FENCE_FLAG_NONE)
                .expect("CreateFence failed")
        };

        let command_completion_event = unsafe {
            let event = CreateEventW(None, false, false, None).unwrap();
            Win32Event::from_raw(event)
        };

        Compositor {
            compositor,
            dxgi_factory,
            device,
            command_queue,
            completion_fence: command_completion_fence,
            completion_event: command_completion_event,
            completion_fence_value: 0,
            visuals: Default::default(),
        }
    }

    /// Creates a container layer.
    pub(crate) fn create_container_layer(&mut self, id: LayerID) {
        let container = self
            .compositor
            .CreateContainerVisual()
            .expect("failed to create container visual");
        self.visuals.insert(
            id,
            NativeLayer {
                visual: container.cast().unwrap(),
                size: Size::ZERO,
                swap_chain: None,
                window_target: None,
            },
        );
    }

    /// Creates a surface layer.
    pub(crate) fn create_surface_layer(&mut self, id: LayerID, size: Size, format: ColorType) {
        // Create the swap chain backing the layer
        let width = size.width as u32;
        let height = size.height as u32;

        assert!(width != 0 && height != 0, "surface layer cannot be zero-sized");

        // create swap chain
        let swap_chain_desc = DXGI_SWAP_CHAIN_DESC1 {
            Width: width,
            Height: height,
            Format: DXGI_FORMAT_R16G16B16A16_FLOAT,
            Stereo: false.into(),
            SampleDesc: DXGI_SAMPLE_DESC { Count: 1, Quality: 0 },
            BufferUsage: DXGI_USAGE_RENDER_TARGET_OUTPUT,
            BufferCount: SWAP_CHAIN_BUFFER_COUNT,
            Scaling: DXGI_SCALING_STRETCH,
            SwapEffect: DXGI_SWAP_EFFECT_FLIP_DISCARD,
            AlphaMode: DXGI_ALPHA_MODE_IGNORE,
            Flags: DXGI_SWAP_CHAIN_FLAG_FRAME_LATENCY_WAITABLE_OBJECT.0 as u32,
        };
        let swap_chain: IDXGISwapChain3 = unsafe {
            self.dxgi_factory
                .CreateSwapChainForComposition(&self.command_queue, &swap_chain_desc, None)
                .expect("CreateSwapChainForComposition failed")
                .cast::<IDXGISwapChain3>()
                .unwrap()
        };
        let frame_latency_waitable = unsafe {
            swap_chain.SetMaximumFrameLatency(1).unwrap();
            swap_chain.GetFrameLatencyWaitableObject()
        };

        let swap_chain = SwapChain {
            inner: swap_chain,
            frame_latency_waitable,
        };

        // Create the composition surface representing the swap chain in the compositor
        let surface = unsafe {
            self.compositor
                .cast::<ICompositorInterop>()
                .unwrap()
                .CreateCompositionSurfaceForSwapChain(&swap_chain.inner)
                .expect("could not create composition surface for swap chain")
        };

        // Create the visual+brush holding the surface
        let visual = self.compositor.CreateSpriteVisual().unwrap();
        let brush = self.compositor.CreateSurfaceBrush().unwrap();
        brush.SetSurface(&surface).unwrap();
        visual.SetBrush(&brush).unwrap();
        let new_size = Vector2::new(size.width as f32, size.height as f32);
        visual.SetSize(new_size).unwrap();

        self.visuals.insert(
            id,
            NativeLayer {
                visual: visual.cast().unwrap(),
                size,
                swap_chain: Some(swap_chain),
                window_target: None,
            },
        );
    }

    /// Waits for submitted GPU commands to complete.
    fn wait_for_gpu_command_completion(&mut self) {
        unsafe {
            let mut fence_value = &mut self.completion_fence_value;
            *fence_value += 1;
            self.command_queue
                .Signal(&self.completion_fence, *fence_value)
                .expect("ID3D12CommandQueue::Signal failed");
            if self.completion_fence.GetCompletedValue() < *fence_value {
                self.completion_fence
                    .SetEventOnCompletion(*fence_value, self.completion_event.handle())
                    .expect("SetEventOnCompletion failed");
                WaitForSingleObject(self.completion_event.handle(), 0xFFFFFFFF);
            }
        }
    }

    /// Resizes a surface layer.
    pub(crate) fn set_surface_layer_size(&mut self, id: LayerID, size: Size) {
        let layer = &mut self.visuals[id];
        // skip if same size
        if layer.size == size {
            return;
        }

        let width = size.width as u32;
        let height = size.height as u32;
        // avoid resizing to zero width
        if width == 0 || height == 0 {
            return;
        }

        if layer.swap_chain.is_some() {
            self.wait_for_gpu_command_completion();
        }

        let layer = &mut self.visuals[id];
        if let Some(ref swap_chain) = layer.swap_chain {
            unsafe {
                // SAFETY: basic FFI call
                match swap_chain.inner.ResizeBuffers(
                    SWAP_CHAIN_BUFFER_COUNT,
                    width,
                    height,
                    DXGI_FORMAT_R16G16B16A16_FLOAT,
                    DXGI_SWAP_CHAIN_FLAG_FRAME_LATENCY_WAITABLE_OBJECT.0 as u32,
                ) {
                    Ok(_) => {}
                    Err(hr) => {
                        //let removed_reason = self.device.GetDeviceRemovedReason().unwrap_err();
                        panic!("IDXGISwapChain::ResizeBuffers failed: {}", hr);
                    }
                }
            }
        }

        layer.size = size;
        layer
            .visual
            .SetSize(Vector2::new(size.width as f32, size.height as f32))
            .unwrap();
    }

    /// Waits for the specified surface to be ready for presentation.
    ///
    /// TODO explain
    pub(crate) fn wait_for_surface(&mut self, surface: LayerID) {
        let layer = &mut self.visuals[surface];
        let swap_chain = layer.swap_chain.as_mut().expect("layer should be a surface layer");

        // if the swapchain has a mechanism for syncing with presentation, use it,
        // otherwise do nothing.
        if !swap_chain.frame_latency_waitable.is_invalid() {
            unsafe {
                WaitForSingleObject(swap_chain.frame_latency_waitable, 1000)
                    .to_hresult()
                    .unwrap();
            }
        }
    }

    /// Binds a composition layer to a window.
    ///
    /// # Safety
    ///
    /// The window handle is valid.
    ///
    /// TODO: return result
    pub(crate) unsafe fn bind_layer(&mut self, id: LayerID, window: RawWindowHandle) {
        let win32_handle = match window {
            RawWindowHandle::Win32(w) => w,
            _ => panic!("expected a Win32 window handle"),
        };
        let interop = self
            .compositor
            .cast::<ICompositorDesktopInterop>()
            .expect("could not retrieve ICompositorDesktopInterop");
        let desktop_window_target = interop
            .CreateDesktopWindowTarget(HWND(win32_handle.hwnd as isize), false)
            .expect("could not create DesktopWindowTarget");
        desktop_window_target
            .SetRoot(&self.visuals[id].visual)
            .expect("SetRoot failed");
        // self.compositor.
        self.visuals[id].window_target = Some(desktop_window_target);
    }

    /// Helper to retrieve the ContainerVisual for the specified layer.
    ///
    /// # Panics
    ///
    /// If the specified layer is not a container layer.
    fn container_visual(&self, layer: LayerID) -> ContainerVisual {
        self.visuals[layer]
            .visual
            .cast::<ContainerVisual>()
            .expect("parameter should be a container layer")
    }

    /// Inserts a layer into the visual tree.
    pub(crate) fn insert_layer(&mut self, parent: LayerID, new_child: LayerID, reference: Option<LayerID>) {
        let parent_container = self.container_visual(parent);
        let new_child_visual = &self.visuals[new_child].visual;
        if let Some(reference) = reference {
            let reference_visual = &self.visuals[reference].visual;
            parent_container
                .Children()
                .unwrap()
                .InsertBelow(new_child_visual, reference_visual)
                .expect("failed to insert visual");
        } else {
            parent_container
                .Children()
                .unwrap()
                .InsertAtTop(new_child_visual)
                .expect("failed to insert visual");
        }
    }

    /// Removes a layer from the visual tree.
    pub(crate) fn remove_layer(&mut self, id: LayerID, parent: LayerID) {
        let parent_container = self.container_visual(parent);
        let child = &self.visuals[id].visual;
        parent_container.Children().unwrap().Remove(child).unwrap();
    }

    /// Destroys a layer.
    pub(crate) fn destroy_layer(&mut self, id: LayerID) {
        self.visuals.remove(id);
    }

    /// Creates a skia drawing context for the specified surface layer.
    pub(crate) fn acquire_drawing_surface(&mut self, surface_layer: LayerID) -> DrawableSurface {
        let layer = &mut self.visuals[surface_layer];
        let swap_chain = layer.swap_chain.as_mut().expect("layer should be a surface layer");

        unsafe {
            // acquire next image from swap chain
            let index = swap_chain.inner.GetCurrentBackBufferIndex();
            let swap_chain_buffer = swap_chain
                .inner
                .GetBuffer::<ID3D12Resource>(index)
                .expect("failed to retrieve swap chain buffer");

            let app = AppGlobals::get();
            let surface = app.drawing.create_surface_for_texture(
                swap_chain_buffer,
                DXGI_FORMAT_R16G16B16A16_FLOAT,
                layer.size,
                sk::gpu::SurfaceOrigin::TopLeft,
                sk::ColorType::RGBAF16,
                sk::ColorSpace::new_srgb_linear(),
                Some(sk::SurfaceProps::new(
                    sk::SurfacePropsFlags::default(),
                    sk::PixelGeometry::RGBH,
                )),
            );
            DrawableSurface { surface }
        }
    }

    pub(crate) fn release_drawing_surface(&mut self, surface_layer: LayerID, mut surface: DrawableSurface) {
        let swap_chain = self.visuals[surface_layer]
            .swap_chain
            .as_mut()
            .expect("layer should be a surface layer");

        unsafe {
            surface.surface.flush_and_submit();
            swap_chain.inner.Present(1, 0).unwrap();
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
