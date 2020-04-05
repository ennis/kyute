//! Platform-specific window creation
use crate::drawing::target::RenderTarget;
use crate::error::{self, Error, Result};
use crate::opengl;
use crate::opengl::api::gl;
use crate::opengl::api::gl::types::*;
use crate::opengl::api::wgl;
use crate::opengl::api::Gl;
use crate::opengl::api::Wgl;
use crate::platform::Platform;
use crate::platform::PlatformState;

use glutin::platform::windows::RawContextExt;
use glutin::{ContextBuilder, PossiblyCurrent, RawContext};
use log::{error, info, trace};

use std::os::raw::c_void;
use std::ptr;
use std::rc::Rc;

use winit::event_loop::EventLoopWindowTarget;
use winit::platform::windows::WindowExtWindows;
use winit::window::{Window, WindowBuilder, WindowId};

use winapi::shared::dxgi::*;
use winapi::shared::dxgi1_2::*;
use winapi::shared::dxgiformat::*;
use winapi::shared::dxgitype::*;
use winapi::shared::minwindef::HINSTANCE;
use winapi::shared::windef::HWND;
use winapi::shared::winerror::SUCCEEDED;
use winapi::um::d2d1::*;
use winapi::um::d3d11::*;
use winapi::um::dcommon::*;
use winapi::um::errhandlingapi::GetLastError;
use winapi::Interface;

use std::ops::{Deref, DerefMut};
use wio::com::ComPtr;

/// DirectX-OpenGL interop state.
struct DxGlInterop {
    gl: Gl,
    wgl: Wgl,
    /// Interop device handle
    device: wgl::types::HANDLE,
    /// Staging texture
    staging: Option<ComPtr<ID3D11Texture2D>>,
    /// Interop handle for the OpenGL drawing target.
    /// If `staging_d3d11` is not None, then this is a handle to the staging texture, otherwise
    /// it's a handle to the true backbuffer.
    target: wgl::types::HANDLE,
    renderbuffer: GLuint,
    framebuffer: GLuint,
}

impl Drop for DxGlInterop {
    fn drop(&mut self) {
        unsafe {
            self.gl.DeleteFramebuffers(1, &mut self.framebuffer);
            self.gl.DeleteRenderbuffers(1, &mut self.renderbuffer);
        }
    }
}

/// Contains resources that should be re-created when the swap chain of a window changes
/// (e.g. on resize).
struct SwapChainResources {
    backbuffer: ComPtr<ID3D11Texture2D>,
    interop: Option<DxGlInterop>,
}

impl SwapChainResources {
    unsafe fn new(
        swap_chain: &ComPtr<IDXGISwapChain1>,
        _device: &ComPtr<ID3D11Device>,
        _width: u32,
        _height: u32,
    ) -> Result<SwapChainResources> {
        let mut buffer: *mut ID3D11Texture2D = ptr::null_mut();
        let hr = swap_chain.GetBuffer(
            0,
            &ID3D11Texture2D::uuidof(),
            &mut buffer as *mut _ as *mut *mut c_void,
        );
        error::wrap_hr(hr, || SwapChainResources {
            backbuffer: ComPtr::from_raw(buffer),
            interop: None,
        })
    }

