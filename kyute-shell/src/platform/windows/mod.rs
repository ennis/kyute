//! Windows-specific UI stuff.
use std::cell::RefCell;

mod window;
mod gl_api;

pub use window::Direct2dDrawContext;
pub use window::OpenGlDrawContext;
pub use window::PlatformWindow;

use anyhow::Result;
use std::rc::Rc;

/// Contains a bunch of application-global objects and factories, mostly DirectX stuff for drawing
/// to the screen.
struct PlatformState {
    d3d11_device: direct3d11::Device,
    d3d11_device_context: direct3d11::DeviceContext,
    dxgi_factory: dxgi::factory::Factory2,
    dwrite_factory: directwrite::Factory,
    d2d_factory: direct2d::factory::Factory1,
    d2d_device: direct2d::device::Device,
    d2d_context: RefCell<direct2d::device_context::DeviceContext>,
}

/// Encapsulates the platform-specific application global state.
pub struct Platform(Rc<PlatformState>);

impl Platform {
    /// Initializes platform-specific application state.
    pub unsafe fn init() -> Result<Platform> {
        use direct3d11::enums::*;

        let (_feature_level, d3d11_device, d3d11_device_context) =
            direct3d11::Device::create()
                .with_flags(CreateDeviceFlags::BGRA_SUPPORT | CreateDeviceFlags::DEBUG)
                .with_driver_type(DriverType::Hardware)
                .build()?;
        let dxgi_factory: dxgi::factory::Factory2 = dxgi::factory::create()?;
        let dwrite_factory = directwrite::Factory::new()?;
        let d2d_factory = direct2d::factory::Factory1::new()?;

        // Create the D2D Device and Context
        let d2d_device =
            direct2d::device::Device::create(&d2d_factory, &d3d11_device.as_dxgi())?;
        let d2d_context = RefCell::new(direct2d::device_context::DeviceContext::create(
            &d2d_device,
        )?);

        Ok(Platform(Rc::new(PlatformState {
            d3d11_device,
            d3d11_device_context,
            dxgi_factory,
            dwrite_factory,
            d2d_factory,
            d2d_device,
            d2d_context,
        })))
    }

    /// Returns the global DirectWrite factory.
    pub fn directwrite(&self) -> &directwrite::Factory {
        &self.0.dwrite_factory
    }

    /// Returns the global Direct2D factory.
    pub fn direct2d(&self) -> &direct2d::factory::Factory1 {
        &self.0.d2d_factory
    }
}
