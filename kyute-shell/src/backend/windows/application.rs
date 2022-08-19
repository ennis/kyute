use crate::backend::windows::{event::Win32Event, util::ToWide, window};
use graal::vk::LPCWSTR;
use parking_lot::Mutex;
use std::{
    ffi::{c_void, OsString},
    mem, ptr,
    time::Duration,
};
use threadbound::ThreadBound;
use windows::{
    core::{Interface, PCWSTR},
    Win32::{
        Foundation::{HINSTANCE, HWND},
        Graphics::{
            Direct3D::D3D_FEATURE_LEVEL_12_0,
            Direct3D12::{
                D3D12CreateDevice, D3D12GetDebugInterface, ID3D12CommandQueue, ID3D12Debug, ID3D12Device, ID3D12Fence,
                D3D12_COMMAND_LIST_TYPE_DIRECT, D3D12_COMMAND_QUEUE_DESC, D3D12_FENCE_FLAG_NONE,
            },
            DirectComposition::{DCompositionCreateDevice3, IDCompositionDesktopDevice, IDCompositionDeviceDebug},
            DirectWrite::{DWriteCreateFactory, IDWriteFactory, DWRITE_FACTORY_TYPE_SHARED},
            Dxgi::{CreateDXGIFactory2, IDXGIFactory3, DXGI_CREATE_FACTORY_DEBUG},
            Gdi::HBRUSH,
            Imaging::{CLSID_WICImagingFactory2, D2D::IWICImagingFactory2},
        },
        System::{
            Com::{CoCreateInstance, CoInitialize, CLSCTX_INPROC_SERVER},
            Threading::{CreateEventW, WaitForSingleObject},
        },
        UI::{
            Input::{KeyboardAndMouse::GetDoubleClickTime, Pointer::EnableMouseInPointer},
            WindowsAndMessaging::{
                DispatchMessageW, GetMessageW, LoadIconW, PeekMessageW, RegisterClassW, TranslateMessage, CS_CLASSDC,
                HCURSOR, IDI_APPLICATION, MSG, PM_NOREMOVE, WM_TIMER, WNDCLASSW, WNDCLASS_STYLES, WNDPROC,
            },
        },
    },
};

////////////////////////////////////////////////////////////////////////////////////////////////////
// COM wrappers
////////////////////////////////////////////////////////////////////////////////////////////////////

// COM thread safety notes: some interfaces are thread-safe, some are not, and for some we don't know due to poor documentation.
// Additionally, some interfaces should only be called on the thread in which they were created.
//
// - For thread-safe interfaces: wrap them in a `Send+Sync` newtype
// - For interfaces bound to a thread: wrap them in `ThreadBound`
// - For interfaces not bound to a thread but with unsynchronized method calls:
//      wrap them in a `Send` newtype, and if you actually need to call the methods from multiple threads, `Mutex`.

/// Defines a send+sync wrapper over a windows interface type.
///
/// This signifies that it's OK to call the interface's methods from multiple threads simultaneously:
/// the object itself should synchronize the calls.
macro_rules! sync_com_ptr_wrapper {
    ($wrapper:ident ( $iface:ident ) ) => {
        #[derive(Clone)]
        pub(crate) struct $wrapper(pub(crate) $iface);
        unsafe impl Sync for $wrapper {} // ok to send &I across threads
        unsafe impl Send for $wrapper {} // ok to send I across threads
        impl ::std::ops::Deref for $wrapper {
            type Target = $iface;
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }
    };
}

/// Defines a send wrapper over a windows interface type.
///
/// This signifies that it's OK to call an interface's methods from a different thread than that in which it was created.
/// However, you still have to synchronize the method calls yourself (with, e.g., a `Mutex`).  
macro_rules! send_com_ptr_wrapper {
    ($wrapper:ident ( $iface:ident ) ) => {
        #[derive(Clone)]
        pub(crate) struct $wrapper(pub(crate) $iface);
        unsafe impl Send for $wrapper {} // ok to send I across threads
        impl ::std::ops::Deref for $wrapper {
            type Target = $iface;
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }
    };
}

