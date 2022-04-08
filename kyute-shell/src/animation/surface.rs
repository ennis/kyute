use crate::{application::Application, backend::util::ToWide};
use graal::{platform::windows::DeviceExtWindows, vk};
use kyute_common::{counter::Counter, SizeI};
use std::ptr;
use tracing::trace;
use windows::{
    core::{Interface, PCWSTR},
    Win32::{
        Graphics::{
            Direct3D12::ID3D12Resource,
            Dxgi::{
                Common::{DXGI_ALPHA_MODE_PREMULTIPLIED, DXGI_FORMAT_R8G8B8A8_UNORM, DXGI_SAMPLE_DESC},
                IDXGISwapChain3, DXGI_SCALING_STRETCH, DXGI_SWAP_CHAIN_DESC1, DXGI_SWAP_EFFECT_FLIP_DISCARD,
                DXGI_USAGE_RENDER_TARGET_OUTPUT,
            },
        },
        System::SystemServices::GENERIC_ALL,
    },
};

static COMPOSITION_SURFACE_COUNTER: Counter = Counter::new();

pub struct SurfaceDrawCtx {
    pub width: u32,
    pub height: u32,
}

pub struct CompositionSurface {
    /// Backing swap chain.
    pub swap_chain: IDXGISwapChain3,
    pub buffers: Vec<graal::ImageInfo>,
    pub(crate) size: SizeI,
}

impl Drop for CompositionSurface {
    fn drop(&mut self) {
        // release the buffers
        let device = Application::instance().gpu_device();
        for img in self.buffers.iter() {
            device.destroy_image(img.id)
        }
    }
}

impl CompositionSurface {
    /// Creates a new composition surface with the given size.
    pub fn new(size: SizeI) -> CompositionSurface {
        let app = Application::instance();

        let width = size.width as u32;
        let height = size.height as u32;

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
            AlphaMode: DXGI_ALPHA_MODE_PREMULTIPLIED,
            Flags: 0,
        };
        let swap_chain: IDXGISwapChain3 = unsafe {
            app.dxgi_factory
                .CreateSwapChainForComposition(&app.d3d12_command_queue.0, &swap_chain_desc, None)
                .expect("CreateSwapChainForComposition failed")
                .cast::<IDXGISwapChain3>()
                .unwrap()
        };

        let d3d12_device = &app.d3d12_device.0;
        let gr_device = app.gpu_device();

        // --- wrap swap chain buffers as vulkan images ---
        let mut buffers = Vec::new();
        for i in 0..2 {
            let dx_buffer: ID3D12Resource =
                unsafe { swap_chain.GetBuffer::<ID3D12Resource>(i).expect("GetBuffer failed") };
            let shared_handle = unsafe {
                d3d12_device
                    .CreateSharedHandle(
                        &dx_buffer,
                        ptr::null(),
                        GENERIC_ALL,
                        PCWSTR(
                            format!(
                                "kyute_shell::animation::CompositionSurface@{}:{}",
                                COMPOSITION_SURFACE_COUNTER.next(),
                                i
                            )
                            .to_wide()
                            .as_ptr(),
                        ),
                    )
                    .expect("CreateSharedHandle failed")
            };

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
                            width,
                            height,
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

            buffers.push(imported_image);
        }

        CompositionSurface {
            swap_chain,
            buffers,
            size,
        }
    }

    /// Draws on this surface.
    pub fn draw(&self, draw: impl FnOnce(&SurfaceDrawCtx, &graal::ImageInfo)) {
        let buf_index = unsafe { self.swap_chain.GetCurrentBackBufferIndex() };
        let target_image = &self.buffers[buf_index as usize];
        let surface_draw_ctx = SurfaceDrawCtx {
            width: self.size.width as u32,
            height: self.size.height as u32,
        };
        trace!("CompositionSurface::draw {:?}", self.size);
        draw(&surface_draw_ctx, &target_image);
        unsafe {
            let _span = trace_span!("composition_surface_present").entered();
            self.swap_chain.Present(0, 0).expect("Present failed");
        }
    }
}
