//! Platform-specific window creation
use crate::{application::Application, drawing::Point, error::Error, Menu};
use graal::{vk, vk::Handle};
use raw_window_handle::HasRawWindowHandle;
use skia_safe::gpu::vk as skia_vk;
use skia_vk::GetProcOf;
use std::ptr;
use windows::Win32::{
    Foundation::{BOOL, HINSTANCE, HWND, POINT},
    Graphics::Gdi::ClientToScreen,
    UI::WindowsAndMessaging::{DestroyMenu, SetMenu, TrackPopupMenu, HMENU, TPM_LEFTALIGN},
};
use winit::{
    event_loop::EventLoopWindowTarget,
    platform::windows::{WindowBuilderExtWindows, WindowExtWindows},
    window::{WindowBuilder, WindowId},
};

//const SWAP_CHAIN_BUFFERS: u32 = 2;

/*/// Context object to draw on a window.
///
/// It implicitly derefs to [`DrawContext`], which has methods to draw primitives on the
/// window surface.
///
/// [`DrawContext`]: crate::drawing::context::DrawContext
pub struct WindowDrawContext<'a> {
    window: &'a mut PlatformWindow,
    draw_context: DrawContext,
}

impl<'a> WindowDrawContext<'a> {
    /// Creates a new [`WindowDrawContext`] for the specified window, allowing to draw on the window.
    pub fn new(window: &'a mut PlatformWindow) -> WindowDrawContext<'a> {
        let platform = Platform::instance();
        let d2d_device_context = &platform.0.d2d_device_context;

        let swap_chain = &window.swap_chain;
        let backbuffer = unsafe { swap_chain.GetBuffer::<IDXGISurface>(0).unwrap() };
        let dpi = 96.0 * window.window.scale_factor() as f32;

        // create target bitmap
        let mut bitmap = unsafe {
            let props = D2D1_BITMAP_PROPERTIES1 {
                pixelFormat: D2D1_PIXEL_FORMAT {
                    format: DXGI_FORMAT::DXGI_FORMAT_R8G8B8A8_UNORM,
                    alphaMode: D2D1_ALPHA_MODE::D2D1_ALPHA_MODE_IGNORE,
                },
                dpiX: dpi,
                dpiY: dpi,
                bitmapOptions: D2D1_BITMAP_OPTIONS::D2D1_BITMAP_OPTIONS_TARGET
                    | D2D1_BITMAP_OPTIONS::D2D1_BITMAP_OPTIONS_CANNOT_DRAW,
                colorContext: None,
            };
            let mut bitmap = None;
            d2d_device_context
                .CreateBitmapFromDxgiSurface(backbuffer, &props, &mut bitmap)
                .and_some(bitmap)
                .expect("CreateBitmapFromDxgiSurface failed")
        };

        // create draw context
        let draw_context = unsafe {
            // set the target on the DC
            d2d_device_context.SetTarget(bitmap);
            d2d_device_context.SetDpi(dpi, dpi);
            // the draw context acquires shared ownership of the device context, but that's OK since we borrow the window,
            // so we can't create another WindowDrawContext that would conflict with it.
            DrawContext::from_device_context(
                platform.0.d2d_factory.0.clone(),
                d2d_device_context.0.clone(),
            )
        };

        WindowDrawContext {
            window,
            draw_context,
        }
    }

    /// Returns the [`PlatformWindow`] that is being drawn to.
    pub fn window(&self) -> &PlatformWindow {
        self.window
    }
}

impl<'a> Drop for WindowDrawContext<'a> {
    fn drop(&mut self) {
        // set the target to null to release the borrow of the backbuffer surface
        // (otherwise it will fail to resize)
        unsafe {
            self.ctx.SetTarget(None);
        }
    }
}

impl<'a> Deref for WindowDrawContext<'a> {
    type Target = DrawContext;
    fn deref(&self) -> &DrawContext {
        &self.draw_context
    }
}

impl<'a> DerefMut for WindowDrawContext<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.draw_context
    }
}*/

