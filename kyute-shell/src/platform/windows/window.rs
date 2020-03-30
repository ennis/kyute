//! Platform-specific window creation
use super::PlatformState;
use super::gl_api::gl::types::*;
use super::gl_api::gl;
use super::gl_api::wgl;
use super::gl_api::Gl;
use super::gl_api::Wgl;

use anyhow::Result;
use anyhow::{Context, Error};
use glutin::platform::windows::RawContextExt;
use glutin::{ContextBuilder, PossiblyCurrent, RawContext};
use log::{error, info, trace};

use std::marker::PhantomData;
use std::os::raw::c_void;
use std::rc::Rc;
use std::{error, fmt, ptr};

use winit::event_loop::{EventLoop, EventLoopWindowTarget};
use winit::platform::windows::WindowExtWindows;
use winit::window::{Window, WindowBuilder, WindowId};

use com_wrapper::ComWrapper;
use direct3d11::device_context::IDeviceContext;
use direct3d11::enums::BindFlags;
use direct3d11::enums::Usage;
use dxgi::enums::*;
use dxgi::enums::{PresentFlags, SwapChainFlags};
use dxgi::swap_chain::swap_chain::ISwapChain;

use winapi::shared::dxgiformat::*;
use winapi::shared::minwindef::HINSTANCE;
use winapi::shared::ntdef::HRESULT;
use winapi::shared::windef::HWND;
use winapi::shared::winerror::SUCCEEDED;
use winapi::um::d2d1::*;
use winapi::um::dcommon::*;
use winapi::um::errhandlingapi::GetLastError;
use crate::platform::Platform;

#[derive(Copy, Clone)]
pub struct HResultError(pub HRESULT);

impl fmt::Debug for HResultError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

impl fmt::Display for HResultError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[HRESULT {:08X}]", self.0)
    }
}

impl error::Error for HResultError {}

fn check_hr(hr: HRESULT) -> Result<HRESULT, HResultError> {
    if !SUCCEEDED(hr) {
        Err(HResultError(hr))
    } else {
        Ok(hr)
    }
}

/// Sets up the OpenGL debug output so that we have more information in case the interop fails.
unsafe fn init_debug_callback(gl: &Gl) {
    gl.Enable(gl::DEBUG_OUTPUT);
    gl.Enable(gl::DEBUG_OUTPUT_SYNCHRONOUS);

    if gl.DebugMessageCallback.is_loaded() {
        extern "system" fn debug_callback(
            source: GLenum,
            gltype: GLenum,
            _id: GLuint,
            severity: GLenum,
            length: GLsizei,
            message: *const GLchar,
            _userParam: *mut c_void,
        ) {
            unsafe {
                use std::ffi::CStr;
                let message = CStr::from_ptr(message);
                eprintln!("{:?}", message);
                match source {
                    gl::DEBUG_SOURCE_API => eprintln!("Source: API"),
                    gl::DEBUG_SOURCE_WINDOW_SYSTEM => eprintln!("Source: Window System"),
                    gl::DEBUG_SOURCE_SHADER_COMPILER => eprintln!("Source: Shader Compiler"),
                    gl::DEBUG_SOURCE_THIRD_PARTY => eprintln!("Source: Third Party"),
                    gl::DEBUG_SOURCE_APPLICATION => eprintln!("Source: Application"),
                    gl::DEBUG_SOURCE_OTHER => eprintln!("Source: Other"),
                    _ => (),
                }

                match gltype {
                    gl::DEBUG_TYPE_ERROR => eprintln!("Type: Error"),
                    gl::DEBUG_TYPE_DEPRECATED_BEHAVIOR => eprintln!("Type: Deprecated Behaviour"),
                    gl::DEBUG_TYPE_UNDEFINED_BEHAVIOR => eprintln!("Type: Undefined Behaviour"),
                    gl::DEBUG_TYPE_PORTABILITY => eprintln!("Type: Portability"),
                    gl::DEBUG_TYPE_PERFORMANCE => eprintln!("Type: Performance"),
                    gl::DEBUG_TYPE_MARKER => eprintln!("Type: Marker"),
                    gl::DEBUG_TYPE_PUSH_GROUP => eprintln!("Type: Push Group"),
                    gl::DEBUG_TYPE_POP_GROUP => eprintln!("Type: Pop Group"),
                    gl::DEBUG_TYPE_OTHER => eprintln!("Type: Other"),
                    _ => (),
                }

                match severity {
                    gl::DEBUG_SEVERITY_HIGH => eprintln!("Severity: high"),
                    gl::DEBUG_SEVERITY_MEDIUM => eprintln!("Severity: medium"),
                    gl::DEBUG_SEVERITY_LOW => eprintln!("Severity: low"),
                    gl::DEBUG_SEVERITY_NOTIFICATION => eprintln!("Severity: notification"),
                    _ => (),
                }
                panic!();
            }
        }
        gl.DebugMessageCallback(Some(debug_callback), ptr::null());
    }
}

