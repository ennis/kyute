//! Windows-specific UI stuff.
use std::{
    cell::RefCell,
    ffi::OsString,
    ops::Deref,
    ptr,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex, MutexGuard,
    },
    time::Duration,
};
use windows::{
    core::Interface,
    Win32::{
        Graphics::{
            Direct2D::{
                D2D1CreateFactory, ID2D1Device, ID2D1DeviceContext, ID2D1Factory1, D2D1_DEBUG_LEVEL_WARNING,
                D2D1_DEVICE_CONTEXT_OPTIONS_NONE, D2D1_FACTORY_OPTIONS, D2D1_FACTORY_TYPE_MULTI_THREADED,
            },
            Direct3D::{D3D_DRIVER_TYPE_HARDWARE, D3D_FEATURE_LEVEL, D3D_FEATURE_LEVEL_11_1},
            Direct3D11::{
                D3D11CreateDevice, ID3D11Device5, D3D11_CREATE_DEVICE_BGRA_SUPPORT, D3D11_CREATE_DEVICE_DEBUG,
                D3D11_SDK_VERSION,
            },
            DirectWrite::{DWriteCreateFactory, IDWriteFactory, DWRITE_FACTORY_TYPE_SHARED},
            Dxgi::{CreateDXGIFactory2, IDXGIDevice, IDXGIFactory3},
            Imaging::{CLSID_WICImagingFactory2, D2D::IWICImagingFactory2},
        },
        System::Com::{CoCreateInstance, CoInitialize, CLSCTX_INPROC_SERVER},
        UI::Input::KeyboardAndMouse::GetDoubleClickTime,
    },
};

/// Mutex-protected and ref-counted alias to `graal::Context`.
pub type GpuContext = Arc<Mutex<graal::Context>>;

macro_rules! sync_com_ptr_wrapper {
    ($wrapper:ident ( $iface:ident ) ) => {
        #[derive(Clone)]
        pub(crate) struct $wrapper(pub(crate) $iface);
        unsafe impl Sync for $wrapper {} // ok to send &I across threads
        unsafe impl Send for $wrapper {} // ok to send I across threads
        impl Deref for $wrapper {
            type Target = $iface;
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }
    };
}

macro_rules! send_com_ptr_wrapper {
    ($wrapper:ident ( $iface:ident ) ) => {
        #[derive(Clone)]
        pub(crate) struct $wrapper(pub(crate) $iface);
        unsafe impl Send for $wrapper {} // ok to send I across threads
        impl Deref for $wrapper {
            type Target = $iface;
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }
    };
}

// Thread safety notes: some services are thread-safe, some are not, and for some we don't know due to poor documentation.
// Additionally, some services should only be used on the "main" thread or the "UI" thread.
// There are different ways to ensure thread safety:
// 1. Mutex-wrap all services for which we have no information about thread-safety
// 2. Restrict access to services to the main thread
//
// Option 2 may seem harsh. Consider an application that layouts the GUI in parallel: it might want to access
// the text services (to measure a text string) simultaneously across several threads.
//
// FIXME this might be a bit too optimistic...
sync_com_ptr_wrapper! { D3D11Device(ID3D11Device5) }
sync_com_ptr_wrapper! { DXGIFactory3(IDXGIFactory3) }
sync_com_ptr_wrapper! { D2D1Factory1(ID2D1Factory1) }
sync_com_ptr_wrapper! { DWriteFactory(IDWriteFactory) }
sync_com_ptr_wrapper! { D2D1Device(ID2D1Device) }
sync_com_ptr_wrapper! { WICImagingFactory2(IWICImagingFactory2) }
send_com_ptr_wrapper! { D2D1DeviceContext(ID2D1DeviceContext) }

/// Encapsulates various platform-specific application services.
///
/// Contains a bunch of application-global objects and factories, mostly DirectX stuff for drawing
/// to the screen.
///
// all of this must be either directly Sync, or wrapped in a mutex, or wrapped in a main-thread-only wrapper.
pub(crate) struct ApplicationImpl {
    pub(crate) gpu_device: Arc<graal::Device>,
    pub(crate) gpu_context: Mutex<graal::Context>,
    pub(crate) d3d11_device: D3D11Device, // thread safe
    //pub(crate) d3d12_device: D3D12Device,  // thread safe
    //pub(crate) d3d11_device_context: Mutex<ComPtr<ID3D11DeviceContext>>,   // not thread safe (should be thread-local)
    pub(crate) dxgi_factory: DXGIFactory3,
    pub(crate) d2d_factory: D2D1Factory1,
    pub(crate) dwrite_factory: DWriteFactory,
    pub(crate) d2d_device: D2D1Device,
    // FIXME: it's far too easy to clone the ID2D11DeviceContext accidentally and use it in a thread-unsafe way: maybe create it on-the-fly instead?
    pub(crate) d2d_device_context: D2D1DeviceContext,
    pub(crate) wic_factory: WICImagingFactory2,
}

