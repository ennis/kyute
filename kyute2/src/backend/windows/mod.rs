//! Windows implementation details
pub mod composition;
mod event;

////////////////////////////////////////////////////////////////////////////////////////////////////

use std::{ffi::OsString, mem, time::Duration};
use threadbound::ThreadBound;
use windows::{
    core::{ComInterface, IUnknown},
    System::DispatcherQueueController,
    Win32::{
        Graphics::{
            Direct3D::D3D_FEATURE_LEVEL_12_0,
            Direct3D12::{
                D3D12CreateDevice, ID3D12CommandAllocator, ID3D12CommandQueue, ID3D12Device, ID3D12Fence,
                D3D12_COMMAND_LIST_TYPE_DIRECT, D3D12_COMMAND_QUEUE_DESC,
            },
            DirectWrite::{DWriteCreateFactory, IDWriteFactory, DWRITE_FACTORY_TYPE_SHARED},
            Dxgi::{CreateDXGIFactory2, IDXGIAdapter1, IDXGIFactory3, DXGI_ADAPTER_DESC1},
        },
        System::{
            Com::{CoInitializeEx, COINIT_APARTMENTTHREADED},
            WinRT::{CreateDispatcherQueueController, DispatcherQueueOptions, DQTAT_COM_NONE, DQTYPE_THREAD_CURRENT},
        },
        UI::Input::KeyboardAndMouse::GetDoubleClickTime,
    },
};

/////////////////////////////////////////////////////////////////////////////
// COM wrappers
/////////////////////////////////////////////////////////////////////////////

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

/*/// Defines a send wrapper over a windows interface type.
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
}*/

sync_com_ptr_wrapper! { D3D12Device(ID3D12Device) }
sync_com_ptr_wrapper! { DXGIFactory3(IDXGIFactory3) }
sync_com_ptr_wrapper! { D3D12CommandQueue(ID3D12CommandQueue) }
sync_com_ptr_wrapper! { DWriteFactory(IDWriteFactory) }
sync_com_ptr_wrapper! { D3D12Fence(ID3D12Fence) }
//sync_com_ptr_wrapper! { D3D11Device(ID3D11Device5) }
//sync_com_ptr_wrapper! { WICImagingFactory2(IWICImagingFactory2) }
//sync_com_ptr_wrapper! { D2D1Factory1(ID2D1Factory1) }
//sync_com_ptr_wrapper! { D2D1Device(ID2D1Device) }
//send_com_ptr_wrapper! { D2D1DeviceContext(ID2D1DeviceContext) }

/////////////////////////////////////////////////////////////////////////////
// AppBackend
/////////////////////////////////////////////////////////////////////////////

pub struct AppBackend {
    pub(crate) dispatcher_queue_controller: DispatcherQueueController,
    pub(crate) adapter: Option<IDXGIAdapter1>,
    pub(crate) d3d12_device: D3D12Device,              // thread safe
    pub(crate) d3d12_command_queue: D3D12CommandQueue, // thread safe
    pub(crate) d3d12_command_allocator: ThreadBound<ID3D12CommandAllocator>,
    pub(crate) dxgi_factory: DXGIFactory3,
    pub(crate) dwrite_factory: DWriteFactory,
    //pub(crate) d3d11_device: D3D11Device,
    //pub(crate) d3d11_device_context: ID3D11DeviceContext,
    //pub(crate) d2d_factory: D2D1Factory1,
    // pub(crate) d2d_device: D2D1Device,
    // FIXME: it's far too easy to clone the ID2D11DeviceContext accidentally and use it in a thread-unsafe way: maybe create it on-the-fly instead?
    //pub(crate) d2d_device_context: D2D1DeviceContext,
    //pub(crate) wic_factory: WICImagingFactory2,
}

impl AppBackend {
    pub(crate) fn new() -> AppBackend {
        unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED).unwrap() };

        // Dispatcher queue
        // SAFETY: FFI
        let dispatcher_queue_controller = unsafe {
            CreateDispatcherQueueController(DispatcherQueueOptions {
                dwSize: mem::size_of::<DispatcherQueueOptions>() as u32,
                threadType: DQTYPE_THREAD_CURRENT,
                apartmentType: DQTAT_COM_NONE,
            })
            .expect("failed to create dispatcher queue controller")
        };

        // DirectWrite factory
        let dwrite_factory = unsafe {
            let dwrite: IDWriteFactory = DWriteCreateFactory(DWRITE_FACTORY_TYPE_SHARED).unwrap();
            DWriteFactory(dwrite)
        };

        //=========================================================
        // DXGI Factory and adapter enumeration

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

        let mut chosen_adapter = None;
        for adapter in adapters.iter() {
            let mut desc = DXGI_ADAPTER_DESC1::default();
            unsafe { adapter.GetDesc1(&mut desc).unwrap() };

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
            /*if (desc.Flags & DXGI_ADAPTER_FLAG_SOFTWARE.0) != 0 {
                continue;
            }*/
            if chosen_adapter.is_none() {
                chosen_adapter = Some(adapter.clone())
            }
        }

        //=========================================================
        // D3D12 stuff

        let d3d12_device = unsafe {
            let mut d3d12_device: Option<ID3D12Device> = None;
            let adapter = chosen_adapter
                .as_ref()
                .map(|adapter| adapter.cast::<IUnknown>().unwrap());
            D3D12CreateDevice(
                // pAdapter:
                adapter.as_ref(),
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

        let d3d12_command_allocator = unsafe {
            let command_allocator = d3d12_device
                .0
                .CreateCommandAllocator(D3D12_COMMAND_LIST_TYPE_DIRECT)
                .unwrap();
            ThreadBound::new(command_allocator)
        };

        AppBackend {
            d3d12_device,
            d3d12_command_queue,
            d3d12_command_allocator,
            dxgi_factory,
            dwrite_factory,
            dispatcher_queue_controller,
            adapter: chosen_adapter,
        }
    }

    /// Returns the system double click time in milliseconds.
    pub(crate) fn double_click_time(&self) -> Duration {
        unsafe {
            let ms = GetDoubleClickTime();
            Duration::from_millis(ms as u64)
        }
    }
}
