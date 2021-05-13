//! Platform-specific window creation
use crate::{
    bindings::Windows::Win32::{
        Direct2D::{
            ID2D1Bitmap1, ID2D1DeviceContext, D2D1_ALPHA_MODE, D2D1_BITMAP_OPTIONS,
            D2D1_BITMAP_PROPERTIES1, D2D1_PIXEL_FORMAT,
        },
        Direct3D11::D3D11_TEXTURE2D_DESC,
        Dxgi::{
            IDXGIResource1, IDXGISurface, DXGI_FORMAT, DXGI_SAMPLE_DESC, DXGI_SHARED_RESOURCE_READ,
            DXGI_SHARED_RESOURCE_WRITE,
        },
        SystemServices::HANDLE,
    },
    error::{Error, Result},
    platform::{GpuContext, Platform},
};
use graal::{platform::windows::ContextExtWindows, vk};
use raw_window_handle::HasRawWindowHandle;
use std::{os::raw::c_void, ptr};
use tracing::trace;

use crate::bindings::Windows::Win32::{
    Direct2D::D2D1_DEVICE_CONTEXT_OPTIONS,
    Direct3D11::{
        ID3D11Fence, D3D11_BIND_FLAG, D3D11_FENCE_FLAG, D3D11_RESOURCE_MISC_FLAG, D3D11_USAGE,
    },
    SystemServices::HINSTANCE,
    WindowsAndMessaging::HWND,
};
use windows::Interface;
use winit::{
    event_loop::EventLoopWindowTarget,
    platform::windows::{WindowBuilderExtWindows, WindowExtWindows},
    window::{Window, WindowBuilder, WindowId},
};
use crate::bindings::Windows::Win32::SystemServices::GENERIC_ALL;

/*#[allow(unused)]
fn check_win32_last_error(returned: i32, function: &str) {
    unsafe {
        if returned == 0 {
            let err = GetLastError();
            panic!("{} failed, GetLastError={:08x}", function, err);
        }
    }
}*/

struct SharedDrawSurface {
    shared_handle: HANDLE,
    fence_shared_handle: HANDLE,
    dxgi_surface: IDXGISurface,
    d3d_fence: ID3D11Fence,
    d2d_bitmap: ID2D1Bitmap1,
    vulkan_image: graal::ImageInfo,
    vulkan_timeline: vk::Semaphore,
    dpi: f32,
}