    unsafe fn with_gl_interop(
        swap_chain: &ComPtr<IDXGISwapChain1>,
        device: &ComPtr<ID3D11Device>,
        gl: Gl,
        wgl: Wgl,
        width: u32,
        height: u32,
        use_staging_texture: bool,
    ) -> Result<SwapChainResources> {
        let mut res = Self::new(swap_chain, device, width, height)?;

        let interop_device = wgl.DXOpenDeviceNV(device.as_raw() as *mut _);
        if interop_device.is_null() {
            error!("Could not create OpenGL-DirectX interop.");
            return Err(Error::OpenGlInteropError);
        }

        let mut renderbuffer = 0;
        gl.GenRenderbuffers(1, &mut renderbuffer);

        let (staging, interop_target) = if use_staging_texture {
            // use staging texture because directly sharing the swap chain buffer when using FLIP_*
            // swap effects seems to cause problems.
            let mut staging = ptr::null_mut();
            let staging_desc = D3D11_TEXTURE2D_DESC {
                Width: width,
                Height: height,
                MipLevels: 1,
                ArraySize: 1,
                Format: DXGI_FORMAT_R8G8B8A8_UNORM,
                SampleDesc: DXGI_SAMPLE_DESC {
                    Count: 1,
                    Quality: 0,
                },
                Usage: D3D11_USAGE_DEFAULT,
                BindFlags: D3D11_BIND_RENDER_TARGET,
                CPUAccessFlags: 0,
                MiscFlags: 0,
            };
            let hr = device.CreateTexture2D(&staging_desc, ptr::null(), &mut staging);
            if !SUCCEEDED(hr) {
                error!("Could not create staging texture.");
                return Err(hr.into());
            }
            let staging = ComPtr::from_raw(staging);

            let interop_staging = wgl.DXRegisterObjectNV(
                interop_device,
                staging.as_raw() as *mut _,
                renderbuffer,
                gl::RENDERBUFFER,
                wgl::ACCESS_READ_WRITE_NV,
            );
            (Some(staging), interop_staging)
        } else {
            // directly share the swap chain buffer (this may cause problems with FLIP_* swap effects)
            let interop_backbuffer = wgl.DXRegisterObjectNV(
                interop_device,
                res.backbuffer.as_raw() as *mut _,
                renderbuffer,
                gl::RENDERBUFFER,
                wgl::ACCESS_READ_WRITE_NV,
            );
            (None, interop_backbuffer)
        };

        if interop_target.is_null() {
            gl.DeleteRenderbuffers(1, &renderbuffer);
            return Err(Error::OpenGlInteropError);
        }

        // create a framebuffer that points to the swap chain buffer
        let mut framebuffer = 0;
        gl.CreateFramebuffers(1, &mut framebuffer);
        gl.NamedFramebufferRenderbuffer(
            framebuffer,
            gl::COLOR_ATTACHMENT0,
            gl::RENDERBUFFER,
            renderbuffer,
        );

        let fb_status = gl.CheckNamedFramebufferStatus(framebuffer, gl::DRAW_FRAMEBUFFER);
        if fb_status != gl::FRAMEBUFFER_COMPLETE {
            // don't forget to release the GL resources still lying around.
            wgl.DXUnregisterObjectNV(interop_device, interop_target);
            gl.DeleteRenderbuffers(1, &renderbuffer);
            gl.DeleteFramebuffers(1, &framebuffer);
            error!("OpenGL framebuffer not complete");
            return Err(Error::OpenGlInteropError);
        }

        res.interop = Some(DxGlInterop {
            gl,
            wgl,
            device: interop_device,
            staging,
            target: interop_target,
            renderbuffer,
            framebuffer,
        });

        Ok(res)
    }
}

fn check_win32_last_error(returned: i32, function: &str) {
    unsafe {
        if returned == 0 {
            let err = GetLastError();
            panic!("{} failed, GetLastError={:08x}", function, err);
        }
    }
}

impl Drop for SwapChainResources {
    fn drop(&mut self) {
        if let Some(ref mut interop) = self.interop {
            unsafe {
                check_win32_last_error(
                    interop
                        .wgl
                        .DXUnregisterObjectNV(interop.device, interop.target),
                    "wglDXUnregisterObjectNV",
                );
                check_win32_last_error(
                    interop.wgl.DXCloseDeviceNV(interop.device),
                    "wglDXCloseDeviceNV",
                );
            }
        }
    }
}

pub struct GlState {
    context: RawContext<PossiblyCurrent>,
    gl: Gl,
    wgl: Wgl,
}

const SWAP_CHAIN_BUFFERS: u32 = 2;
//const USE_INTEROP_STAGING_TEXTURE: bool = false;