fn skia_get_proc_addr(of: skia_vk::GetProcOf) -> skia_vk::GetProcResult {
    unsafe {
        let entry = graal::get_vulkan_entry();
        let instance = graal::get_vulkan_instance();

        match of {
            GetProcOf::Instance(instance, name) => entry
                .get_instance_proc_addr(graal::vk::Instance::from_raw(instance as u64), name)
                .unwrap()
                as skia_vk::GetProcResult,
            GetProcOf::Device(device, name) => instance
                .get_device_proc_addr(graal::vk::Device::from_raw(device as u64), name)
                .unwrap() as skia_vk::GetProcResult,
        }
    }
}

unsafe fn create_skia_vulkan_backend_context(
    device: &graal::Device,
) -> skia_safe::gpu::vk::BackendContext<'static> {
    let vk_device = device.device.handle();
    let vk_instance = graal::get_vulkan_instance().handle();
    let vk_physical_device = device.physical_device();
    let (vk_queue, vk_queue_family_index) = device.graphics_queue();
    let instance_extensions = graal::get_instance_extensions();

    let mut ctx = skia_vk::BackendContext::new_with_extensions(
        vk_instance.as_raw() as *mut _,
        vk_physical_device.as_raw() as *mut _,
        vk_device.as_raw() as *mut _,
        (vk_queue.as_raw() as *mut _, vk_queue_family_index as usize),
        &skia_get_proc_addr,
        instance_extensions,
        &[],
    );

    ctx.set_max_api_version(skia_vk::Version::new(1, 0, 0));
    ctx
}

/// Encapsulates a Win32 window and associated resources for drawing to it.
pub struct Window {
    window: winit::window::Window,
    hwnd: HWND,
    hinstance: HINSTANCE,
    //swap_chain: IDXGISwapChain1,
    surface: vk::SurfaceKHR,
    menu: Option<HMENU>,
    swap_chain: graal::Swapchain,
    swap_chain_width: u32,
    swap_chain_height: u32,
    skia_backend_context: skia_safe::gpu::vk::BackendContext<'static>,
    skia_recording_context: skia_safe::gpu::DirectContext,
}

impl Window {
    /// Returns the underlying winit [`Window`].
    ///
    /// [`Window`]: winit::Window
    pub fn window(&self) -> &winit::window::Window {
        &self.window
    }

    /// Returns the underlying winit [`WindowId`].
    /// Equivalent to calling `self.window().id()`.
    ///
    /// [`WindowId`]: winit::WindowId
    pub fn id(&self) -> WindowId {
        self.window.id()
    }

    /// Sets this window's main menu bar.
    pub fn set_menu(&mut self, new_menu: Option<Menu>) {
        unsafe {
            // SAFETY: TODO
            if let Some(current_menu) = self.menu.take() {
                SetMenu(self.hwnd, None);
                DestroyMenu(current_menu);
            }
            if let Some(menu) = new_menu {
                let hmenu = menu.into_hmenu();
                SetMenu(self.hwnd, hmenu);
                self.menu = Some(hmenu);
            }
        }
    }

    /// Shows a context menu.
    pub fn show_context_menu(&self, menu: Menu, at: Point) {
        unsafe {
            let hmenu = menu.into_hmenu();
            let scale_factor = self.window.scale_factor();
            let x = at.x * scale_factor;
            let y = at.y * scale_factor;
            let mut point = POINT {
                x: x as i32,
                y: y as i32,
            };
            ClientToScreen(self.hwnd, &mut point);
            if TrackPopupMenu(
                hmenu,
                TPM_LEFTALIGN,
                point.x,
                point.y,
                0,
                self.hwnd,
                ptr::null(),
            ) == BOOL::from(false)
            {
                tracing::warn!("failed to track popup menu");
            }
        }
    }

    /// Returns the current swap chain size in physical pixels.
    pub fn swap_chain_size(&self) -> (u32, u32) {
        (self.swap_chain_width, self.swap_chain_height)
    }

    /// Resizes the swap chain and associated resources of the window.
    ///
    /// Must be called whenever winit sends a resize message.
    pub fn resize(&mut self, (width, height): (u32, u32)) {
        let app = Application::instance();

        tracing::trace!("resizing swap chain: {}x{}", width, height);

        // resizing to 0x0 will fail, so don't bother
        if width == 0 || height == 0 {
            return;
        }

        unsafe {
            let device = app.gpu_device();
            device.resize_swapchain(&mut self.swap_chain, (width, height));
        }

        self.swap_chain_width = width;
        self.swap_chain_height = height;
    }