/// Encapsulates application-global services.
#[derive(Clone)]
pub struct Application(pub(crate) Arc<ApplicationImpl>);

thread_local! {
    /// Platform singleton. Only accessible from the main thread, hence the `thread_local`.
    /// Contains `None` if

    // NOTE: we previously used `OnceCell` so that we could get a `&'static Platform` that lived for
    // the duration of the application, but the destructor wasn't called. This has consequences on
    // windows because the DirectX debug layers trigger panics when objects are leaked.
    // Now we use shared ownership instead, and automatically release this global reference when
    // `run` returns.
    static APPLICATION: RefCell<Option<Application>> = RefCell::new(None);
}

/// Global flag that tells whether there's an active `Application` object in `APPLICATION`.
static APPLICATION_CREATED: AtomicBool = AtomicBool::new(false);

impl Application {
    /// Initializes the global application object.
    ///
    /// The application object will be tied to this thread (the "main thread").
    pub fn new() -> anyhow::Result<Application> {
        // check that we don't already have an active platform, and acquire the global flag
        APPLICATION_CREATED
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .map_err(|_| anyhow::anyhow!("an application has already been created."))?;

        // actually create the platform
        let application = Self::new_impl();

        let application = match application {
            Err(e) => {
                // if creation failed, don't forget to release the global flag
                APPLICATION_CREATED.store(false, Ordering::Release);
                return Err(e.context("failed to create application"));
            }
            Ok(p) => p,
        };

        APPLICATION.with(|app| app.replace(Some(application.clone())));
        Ok(application)
    }

