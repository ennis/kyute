//! Windows-specific UI stuff.

use crate::error::Error;

use std::os::raw::c_void;
use std::ptr;
use std::rc::Rc;
use winapi::shared::dxgi::*;
use winapi::shared::dxgi1_3::*;
use winapi::shared::winerror::SUCCEEDED;
use winapi::um::d2d1::*;
use winapi::um::d2d1_1::*;
use winapi::um::d3d11::*;
use winapi::um::d3dcommon::*;
use winapi::um::dwrite::*;
use winapi::um::unknwnbase::IUnknown;
use winapi::um::winuser::GetDoubleClickTime;
use winapi::Interface;
use wio::com::ComPtr;
use std::time::Duration;

/// Contains a bunch of application-global objects and factories, mostly DirectX stuff for drawing
/// to the screen.
pub(crate) struct PlatformState {
    pub(crate) d3d11_device: ComPtr<ID3D11Device>,
    pub(crate) d3d11_device_context: ComPtr<ID3D11DeviceContext>,
    pub(crate) dxgi_factory: ComPtr<IDXGIFactory3>,
    pub(crate) d2d_factory: ComPtr<ID2D1Factory1>,
    pub(crate) dwrite_factory: ComPtr<IDWriteFactory>,
    pub(crate) d2d_device: ComPtr<ID2D1Device>,
}

/// Encapsulates the platform-specific application global state.
pub struct Platform(pub(crate) Rc<PlatformState>);

impl Platform {
    /// Initializes platform-specific application state.
    pub unsafe fn init() -> Platform {
        // any failure in this function prevents the application from running properly,
        // so just panic

        // ---------- create the D3D11 device and device context ----------
        let mut d3d11_device = ptr::null_mut();
        let mut feature_level = 0;
        let mut d3d11_device_context = ptr::null_mut();

        let hr = D3D11CreateDevice(
            // pAdapter:
            ptr::null_mut(),
            // DriverType:
            D3D_DRIVER_TYPE_HARDWARE,
            // Software:
            ptr::null_mut(),
            // Flags:
            D3D11_CREATE_DEVICE_BGRA_SUPPORT | D3D11_CREATE_DEVICE_DEBUG,
            // pFeatureLevels:
            ptr::null(),
            // FeatureLevels:
            0,
            // SDKVersion
            D3D11_SDK_VERSION,
            // ppDevice:
            &mut d3d11_device,
            // pFeatureLevel:
            &mut feature_level,
            // ppImmediateContext:
            &mut d3d11_device_context,
        );

        if !SUCCEEDED(hr) {
            panic!(
                "Could not create a D3D11 device: {}",
                Error::HResultError(hr)
            );
        }

        let d3d11_device = ComPtr::from_raw(d3d11_device);
        let d3d11_device_context = ComPtr::from_raw(d3d11_device_context);

        // ---------- DXGI Factory ----------
        let mut ptr: *mut IDXGIFactory3 = ptr::null_mut();
        let hr = CreateDXGIFactory2(
            0,
            &IDXGIFactory3::uuidof(),
            &mut ptr as *mut _ as *mut *mut c_void,
        );
        if !SUCCEEDED(hr) {
            panic!(
                "Could not create a DXGI factory: {}",
                Error::HResultError(hr)
            );
        }
        let dxgi_factory = ComPtr::from_raw(ptr);

        // ---------- Direct2D,DirectWrite factories ----------
        let mut dwrite_factory: *mut IDWriteFactory = ptr::null_mut();
        let hr = DWriteCreateFactory(
            DWRITE_FACTORY_TYPE_SHARED,
            &IDWriteFactory::uuidof(),
            &mut dwrite_factory as *mut _ as *mut *mut IUnknown,
        );
        if !SUCCEEDED(hr) {
            panic!(
                "Could not create a DirectWrite factory: {}",
                Error::HResultError(hr)
            );
        }
        let dwrite_factory = ComPtr::from_raw(dwrite_factory);

        let mut d2d_factory: *mut ID2D1Factory1 = ptr::null_mut();
        let hr = D2D1CreateFactory(
            D2D1_FACTORY_TYPE_MULTI_THREADED,
            &ID2D1Factory1::uuidof(),
            &D2D1_FACTORY_OPTIONS {
                debugLevel: D2D1_DEBUG_LEVEL_WARNING,
            },
            &mut d2d_factory as *mut _ as *mut *mut c_void,
        );
        if !SUCCEEDED(hr) {
            panic!(
                "Could not create a Direct2D factory: {}",
                Error::HResultError(hr)
            );
        }
        let d2d_factory = ComPtr::from_raw(d2d_factory);

        // ---------- Create the D2D Device and Context ----------
        let mut d2d_device = ptr::null_mut();
        let dxgi_device = d3d11_device.cast::<IDXGIDevice>().unwrap();
        let hr = d2d_factory.CreateDevice(dxgi_device.as_raw(), &mut d2d_device);
        if !SUCCEEDED(hr) {
            panic!(
                "Could not create a Direct2D device: {}",
                Error::HResultError(hr)
            );
        }
        let d2d_device = ComPtr::from_raw(d2d_device);

        Platform(Rc::new(PlatformState {
            d3d11_device,
            d3d11_device_context,
            dxgi_factory,
            dwrite_factory,
            d2d_factory,
            d2d_device,
        }))
    }

    /// Returns the system double click time in milliseconds.
    pub fn double_click_time(&self) -> Duration {
        unsafe {
            let ms = GetDoubleClickTime();
            Duration::from_millis(ms as u64)
        }
    }
}