    /// Returns the swap chain of this window.
    pub fn swap_chain(&self) -> &graal::Swapchain {
        &self.swap_chain
    }

    pub fn skia_backend_context(&self) -> &skia_safe::gpu::vk::BackendContext<'static> {
        &self.skia_backend_context
    }

    pub fn skia_recording_context(&self) -> &skia_safe::gpu::DirectContext {
        &self.skia_recording_context
    }

    /// Creates a new window from the options given in the provided [`WindowBuilder`].
    ///
    /// To create the window with an OpenGL context, `with_gl` should be `true`.
    ///
    /// [`WindowBuilder`]: winit::WindowBuilder
    pub fn new(
        event_loop: &EventLoopWindowTarget<()>,
        mut builder: WindowBuilder,
        parent_window: Option<&Window>,
    ) -> Result<Window, Error> {
        let app = Application::instance();

        if let Some(parent_window) = parent_window {
            builder = builder.with_parent_window(parent_window.hwnd as *mut _);
        }
        let window = builder.build(event_loop).map_err(Error::Winit)?;

        /*let dxgi_factory = &platform.0.dxgi_factory;
        let d3d11_device = &platform.0.d3d11_device;

        // create a DXGI swap chain for the window
        let hinstance = HINSTANCE(window.hinstance() as isize);
        let hwnd = HWND(window.hwnd() as isize);
        let (width, height): (u32, u32) = window.inner_size().into();

        // TODO flip effects
        let swap_effect = DXGI_SWAP_EFFECT::DXGI_SWAP_EFFECT_SEQUENTIAL;

        // create the swap chain
        let swap_chain = unsafe {
            let mut swap_chain = None;

            let swap_chain_desc = DXGI_SWAP_CHAIN_DESC1 {
                Width: width,
                Height: height,
                Format: DXGI_FORMAT::DXGI_FORMAT_R8G8B8A8_UNORM,
                Stereo: false.into(),
                SampleDesc: DXGI_SAMPLE_DESC {
                    Count: 1,
                    Quality: 0,
                },
                BufferUsage: DXGI_USAGE_RENDER_TARGET_OUTPUT,
                BufferCount: SWAP_CHAIN_BUFFERS,
                Scaling: DXGI_SCALING::DXGI_SCALING_STRETCH,
                SwapEffect: swap_effect,
                AlphaMode: DXGI_ALPHA_MODE::DXGI_ALPHA_MODE_UNSPECIFIED,
                Flags: 0,
            };

            dxgi_factory
                .CreateSwapChainForHwnd(
                    d3d11_device.0.clone(),
                    hwnd,
                    &swap_chain_desc,
                    ptr::null(),
                    None,
                    &mut swap_chain,
                )
                .and_some(swap_chain)
                .expect("failed to create swap chain")
        };*/

        // create a swap chain for the window
        let device = app.gpu_device();
        let surface = graal::surface::get_vulkan_surface(window.raw_window_handle());
        let swapchain_size = window.inner_size().into();
        // ensure that the surface can be drawn to with the device that we created. must be called to
        // avoid validation errors.
        unsafe {
            assert!(device.is_compatible_for_presentation(surface));
        }
        let swap_chain = unsafe { device.create_swapchain(surface, swapchain_size) };

        let skia_backend_context = unsafe { create_skia_vulkan_backend_context(device) };
        let recording_context_options = skia_safe::gpu::ContextOptions::new();
        let skia_recording_context = skia_safe::gpu::DirectContext::new_vulkan(
            &skia_backend_context,
            &recording_context_options,
        )
        .unwrap();

        let hinstance = window.hinstance() as HINSTANCE;
        let hwnd = window.hwnd() as HWND;

        let pw = Window {
            window,
            hwnd,
            hinstance,
            surface,
            // TODO menu initializer
            menu: None,
            swap_chain,
            swap_chain_width: swapchain_size.0,
            swap_chain_height: swapchain_size.1,
            skia_backend_context,
            skia_recording_context,
        };

        Ok(pw)
    }

    /*pub fn draw_skia(&mut self, f: impl FnOnce(&mut skia_safe::Canvas)) {
        let context = Platform::instance().gpu_context();
        let mut context = context.lock().unwrap();

        let swapchain_image = unsafe { self.swap_chain.acquire_next_image(&mut context) };

        let swap_chain_width = self.swap_chain_width;
        let swap_chain_height = self.swap_chain_height;

        // do the dance required to create a skia context on the swapchain image
        let graphics_queue_family = context.device().graphics_queue().1;

        // start our frame
        context.start_frame(Default::default());

        // skia may not support rendering directly to the swapchain image (for example, it doesn't seem to support BGRA8888_SRGB).
        // so allocate a separate image to use as a render target, then copy.
        let skia_image_usage_flags = graal::vk::ImageUsageFlags::COLOR_ATTACHMENT
            | graal::vk::ImageUsageFlags::TRANSFER_SRC
            | graal::vk::ImageUsageFlags::TRANSFER_DST;
        let skia_image_format = graal::vk::Format::R16G16B16A16_SFLOAT;

        let skia_image = .context().create_image(
            "skia render target",
            MemoryLocation::GpuOnly,
            &graal::ImageResourceCreateInfo {
                image_type: graal::vk::ImageType::TYPE_2D,
                usage: skia_image_usage_flags,
                format: skia_image_format,
                extent: graal::vk::Extent3D {
                    width: swap_chain_width,
                    height: swap_chain_height,
                    depth: 1,
                },
                mip_levels: 1,
                array_layers: 1,
                samples: 1,
                tiling: graal::vk::ImageTiling::OPTIMAL,
            },
        );

        let skia_recording_context = &mut self.skia_recording_context;

        // create the skia render pass
        frame.add_graphics_pass("skia render", |pass| {
            // register access by skia, just assume how it's going to be used
            pass.register_image_access(
                skia_image.id,
                graal::vk::AccessFlags::MEMORY_READ | graal::vk::AccessFlags::MEMORY_WRITE,
                graal::vk::PipelineStageFlags::ALL_COMMANDS,
                graal::vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                graal::vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
            );

            pass.set_queue_commands(move |cctx, _queue| {
                // now do something with skia or whatever
                let skia_image_info = skia_vk::ImageInfo {
                    image: skia_image.handle.as_raw() as *mut _,
                    alloc: Default::default(),
                    tiling: skia_vk::ImageTiling::OPTIMAL,
                    layout: skia_vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                    format: unsafe { mem::transmute(skia_image_format.as_raw()) }, // SAFETY: it's a VkFormat, and hopefully skia_vk has a definition with all the latest enumerators...
                    image_usage_flags: skia_image_usage_flags.as_raw(),
                    sample_count: 1,
                    level_count: 1,
                    current_queue_family: graphics_queue_family,
                    protected: skia_safe::gpu::Protected::No,
                    ycbcr_conversion_info: Default::default(),
                    sharing_mode: skia_vk::SharingMode::EXCLUSIVE,
                };
                let render_target = skia_safe::gpu::BackendRenderTarget::new_vulkan(
                    (swap_chain_width as i32, swap_chain_height as i32),
                    1,
                    &skia_image_info,
                );
                let mut surface = skia_safe::Surface::from_backend_render_target(
                    skia_recording_context,
                    &render_target,
                    skia_safe::gpu::SurfaceOrigin::TopLeft,
                    skia_safe::ColorType::RGBAF16Norm, // ???
                    sk::ColorSpace::new_srgb_linear(),
                    Some(&sk::SurfaceProps::new(
                        Default::default(),
                        sk::PixelGeometry::RGBH,
                    )),
                )
                .unwrap();

                let canvas = surface.canvas();
                f(canvas);
                surface.flush_and_submit();
            });
        });

        // copy skia result to swapchain image
        graal::utils::blit_images(
            &frame,
            skia_image,
            swapchain_image.image_info,
            (self.swap_chain_width, self.swap_chain_height),
            graal::vk::ImageAspectFlags::COLOR,
        );

        // present
        frame.present("present", &swapchain_image);
        frame.finish();
    }*/

    /*pub fn present(&mut self) {
        unsafe {
            if let Err(err) = self.swap_chain.Present(1, 0).ok() {
                tracing::error!("IDXGISwapChain::Present failed: {}", err)
            }
        }
    }*/
}