impl SharedDrawSurface {
    fn new(
        d2d_device_context: &ID2D1DeviceContext,
        size: (u32, u32),
        scale_factor: f64,
    ) -> SharedDrawSurface {
        let platform = Platform::instance();
        let d3d11_device = &platform.d3d11_device;
        let mut context = platform.gpu_context.lock().unwrap();

        // ---- NEW
        let texture_desc = D3D11_TEXTURE2D_DESC {
            Width: size.0,
            Height: size.1,
            MipLevels: 1,
            ArraySize: 1,
            Format: DXGI_FORMAT::DXGI_FORMAT_B8G8R8A8_UNORM,
            SampleDesc: DXGI_SAMPLE_DESC {
                Count: 1,
                Quality: 0,
            },
            Usage: D3D11_USAGE::D3D11_USAGE_DEFAULT,
            BindFlags: (D3D11_BIND_FLAG::D3D11_BIND_RENDER_TARGET.0
                | D3D11_BIND_FLAG::D3D11_BIND_SHADER_RESOURCE.0) as u32,
            CPUAccessFlags: 0,
            MiscFlags: (D3D11_RESOURCE_MISC_FLAG::D3D11_RESOURCE_MISC_SHARED_KEYEDMUTEX.0
                | D3D11_RESOURCE_MISC_FLAG::D3D11_RESOURCE_MISC_SHARED_NTHANDLE.0)
                as u32,
        };
        let d3d_texture = unsafe {
            let mut d3d_texture = None;
            d3d11_device
                .CreateTexture2D(&texture_desc, ptr::null(), &mut d3d_texture)
                .and_some(d3d_texture)
                .unwrap()
        };

        let dxgi_resource = d3d_texture.cast::<IDXGIResource1>().unwrap();
        let dxgi_surface = d3d_texture.cast::<IDXGISurface>().unwrap();

        let mut shared_handle = HANDLE::INVALID;
        unsafe {
            dxgi_resource
                .CreateSharedHandle(
                    ptr::null(),
                    (DXGI_SHARED_RESOURCE_READ as u32) | DXGI_SHARED_RESOURCE_WRITE,
                    None,
                    &mut shared_handle,
                )
                .unwrap();
        }

        let vulkan_image = unsafe {
            context.create_imported_image_win32(
                "SharedDrawSurface",
                &graal::ResourceMemoryInfo::DEVICE_LOCAL,
                &graal::ImageResourceCreateInfo {
                    image_type: vk::ImageType::TYPE_2D,
                    usage: vk::ImageUsageFlags::COLOR_ATTACHMENT
                        | vk::ImageUsageFlags::STORAGE
                        | vk::ImageUsageFlags::SAMPLED
                        | vk::ImageUsageFlags::TRANSFER_DST
                        | vk::ImageUsageFlags::TRANSFER_SRC,
                    format: vk::Format::B8G8R8A8_UNORM, // TODO
                    extent: vk::Extent3D {
                        width: size.0,
                        height: size.1,
                        depth: 1,
                    },
                    mip_levels: 1,
                    array_layers: 1,
                    samples: 1,
                    tiling: vk::ImageTiling::OPTIMAL,
                },
                vk::ExternalMemoryHandleTypeFlags::D3D11_TEXTURE,
                shared_handle.0 as *mut _,
                None,
            )
        };

        // --- fence
        let d3d_fence = unsafe {
            d3d11_device
                .CreateFence::<ID3D11Fence>(0, D3D11_FENCE_FLAG::D3D11_FENCE_FLAG_SHARED)
                .unwrap()
        };

        let mut fence_shared_handle = HANDLE::INVALID;
        unsafe {
            d3d_fence
                .CreateSharedHandle(ptr::null(), GENERIC_ALL, None, &mut fence_shared_handle)
                .unwrap();
        }

        // import fence in vulkan as a timeline semaphore
        let vulkan_timeline = unsafe {
            context.create_imported_semaphore_win32(
                vk::SemaphoreImportFlags::TEMPORARY,
                vk::ExternalSemaphoreHandleTypeFlags::D3D12_FENCE,
                fence_shared_handle.0 as vk::HANDLE,
                None,
            )
        };

        // ---- OLD
        /*let (target, target_shared_handle) = unsafe {
            context.create_exported_image_win32(
                "SharedDrawSurface",
                &graal::ResourceMemoryInfo::DEVICE_LOCAL,
                &graal::ImageResourceCreateInfo {
                    image_type: vk::ImageType::TYPE_2D,
                    usage: vk::ImageUsageFlags::COLOR_ATTACHMENT
                        | vk::ImageUsageFlags::STORAGE
                        | vk::ImageUsageFlags::SAMPLED
                        | vk::ImageUsageFlags::TRANSFER_DST
                        | vk::ImageUsageFlags::TRANSFER_SRC,
                    format: vk::Format::B8G8R8A8_UNORM, // TODO
                    extent: vk::Extent3D {
                        width: size.0,
                        height: size.1,
                        depth: 1,
                    },
                    mip_levels: 1,
                    array_layers: 1,
                    samples: 1,
                    tiling: vk::ImageTiling::OPTIMAL,
                },
                vk::ExternalMemoryHandleTypeFlags::D3D11_TEXTURE,
                ptr::null(),
                DXGI_SHARED_RESOURCE_READ | DXGI_SHARED_RESOURCE_WRITE | GENERIC_ALL,
                None,
            )
        };
        dbg!(target_shared_handle);
        // open the shared handle on the D2D side
        let target_surface = unsafe {
            let mut ptr: *mut ID3D11Texture2D = ptr::null_mut();
            check_hr(d3d11_device.OpenSharedResource1(
                target_shared_handle,
                &ID3D11Texture2D::uuidof(),
                &mut ptr as *mut _ as *mut *mut c_void,
            ))
            .unwrap();
            ComPtr::from_raw(ptr)
        };
        let target_surface: ComPtr<IDXGISurface> = target_surface.cast().unwrap();*/

        // create the target D2D bitmap
        let dpi = (96.0 * scale_factor) as f32;
        let props = D2D1_BITMAP_PROPERTIES1 {
            pixelFormat: D2D1_PIXEL_FORMAT {
                format: DXGI_FORMAT::DXGI_FORMAT_B8G8R8A8_UNORM, // TODO
                alphaMode: D2D1_ALPHA_MODE::D2D1_ALPHA_MODE_IGNORE,
            },
            dpiX: dpi,
            dpiY: dpi,
            bitmapOptions: D2D1_BITMAP_OPTIONS::D2D1_BITMAP_OPTIONS_TARGET
                | D2D1_BITMAP_OPTIONS::D2D1_BITMAP_OPTIONS_CANNOT_DRAW,
            colorContext: None,
        };

        // create target bitmap
        let d2d_bitmap = unsafe {
            let mut bitmap = None;
            d2d_device_context
                .CreateBitmapFromDxgiSurface(&dxgi_surface, &props, &mut bitmap)
                .and_some(bitmap)
                .unwrap()
        };

        SharedDrawSurface {
            vulkan_image,
            shared_handle,
            fence_shared_handle,
            dxgi_surface,
            dpi,
            d2d_bitmap,
            d3d_fence,
            vulkan_timeline
        }
    }
}