    fn new_impl() -> anyhow::Result<Application> {
        // --- Create the graal context (implying a vulkan instance and device)
        // FIXME technically we need the target surface so we can pick a device that can
        // render to it. However, on most systems, all available devices can render to window surfaces,
        // so skip that for now.
        let (gpu_device, gpu_context) = unsafe {
            // SAFETY: we don't pass a surface handle
            graal::create_device_and_context(None)
        };

        // FIXME technically we need the target surface so we can pick a device that can
        // render to it. However, on most systems, all available devices can render to window surfaces,
        // so skip that for now.
        //let gpu_context = graal::Context::new();

        // ---------- DXGI Factory ----------

        // SAFETY: the paramters are valid
        let dxgi_factory = unsafe { DXGIFactory3(CreateDXGIFactory2::<IDXGIFactory3>(0).unwrap()) };

        // --- Enumerate adapters
        let mut adapters = Vec::new();
        unsafe {
            let mut i = 0;
            while let Ok(adapter) = dxgi_factory.EnumAdapters1(i) {
                adapters.push(adapter);
                i += 1;
            }
        };

        for adapter in adapters.iter() {
            let desc = unsafe { adapter.GetDesc1().unwrap() };

            use std::os::windows::ffi::OsStringExt;

            let name = &desc.Description[..];
            let name_len = name.iter().take_while(|&&c| c != 0).count();
            let name = OsString::from_wide(&desc.Description[..name_len])
                .to_string_lossy()
                .into_owned();
            tracing::info!(
                "DXGI adapter: name={}, LUID={:08x}{:08x}",
                name,
                desc.AdapterLuid.HighPart,
                desc.AdapterLuid.LowPart,
            );
        }

        // --- Create the D3D11 device and device context

        // This is needed for D2D stuff.

        // SAFETY: the parameters are valid
        let (d3d11_device, d3d11_device_context) = unsafe {
            let mut d3d11_device = None;
            let mut feature_level = D3D_FEATURE_LEVEL::default();
            let mut _d3d11_device_context = None;

            let feature_levels = [D3D_FEATURE_LEVEL_11_1];

            D3D11CreateDevice(
                // pAdapter:
                None,
                // DriverType:
                D3D_DRIVER_TYPE_HARDWARE,
                // Software:
                None,
                // Flags:
                D3D11_CREATE_DEVICE_BGRA_SUPPORT | D3D11_CREATE_DEVICE_DEBUG,
                // pFeatureLevels:
                feature_levels.as_ptr(),
                // FeatureLevels:
                1,
                // SDKVersion
                D3D11_SDK_VERSION,
                // ppDevice:
                &mut d3d11_device,
                // pFeatureLevel:
                &mut feature_level,
                // ppImmediateContext:
                &mut _d3d11_device_context,
            )?;

            tracing::info!("Direct3D feature level: {:?}", feature_level);

            (
                D3D11Device(d3d11_device.unwrap().cast::<ID3D11Device5>().unwrap()),
                _d3d11_device_context.unwrap(),
            )
        };

        /*let d3d12_device = unsafe {
            let mut d3d12_device = ptr::null_mut();
            let mut feature_level = 0;
            check_hr(D3D12CreateDevice(
                // pAdapter:
                ptr::null_mut(),
                // MinimumFeatureLevel:
                D3D_FEATURE_LEVEL_11_1,
                // riid:
                &ID3D12Device::uuidof(),
                // ppDevice:
                &mut d3d12_device as *mut _ as *mut *mut c_void,
                ))
                .expect("D3D12CreateDevice failed");

            D3D12Device(ComPtr::from_raw(d3d12_device))
        };*/

        // SAFETY: pointers should be non-null if D3D11CreateDevice succeeds

        // ---------- Direct2D,DirectWrite factories ----------
        let dwrite_factory = unsafe {
            let dwrite = DWriteCreateFactory(DWRITE_FACTORY_TYPE_SHARED, &IDWriteFactory::IID)
                .unwrap()
                .cast::<IDWriteFactory>()
                .unwrap();
            DWriteFactory(dwrite)
        };

        let d2d_factory = unsafe {
            let mut result: Option<ID2D1Factory1> = None;
            let d2d = D2D1CreateFactory(
                D2D1_FACTORY_TYPE_MULTI_THREADED,
                &ID2D1Factory1::IID,
                &D2D1_FACTORY_OPTIONS {
                    debugLevel: D2D1_DEBUG_LEVEL_WARNING,
                },
                &mut result as *mut _ as *mut *mut _,
            )
            .map(|()| result.unwrap())?;
            D2D1Factory1(d2d)
        };

        // ---------- Create the D2D Device and Context ----------
        let d2d_device = unsafe {
            let dxgi_device = d3d11_device.cast::<IDXGIDevice>().unwrap();
            let device = d2d_factory.CreateDevice(&dxgi_device).unwrap();
            D2D1Device(device)
        };

        let d2d_device_context = unsafe {
            D2D1DeviceContext(
                d2d_device
                    .0
                    .CreateDeviceContext(D2D1_DEVICE_CONTEXT_OPTIONS_NONE)
                    .unwrap(),
            )
        };

        // ---------- Create the Windows Imaging Component (WIC) factory ----------
        let wic_factory = unsafe {
            CoInitialize(ptr::null_mut()).unwrap();
            let wic: IWICImagingFactory2 = CoCreateInstance(&CLSID_WICImagingFactory2, None, CLSCTX_INPROC_SERVER)
                .expect("CoCreateInstance(CLSID_WICImagingFactory2) failed");
            WICImagingFactory2(wic)
        };

        let app_impl = ApplicationImpl {
            gpu_device,
            gpu_context: Mutex::new(gpu_context),
            //d3d12_device,
            d3d11_device,
            dxgi_factory,
            dwrite_factory,
            d2d_factory,
            d2d_device,
            d2d_device_context,
            wic_factory,
        };

        let app = Application(Arc::new(app_impl));
        Ok(app)
    }

    /// Returns the global application object that was created by a call to `init`.
    ///
    /// # Panics
    ///
    /// Panics of no platform is active, or if called outside of the main thread, which is the thread
    /// that called `Platform::new`.
    pub fn instance() -> Application {
        APPLICATION
            .with(|p| p.borrow().clone())
            .expect("either the platform instance was not initialized, or not calling from the main thread")
    }

    /// Deletes the application object and closes the associated services.
    pub fn shutdown() {
        APPLICATION.with(|p| p.replace(None));
    }

    // issue: this returns different objects before and after `run` is called.
    // bigger issue: an `&EventLoopWindowTarget` can only be retrieved from the event loop callback,
    // once the main event loop object is consumed in `run`.
    // This means that we must pass around the event loop stuff Fuck this shit already.
    //pub fn event_loop(&self) -> &EventLoopWindowTarget<()> {
    //    &self.0.event_loop
    //}

    /// Returns the system double click time in milliseconds.
    pub fn double_click_time(&self) -> Duration {
        unsafe {
            let ms = GetDoubleClickTime();
            Duration::from_millis(ms as u64)
        }
    }

    /// Returns the `graal::Device` instance.
    pub fn gpu_device(&self) -> &Arc<graal::Device> {
        &self.0.gpu_device
    }

    /// Locks the GPU context.
    pub fn lock_gpu_context(&self) -> MutexGuard<graal::Context> {
        self.0.gpu_context.lock().unwrap()
    }

    /* pub fn run() {
        PLATFORM.with(|p| {
            p.borrow().unwrap().0.event_loop.run()
        })
    }*/
}