sync_com_ptr_wrapper! { D3D12Device(ID3D12Device) }
sync_com_ptr_wrapper! { DXGIFactory3(IDXGIFactory3) }
sync_com_ptr_wrapper! { D3D12CommandQueue(ID3D12CommandQueue) }
sync_com_ptr_wrapper! { DWriteFactory(IDWriteFactory) }
sync_com_ptr_wrapper! { WICImagingFactory2(IWICImagingFactory2) }
sync_com_ptr_wrapper! { D3D12Fence(ID3D12Fence) }

// D3D11/D2D not used anymore
//sync_com_ptr_wrapper! { D3D11Device(ID3D11Device5) }
//sync_com_ptr_wrapper! { D2D1Factory1(ID2D1Factory1) }
//sync_com_ptr_wrapper! { D2D1Device(ID2D1Device) }
//send_com_ptr_wrapper! { D2D1DeviceContext(ID2D1DeviceContext) }

pub(crate) const CLASS_NAME: &str = "kyuuuuute";

////////////////////////////////////////////////////////////////////////////////////////////////////
// Application (win32 backend)
////////////////////////////////////////////////////////////////////////////////////////////////////

pub(crate) struct Application {
    pub(crate) d3d12_device: D3D12Device,              // thread safe
    pub(crate) d3d12_command_queue: D3D12CommandQueue, // thread safe
    pub(crate) command_completion_fence: D3D12Fence,
    pub(crate) command_completion_event: Win32Event,
    pub(crate) command_completion_fence_value: Mutex<u64>,
    pub(crate) dxgi_factory: DXGIFactory3,
    pub(crate) dwrite_factory: DWriteFactory,
    //pub(crate) wic_factory: WICImagingFactory2,
    pub(crate) composition_device: ThreadBound<IDCompositionDesktopDevice>,
}

impl Application {
    pub(crate) fn new() -> Application {
        let _d3d12_debug = {
            // D3D12 debug interface
            let mut dbg: Option<ID3D12Debug> = None;
            unsafe {
                D3D12GetDebugInterface(&mut dbg).expect("D3D12GetDebugInterface failed");
                dbg.unwrap()
            }
        };

        // ---------- DXGI Factory ----------

        // SAFETY: the paramters are valid
        let dxgi_factory =
            unsafe { DXGIFactory3(CreateDXGIFactory2::<IDXGIFactory3>(DXGI_CREATE_FACTORY_DEBUG).unwrap()) };

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

        let d3d12_device = unsafe {
            let mut d3d12_device: Option<ID3D12Device> = None;
            D3D12CreateDevice(
                // pAdapter:
                None,
                // MinimumFeatureLevel:
                D3D_FEATURE_LEVEL_12_0,
                // ppDevice:
                &mut d3d12_device,
            )
            .expect("D3D12CreateDevice failed");
            D3D12Device(d3d12_device.unwrap())
        };

        let d3d12_command_queue = unsafe {
            let cqdesc = D3D12_COMMAND_QUEUE_DESC {
                Type: D3D12_COMMAND_LIST_TYPE_DIRECT,
                Priority: 0,
                Flags: Default::default(),
                NodeMask: 0,
            };
            let cq: ID3D12CommandQueue = d3d12_device
                .0
                .CreateCommandQueue(&cqdesc)
                .expect("CreateCommandQueue failed");
            D3D12CommandQueue(cq)
        };

        let command_completion_fence = unsafe {
            let fence = d3d12_device
                .0
                .CreateFence::<ID3D12Fence>(0, D3D12_FENCE_FLAG_NONE)
                .expect("CreateFence failed");
            D3D12Fence(fence)
        };

        // ---------- Direct2D,DirectWrite factories ----------
        let dwrite_factory = unsafe {
            let dwrite = DWriteCreateFactory(DWRITE_FACTORY_TYPE_SHARED, &IDWriteFactory::IID)
                .unwrap()
                .cast::<IDWriteFactory>()
                .unwrap();
            DWriteFactory(dwrite)
        };

        // --------- Compositor -----------
        let composition_device = unsafe {
            let mut composition_device: Option<IDCompositionDesktopDevice> = None;
            DCompositionCreateDevice3(
                None,
                &IDCompositionDesktopDevice::IID,
                &mut composition_device as *mut _ as *mut *mut c_void,
            )
            .expect("DCompositionCreateDevice failed");
            // enable composition device debug
            let composition_device = composition_device.unwrap();
            let _debug: IDCompositionDeviceDebug = composition_device.cast::<IDCompositionDeviceDebug>().unwrap();
            //debug.EnableDebugCounters();
            ThreadBound::new(composition_device)
        };

        let command_completion_event = unsafe {
            let event = CreateEventW(ptr::null(), false, false, None).unwrap();
            Win32Event::from_raw(event)
        };

        // --------- Window class -----------
        //
        let class_name = CLASS_NAME.to_wide();
        let icon = unsafe { LoadIconW(HINSTANCE::default(), IDI_APPLICATION).unwrap() };
        let wndclass = WNDCLASSW {
            style: WNDCLASS_STYLES::default(),
            lpfnWndProc: Some(window::win_proc_dispatch),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: HINSTANCE::default(),
            hIcon: icon,
            hCursor: HCURSOR::default(),
            hbrBackground: HBRUSH::default(),
            lpszMenuName: PCWSTR::null(),
            lpszClassName: PCWSTR(class_name.as_ptr()),
        };
        let class_atom = unsafe { RegisterClassW(&wndclass) };
        if class_atom == 0 {
            panic!("Error registering class");
        }

        unsafe {
            EnableMouseInPointer(true);
        }

        Application {
            d3d12_device,
            d3d12_command_queue,
            command_completion_event,
            command_completion_fence,
            command_completion_fence_value: Mutex::new(0),
            dxgi_factory,
            dwrite_factory,
            //wic_factory,
            composition_device,
        }
    }