impl Drop for SharedDrawSurface {
    fn drop(&mut self) {
        let mut context = Platform::instance().gpu_context().lock().unwrap();
        context.destroy_image(self.vulkan_image.id);
        // TODO close the shared handle?
    }
}

pub struct DrawSurface {
    /// Direct2D device context (independent of the surface)
    d2d_device_context: ID2D1DeviceContext,
    /// Fence used for synchronizing between D3D11 and vulkan (payload shared with `vk_semaphore`)
    //d3d11_fence: ID3D11Fence,
    /// Semaphore for synchronization on the vulkan side, imported from `d3d11_fence`.
    //vk_semaphore: vk::Semaphore,
    surface: Option<SharedDrawSurface>,
}

impl DrawSurface {
    pub fn new(size: (u32, u32), scale_factor: f64) -> DrawSurface {
        // create the device context
        let mut d2d_device_context = None;
        let d2d_device_context = unsafe {
            Platform::instance()
                .d2d_device
                .CreateDeviceContext(
                    D2D1_DEVICE_CONTEXT_OPTIONS::D2D1_DEVICE_CONTEXT_OPTIONS_NONE,
                    &mut d2d_device_context,
                )
                .and_some(d2d_device_context)
                .unwrap()
        };

        //let d3d_fence

        let surface = SharedDrawSurface::new(&d2d_device_context, size, scale_factor);

        unsafe {
            // set the target on the DC
            d2d_device_context.SetTarget(&surface.d2d_bitmap);
            d2d_device_context.SetDpi(surface.dpi, surface.dpi);
        }

        DrawSurface {
            d2d_device_context,
            surface: Some(surface),
        }
    }

    // TODO:
    // * D2D: flush?
    // * D3D: wait for fence
    // * D2D: draw
    // * D2D: flush
    // * D2D: fence signal
    // * Vulkan: fence wait
    // * Vulkan: copy to swapchain
    // * Vulkan: fence signal
    //
    // Two types:
    // - Draw2DContext(&'a mut DrawSurface): provides a drawing target
    //      - need access to the D3D immediate mode context
    // - Draw3DContext(&graal::Frame, &'a mut DrawSurface): provides access to the underlying vulkan image
    //
    // Since both borrow mutably, they can't be alive at the same time
    // - however, it's a bit too easy to just copy the vulkan ImageInfo

    /*pub fn acquire(&self, frame: &graal::Frame) {
        let semaphore = self.surface.unwrap().vulkan_timeline;
        frame.add_graphics_pass("draw surface acquire", |pass| {
            pass.add_external_semaphore_wait(semaphore, vk::PipelineStageFlags::ALL_COMMANDS);
        });
    }*/