struct DxGlInterop {
    gl: Gl,
    wgl: Wgl,
    /// Interop device handle
    device: wgl::types::HANDLE,
    /// Staging texture
    staging: Option<direct3d11::Texture2D>,
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
    backbuffer: direct3d11::Texture2D,
    interop: Option<DxGlInterop>,
}

impl SwapChainResources {
    unsafe fn new(
        swap_chain: &dxgi::swap_chain::SwapChain1,
        _device: &direct3d11::Device,
        width: u32,
        height: u32,
    ) -> Result<SwapChainResources> {
        let buffer = swap_chain
            .buffer(0)
            .context("failed in IDXGISwapChain1::GetBuffer")?;
        Ok(SwapChainResources {
            backbuffer: buffer,
            interop: None,
        })
    }

    unsafe fn with_gl_interop(
        swap_chain: &dxgi::swap_chain::SwapChain1,
        device: &direct3d11::Device,
        gl: Gl,
        wgl: Wgl,
        width: u32,
        height: u32,
        use_staging_texture: bool,
    ) -> Result<SwapChainResources> {
        let mut res = Self::new(swap_chain, device, width, height)?;

        let interop_device = wgl.DXOpenDeviceNV(device.get_raw() as *mut _);
        if interop_device.is_null() {
            return Err(anyhow::Error::msg("could not create OpenGL-DX interop"));
        }

        let mut renderbuffer = 0;
        gl.GenRenderbuffers(1, &mut renderbuffer);

        let (staging, interop_target) = if use_staging_texture {
            // use staging texture because directly sharing the swap chain buffer when using FLIP_*
            // swap effects seems to cause problems.
            let staging = direct3d11::Texture2D::create(device)
                .with_format(Format::R8G8B8A8Unorm)
                .with_size(width, height)
                .with_bind_flags(BindFlags::RENDER_TARGET)
                .with_mip_levels(1)
                .with_usage(Usage::Default)
                .build()?;

            let interop_staging = wgl.DXRegisterObjectNV(
                interop_device,
                staging.get_raw() as *mut _,
                renderbuffer,
                gl::RENDERBUFFER,
                wgl::ACCESS_READ_WRITE_NV,
            );
            (Some(staging), interop_staging)
        } else {
            // directly share the swap chain buffer (this may cause problems with FLIP_* swap effects)
            let interop_backbuffer = wgl.DXRegisterObjectNV(
                interop_device,
                res.backbuffer.get_raw() as *mut _,
                renderbuffer,
                gl::RENDERBUFFER,
                wgl::ACCESS_READ_WRITE_NV,
            );
            (None, interop_backbuffer)
        };

        if interop_target.is_null() {
            gl.DeleteRenderbuffers(1, &renderbuffer);
            return Err(Error::msg("wglDXRegisterObjectNV error"));
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
            return Err(Error::msg(format!(
                "could not create window framebuffer: CheckNamedFramebufferStatus returned {}",
                fb_status
            )));
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
const USE_INTEROP_STAGING_TEXTURE: bool = false;

/// Guard object that holds a lock on an interop OpenGL context.
pub struct OpenGlDrawContext<'a> {
    gl: &'a Gl,
    wgl: &'a Wgl,
    interop: &'a mut DxGlInterop,
    backbuffer: &'a direct3d11::Texture2D,
    d3d11_ctx: &'a direct3d11::DeviceContext,
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
            let backbuffer = self.backbuffer.as_resource();
            let staging = staging_d3d11.as_resource();
            unsafe {
                self.d3d11_ctx.copy_resource(&staging, &backbuffer);
            }
        }
    }
}

pub struct Direct2dDrawContext<'a> {
    window: &'a mut PlatformWindow,
    target: direct2d::render_target::RenderTarget,
    _phantom: PhantomData<&'a ()>,
}