    pub(crate) fn wait_for_command_completion(&self) {
        unsafe {
            let mut fence_value = self.command_completion_fence_value.lock();
            *fence_value += 1;
            self.d3d12_command_queue
                .0
                .Signal(&self.command_completion_fence.0, *fence_value)
                .expect("ID3D12CommandQueue::Signal failed");
            if self.command_completion_fence.0.GetCompletedValue() < *fence_value {
                self.command_completion_fence
                    .0
                    .SetEventOnCompletion(*fence_value, self.command_completion_event.handle())
                    .expect("SetEventOnCompletion failed");
                WaitForSingleObject(self.command_completion_event.handle(), 0xFFFFFFFF);
            }
        }
    }

    /// Returns the system double click time in milliseconds.
    pub(crate) fn double_click_time(&self) -> Duration {
        unsafe {
            let ms = GetDoubleClickTime();
            Duration::from_millis(ms as u64)
        }
    }

    /// Enters the main event loop.
    pub fn run(&self) {
        // Code adapted from the druid-shell crate.
        // See license info in the repository root.
        //
        // https://github.com/linebender/druid/blob/ffcbc49920f051bf6757c8bfaf71e4b934e02ecd/druid-shell/src/backend/windows/application.rs#L122
        unsafe {
            // Handle windows messages.
            //
            // NOTE: Code here will not run when we aren't in charge of the message loop. That
            // will include when moving or resizing the window, and when showing modal dialogs.
            loop {
                let mut msg = mem::MaybeUninit::uninit();

                // Timer messages have a low priority and tend to get delayed. Peeking for them
                // helps for some reason; see
                // https://devblogs.microsoft.com/oldnewthing/20191108-00/?p=103080
                PeekMessageW(msg.as_mut_ptr(), HWND::default(), WM_TIMER, WM_TIMER, PM_NOREMOVE);

                let res = GetMessageW(msg.as_mut_ptr(), HWND::default(), 0, 0).0;
                if res <= 0 {
                    if res == -1 {
                        let last_err = windows::core::Error::from_win32();
                        error!("GetMessageW failed: {}", last_err);
                    }
                    break;
                }
                let mut msg: MSG = msg.assume_init();
                //let accels = accels::find_accels(GetAncestor(msg.hwnd, GA_ROOT));
                //let translated = accels.map_or(false, |it| TranslateAcceleratorW(msg.hwnd, it.handle(), &mut msg) != 0);
                //if !translated {
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
                //}
            }
        }
    }
}