/// Guard object that holds a lock on an interop OpenGL context.
pub struct OpenGlDrawContext<'a> {
    gl: &'a Gl,
    wgl: &'a Wgl,
    interop: &'a mut DxGlInterop,
    backbuffer: &'a ComPtr<ID3D11Texture2D>,
    d3d11_ctx: &'a ComPtr<ID3D11DeviceContext>,
}

impl<'a> OpenGlDrawContext<'a> {
    pub fn new(w: &'a mut PlatformWindow) -> OpenGlDrawContext<'a> {
        let gl_state = w.gl.as_mut().expect("a GL context was not requested");
        let swap_res = w
            .swap_res
            .as_mut()
            .expect("the swap chain is not initialized");
        let backbuffer = &swap_res.backbuffer;
        let interop = swap_res
            .interop
            .as_mut()
            .expect("DX-GL interop not initialized");

        let gl = &gl_state.gl;
        let wgl = &gl_state.wgl;

        unsafe {
            // signals to the interop device that OpenGL is going to use the resource specified by the
            // given interop handle.
            wgl.DXLockObjectsNV(interop.device, 1, &mut interop.target);
        }

        OpenGlDrawContext {
            gl,
            wgl,
            interop,
            backbuffer,
            d3d11_ctx: &w.shared.d3d11_device_context,
        }
    }

    /// Returns the OpenGL functions.
    pub fn functions(&self) -> &Gl {
        self.gl
    }

    /// Returns the framebuffer associated to the window surface.
    pub fn framebuffer(&self) -> GLuint {
        self.interop.framebuffer
    }
}

impl<'a> Drop for OpenGlDrawContext<'a> {
    fn drop(&mut self) {
        // finished using the resource
        unsafe {
            self.wgl
                .DXUnlockObjectsNV(self.interop.device, 1, &mut self.interop.target);
        }

        if let Some(ref staging_d3d11) = self.interop.staging {
            // copy staging tex to actual backbuffer
            let backbuffer = self.backbuffer.cast::<ID3D11Resource>().unwrap();
            let staging = staging_d3d11.cast::<ID3D11Resource>().unwrap();
            unsafe {
                self.d3d11_ctx
                    .CopyResource(staging.as_raw(), backbuffer.as_raw());
            }
        }
    }
}

/// Context object to draw on a window.
///
/// It implicitly derefs to [`RenderTarget`], which has methods to draw primitives on the
/// window surface.
///
/// [`RenderTarget`]: crate::drawing::target::RenderTarget
pub struct DrawContext<'a> {
    window: &'a mut PlatformWindow,
    target: RenderTarget,
}

impl<'a> DrawContext<'a> {
    /// Creates a new [`DrawContext`] for the specified window, allowing to draw on the window.
    pub fn new(window: &'a mut PlatformWindow) -> DrawContext<'a> {
        let d2d = &window.shared.d2d_factory;
        let swap_res = window.swap_res.as_ref().unwrap();
        let dxgi_buffer = swap_res.backbuffer.cast::<IDXGISurface>().unwrap();

        let dpi = 96.0 * window.window.scale_factor() as f32;
        let props = D2D1_RENDER_TARGET_PROPERTIES {
            _type: D2D1_RENDER_TARGET_TYPE_DEFAULT,
            pixelFormat: D2D1_PIXEL_FORMAT {
                format: DXGI_FORMAT_R8G8B8A8_UNORM,
                alphaMode: D2D1_ALPHA_MODE_IGNORE,
            },
            dpiX: dpi,
            dpiY: dpi,
            usage: D2D1_RENDER_TARGET_USAGE_NONE,
            minLevel: D2D1_FEATURE_LEVEL_DEFAULT,
        };

        let mut target = unsafe {
            let mut render_target: *mut ID2D1RenderTarget = ptr::null_mut();
            let hr =
                d2d.CreateDxgiSurfaceRenderTarget(dxgi_buffer.as_raw(), &props, &mut render_target);
            if !SUCCEEDED(hr) {
                panic!("CreateDxgiSurfaceRenderTarget failed: {}", Error::HResultError(hr));
            }
            // start drawing immediately
            RenderTarget::from_raw(d2d.clone().up(), render_target)
        };

        target.begin_draw();
        DrawContext { window, target }
    }

    /// Returns the [`PlatformWindow`] that is being drawn to.
    pub fn window(&self) -> &PlatformWindow {
        self.window
    }
}

impl<'a> Drop for DrawContext<'a> {
    fn drop(&mut self) {
        self.target.end_draw()
    }
}

impl<'a> Deref for DrawContext<'a> {
    type Target = RenderTarget;