    pub fn resize(&mut self, size: (u32, u32), scale_factor: f64) {
        // release the old surface
        unsafe {
            self.d2d_device_context.SetTarget(None);
        }
        self.surface = None;

        let new_surface = SharedDrawSurface::new(&self.d2d_device_context, size, scale_factor);

        unsafe {
            // set the target on the DC
            self.d2d_device_context.SetTarget(&new_surface.d2d_bitmap);
            self.d2d_device_context
                .SetDpi(new_surface.dpi, new_surface.dpi);
        }

        self.surface = Some(new_surface);
    }

    pub fn image(&self) -> graal::ImageInfo {
        self.surface.as_ref().unwrap().vulkan_image
    }
}

struct SurfaceDrawContext {}

/// Encapsulates a Win32 window and associated resources for drawing to it.
pub struct PlatformWindow {
    window: Window,
    hwnd: HWND,
    hinstance: HINSTANCE,
    surface: vk::SurfaceKHR,
    swap_chain: graal::SwapchainInfo,
    swap_chain_width: u32,
    swap_chain_height: u32,
}

impl PlatformWindow {
    /// Returns the underlying winit [`Window`].
    ///
    /// [`Window`]: winit::Window
    pub fn window(&self) -> &Window {
        &self.window
    }

    /// Returns the underlying winit [`WindowId`].
    /// Equivalent to calling `self.window().id()`.
    ///
    /// [`WindowId`]: winit::WindowId
    pub fn id(&self) -> WindowId {
        self.window.id()
    }

    /// Returns the rendering context associated to this window.
    pub fn gpu_context(&self) -> &GpuContext {
        Platform::instance().gpu_context()
    }

    /// Returns the current swap chain size in physical pixels.
    pub fn swap_chain_size(&self) -> (u32, u32) {
        (self.swap_chain_width, self.swap_chain_height)
    }

    /// Resizes the swap chain and associated resources of the window.
    ///
    /// Must be called whenever winit sends a resize message.
    pub fn resize(&mut self, (width, height): (u32, u32)) {
        let platform = Platform::instance();

        trace!("resizing swap chain: {}x{}", width, height);

        // resizing to 0x0 will fail, so don't bother
        if width == 0 || height == 0 {
            return;
        }

        unsafe {
            platform
                .gpu_context()
                .lock()
                .unwrap()
                .resize_swapchain(self.swap_chain.id, (width, height));
        }

        self.swap_chain_width = width;
        self.swap_chain_height = height;
    }

    /// Returns the swap chain object for the window.
    pub fn swap_chain(&self) -> graal::SwapchainInfo {
        self.swap_chain
    }

    /// Creates a new window from the options given in the provided [`WindowBuilder`].
    ///
    /// To create the window with an OpenGL context, `with_gl` should be `true`.
    ///
    /// [`WindowBuilder`]: winit::WindowBuilder
    pub fn new(
        event_loop: &EventLoopWindowTarget<()>,
        mut builder: WindowBuilder,
        parent_window: Option<&PlatformWindow>,
    ) -> Result<PlatformWindow> {
        let platform = Platform::instance();

        if let Some(parent_window) = parent_window {
            builder = builder.with_parent_window(parent_window.hwnd.0 as *mut _);
        }
        let window = builder.build(event_loop).map_err(Error::Winit)?;

        // create a swap chain for the window
        let surface = graal::surface::get_vulkan_surface(window.raw_window_handle());
        let swapchain_size = window.inner_size().into();
        let swap_chain = unsafe {
            platform
                .gpu_context()
                .lock()
                .unwrap()
                .create_swapchain(surface, swapchain_size)
        };

        let hinstance = HINSTANCE(window.hinstance() as isize);
        let hwnd = HWND(window.hwnd() as isize);

        let pw = PlatformWindow {
            window,
            hwnd,
            hinstance,
            surface,
            swap_chain,
            swap_chain_width: swapchain_size.0,
            swap_chain_height: swapchain_size.1,
        };

        Ok(pw)
    }
}
