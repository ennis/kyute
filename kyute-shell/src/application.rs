//! Application.
use crate::backend;
use lazy_static::lazy_static;
use std::{
    sync::{Arc, Mutex, MutexGuard},
    time::Duration,
};

/// Mutex-protected and ref-counted alias to `graal::Context`.
pub type GpuContext = Arc<Mutex<graal::Context>>;

/// Encapsulates various platform-specific application services.
///
/// Contains a bunch of application-global objects and factories, mostly DirectX stuff for drawing
/// to the screen.
///
// all of this must be either directly Sync, or wrapped in a mutex, or wrapped in a main-thread-only wrapper.
pub struct Application {
    pub(crate) gpu_device: Arc<graal::Device>,
    pub(crate) gpu_context: Mutex<graal::Context>,
    pub(crate) backend: backend::Application,
}

lazy_static! {
    // NOTE: we previously used `OnceCell` so that we could get a `&'static Platform` that lived for
    // the duration of the application, but the destructor wasn't called. This has consequences on
    // windows because the DirectX debug layers trigger panics when objects are leaked.
    pub static ref APPLICATION: Application = Application::new().expect("failed to initialize application");
}

impl Application {
    /// Initializes the global application object.
    ///
    /// The application object will be tied to this thread (the "main thread").
    fn new() -> anyhow::Result<Application> {
        // --- Create the graal context (implying a vulkan instance and device)
        // FIXME technically we need the target surface so we can pick a device that can
        // render to it. However, on most systems, all available devices can render to window surfaces,
        // so skip that for now.
        let (gpu_device, gpu_context) = unsafe {
            // SAFETY: we don't pass a surface handle
            graal::create_device_and_context(None)
        };

        let app = Application {
            gpu_device,
            gpu_context: Mutex::new(gpu_context),
            backend: backend::Application::new(),
        };

        Ok(app)
    }

    /// Returns the global application object.
    pub fn instance() -> &'static Application {
        &*APPLICATION
    }

    /// Returns the system double click time in milliseconds.
    pub fn double_click_time(&self) -> Duration {
        self.backend.double_click_time()
    }

    /// Returns the `graal::Device` instance.
    pub fn gpu_device(&self) -> &Arc<graal::Device> {
        &self.gpu_device
    }

    /// Locks the GPU context.
    pub fn lock_gpu_context(&self) -> MutexGuard<graal::Context> {
        self.gpu_context.lock().unwrap()
    }

    /// Enters the main event loop.
    pub fn run(&self) {
        self.backend.run();
    }
}