impl<'a> Direct2dDrawContext<'a> {
    pub fn new(window: &'a mut PlatformWindow) -> Direct2dDrawContext<'a> {
        let d2d = &window.shared.d2d_factory;
        let dwrite = &window.shared.dwrite_factory;
        //let mut context = self.shared.d2d_context.borrow_mut();
        let swap_res = window.swap_res.as_ref().unwrap();
        let dxgi_buffer = swap_res.backbuffer.as_dxgi();


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

        let target = unsafe {
            let mut render_target: *mut ID2D1RenderTarget = ptr::null_mut();
            let res = (*d2d.get_raw()).CreateDxgiSurfaceRenderTarget(
                dxgi_buffer.get_raw(),
                &props,
                &mut render_target,
            );
            direct2d::render_target::RenderTarget::from_raw(render_target)
        };

        Direct2dDrawContext {
            window,
            target,
            _phantom: PhantomData,
        }
    }

    pub fn window(&self) -> &PlatformWindow {
        self.window
    }

    pub fn render_target(&self) -> &direct2d::render_target::RenderTarget {
        &self.target
    }

    pub fn render_target_mut(&mut self) -> &mut direct2d::render_target::RenderTarget {
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
    swap_chain: dxgi::swap_chain::SwapChain1,
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
    pub fn resize(&mut self, (width, height): (u32, u32)) -> Result<()> {
        trace!("resizing swap chain: {}x{}", width, height);

        // signal the GL context as well if we have one
        if let Some(ref mut gl) = self.gl {
            gl.context.resize((width, height).into());
        }

        unsafe {
            // explicitly release swap-chain dependent resources
            self.swap_res = None;

            // resize the swap chain
            let err = self
                .swap_chain
                .resize_buffers()
                .dimensions(width, height)
                .finish();

            if let Err(err) = err {
                // it fails sometimes...
                error!("IDXGISwapChain1::ResizeBuffers failed: {}", err);
                return Ok(());
            }

            // re-create all resources that depend on the swap chain
            self.swap_res = Some(if let Some(ref mut gl) = self.gl {
                SwapChainResources::with_gl_interop(
                    &self.swap_chain,
                    &self.shared.d3d11_device,
                    gl.gl.clone(),
                    gl.wgl.clone(),
                    width,
                    height,
                    self.interop_needs_staging,
                )?
            } else {
                SwapChainResources::new(&self.swap_chain, &self.shared.d3d11_device, width, height)?
            });
        }

        Ok(())
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
        with_gl: bool) -> Result<PlatformWindow> {
        // We want to be able to render 3D stuff with OpenGL, and still be able to use
        // D3D11/Direct2D/DirectWrite.
        // To do so, we use a DXGI swap chain to manage presenting. Then, using WGL_NV_DX_interop2,
        // we register the buffers of the swap chain as a renderbuffer in GL so we can use both
        // on the same render target.
        unsafe {
            // first, build the window using the provided builder
            let window = builder.build(event_loop)?;

            let dxgi_factory = &platform.0.dxgi_factory;
            let d3d11_device = &platform.0.d3d11_device;

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
            let swap_effect = SwapEffect::Discard;

            // OpenGL interop does not work well with FLIP_* swap effects
            // (generates a "D3D11 Device Lost" error during resizing after a while).
            // In those cases, draw on a staging texture, and then copy to the backbuffer.
            let interop_needs_staging = match swap_effect {
                SwapEffect::FlipSequential | SwapEffect::FlipDiscard => true,
                _ => false,
            };

            if interop_needs_staging && with_gl {
                info!("FLIP_DISCARD or FLIP_SEQUENTIAL swap chains with OpenGL interop may cause crashes. \
                 Will allocate a staging target to work around this issue.");
            }

            // create the swap chain
            let swap_chain =
                dxgi::swap_chain::SwapChain1::create_hwnd(dxgi_factory, &d3d11_device.as_dxgi())
                    .with_flags(SwapChainFlags::NONE)
                    .with_swap_effect(swap_effect)
                    .with_format(Format::R8G8B8A8Unorm)
                    .with_buffer_count(SWAP_CHAIN_BUFFERS)
                    .with_scaling(Scaling::Stretch)
                    .with_alpha_mode(AlphaMode::Unspecified)
                    .with_buffer_usage(UsageFlags::RENDER_TARGET_OUTPUT)
                    .with_hwnd(hwnd)
                    .build()?;

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
                init_debug_callback(&gl);
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
        self.swap_chain
            .present(1, PresentFlags::NONE)
            .expect("present failed");
    }
}