    fn deref(&self) -> &RenderTarget {
        &self.target
    }
}

impl<'a> DerefMut for DrawContext<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.target
    }
}

/// Encapsulates a Win32 window and associated resources for drawing to it.
pub struct PlatformWindow {
    // we don't really need it to  have a shared ref here, but
    // this way we can avoid passing WindowCtx everywhere.
    shared: Rc<PlatformState>,
    window: Window,
    hwnd: HWND,
    hinstance: HINSTANCE,
    swap_chain: ComPtr<IDXGISwapChain1>,
    swap_res: Option<SwapChainResources>,
    gl: Option<GlState>,
    interop_needs_staging: bool,
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

    /// Resizes the swap chain and associated resources of the window.
    ///
    /// Must be called whenever winit sends a resize message.
    pub fn resize(&mut self, (width, height): (u32, u32)) {
        trace!("resizing swap chain: {}x{}", width, height);

        // signal the GL context as well if we have one
        if let Some(ref mut gl) = self.gl {
            gl.context.resize((width, height).into());
        }

        unsafe {
            // explicitly release swap-chain dependent resources
            self.swap_res = None;

            // resize the swap chain
            let hr = self
                .swap_chain
                .ResizeBuffers(0, width, height, DXGI_FORMAT_UNKNOWN, 0);
            if !SUCCEEDED(hr) {
                // it fails sometimes...
                error!(
                    "IDXGISwapChain1::ResizeBuffers failed: {}",
                    Error::HResultError(hr)
                );
                return;
            }

            // re-create all resources that depend on the swap chain
            let new_swap_res = if let Some(ref mut gl) = self.gl {
                SwapChainResources::with_gl_interop(
                    &self.swap_chain,
                    &self.shared.d3d11_device,
                    gl.gl.clone(),
                    gl.wgl.clone(),
                    width,
                    height,
                    self.interop_needs_staging,
                )
            } else {
                SwapChainResources::new(&self.swap_chain, &self.shared.d3d11_device, width, height)
            };

            match new_swap_res {
                Ok(r) => self.swap_res = Some(r),
                Err(e) => error!("Failed to allocate swap chain resources: {}", e),
            }
        }
    }

    /// Creates a new window from the options given in the provided [`WindowBuilder`].
    ///
    /// To create the window with an OpenGL context, `with_gl` should be `true`.
    ///
    /// [`WindowBuilder`]: winit::WindowBuilder
    pub fn new(
        event_loop: &EventLoopWindowTarget<()>,
        builder: WindowBuilder,
        platform: &Platform,
        with_gl: bool,
    ) -> Result<PlatformWindow> {
        // We want to be able to render 3D stuff with OpenGL, and still be able to use
        // D3D11/Direct2D/DirectWrite.
        // To do so, we use a DXGI swap chain to manage presenting. Then, using WGL_NV_DX_interop2,
        // we register the buffers of the swap chain as a renderbuffer in GL so we can use both
        // on the same render target.
        unsafe {
            // first, build the window using the provided builder
            let window = builder.build(event_loop).map_err(Error::Winit)?;

            let dxgi_factory = &platform.0.dxgi_factory;
            let d3d11_device = &platform.0.d3d11_device;
            //let dxgi_device = &d3d11_device.cast::<IDXGIDevice>().unwrap(); // shouldn't fail?

            // create a DXGI swap chain for the window
            let hinstance: HINSTANCE = window.hinstance() as HINSTANCE;
            let hwnd: HWND = window.hwnd() as HWND;
            let (width, height): (u32, u32) = window.inner_size().into();

            // it might also be better to just use Discard always
            //let swap_effect = if with_gl {
            //   SwapEffect::Discard
            //} else {
            //    SwapEffect::FlipDiscard
            //};
            let swap_effect = DXGI_SWAP_EFFECT_SEQUENTIAL;

            // OpenGL interop does not work well with FLIP_* swap effects
            // (generates a "D3D11 Device Lost" error during resizing after a while).
            // In those cases, draw on a staging texture, and then copy to the backbuffer.
            let interop_needs_staging = match swap_effect {
                DXGI_SWAP_EFFECT_FLIP_SEQUENTIAL | DXGI_SWAP_EFFECT_FLIP_DISCARD => true,
                _ => false,
            };

            if interop_needs_staging && with_gl {
                info!("FLIP_DISCARD or FLIP_SEQUENTIAL swap chains with OpenGL interop may cause crashes. \
                 Will allocate a staging target to work around this issue.");
            }

            // create the swap chain
            let mut swap_chain = ptr::null_mut();
            let swap_chain_desc = DXGI_SWAP_CHAIN_DESC1 {
                Width: 0,
                Height: 0,
                Format: DXGI_FORMAT_R8G8B8A8_UNORM,
                Stereo: 0,
                SampleDesc: DXGI_SAMPLE_DESC {
                    Count: 1,
                    Quality: 0,
                },
                BufferUsage: DXGI_USAGE_RENDER_TARGET_OUTPUT,
                BufferCount: SWAP_CHAIN_BUFFERS,
                Scaling: DXGI_SCALING_STRETCH,
                SwapEffect: swap_effect,
                AlphaMode: DXGI_ALPHA_MODE_UNSPECIFIED,
                Flags: 0,
            };

            let hr = dxgi_factory.CreateSwapChainForHwnd(
                d3d11_device.clone().up().as_raw(),
                hwnd,
                &swap_chain_desc,
                ptr::null(),
                ptr::null_mut(),
                &mut swap_chain,
            );

            if !SUCCEEDED(hr) {
                return Err(hr.into());
            }

            let swap_chain = ComPtr::from_raw(swap_chain);

            // Create the OpenGL context
            let (swap_res, gl) = if with_gl {
                trace!("creating OpenGL context");
                let context = ContextBuilder::new()
                    .with_gl_profile(glutin::GlProfile::Core)
                    .with_gl_debug_flag(true)
                    .with_vsync(true)
                    .with_gl(glutin::GlRequest::Specific(glutin::Api::OpenGl, (4, 6)))
                    .build_raw_context(hwnd as *mut c_void)
                    .expect("failed to create OpenGL context on window");
                let context = context
                    .make_current()
                    .expect("could not make context current");
                // load GL functions
                let loader = |symbol| {
                    let ptr = context.get_proc_address(symbol) as *const _;
                    ptr
                };
                let gl = Gl::load_with(loader);
                let wgl = Wgl::load_with(loader);
                // set up a debug callback so we have a clue of what's going wrong
                opengl::init_debug_callback(&gl);
                // first-time initialization of the swap chain resources, with GL interop enabled
                let swap_res = SwapChainResources::with_gl_interop(
                    &swap_chain,
                    &d3d11_device,
                    gl.clone(),
                    wgl.clone(),
                    width,
                    height,
                    interop_needs_staging,
                )?;

                let gl = GlState { context, gl, wgl };

                (swap_res, Some(gl))
            } else {
                // no OpenGL requested for this window
                let swap_res = SwapChainResources::new(&swap_chain, &d3d11_device, width, height)?;
                (swap_res, None)
            };

            let pw = PlatformWindow {
                shared: platform.0.clone(),
                window,
                hwnd,
                hinstance,
                swap_chain,
                swap_res: Some(swap_res),
                gl,
                interop_needs_staging,
            };

            Ok(pw)
        }
    }

    pub fn present(&mut self) {
        unsafe {
            let hr = self.swap_chain.Present(1, 0);
            if !SUCCEEDED(hr) {
                error!(
                    "IDXGISwapChain::Present failed: {}",
                    Error::HResultError(hr)
                )
            }
        }
    }
}
