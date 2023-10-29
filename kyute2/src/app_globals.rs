use crate::{backend::AppBackend, composition::Compositor, skia_backend};
use std::{cell::RefCell, rc::Rc, time::Duration};

//==================================================================================================

/// Application globals.
///
/// Stuff that would be too complicated/impractical/ugly to carry and pass around as parameters.
pub struct AppGlobals {
    pub(crate) backend: AppBackend,
    pub(crate) drawing: skia_backend::DrawingBackend,
    pub compositor: Compositor,
}

thread_local! {
    static APP_GLOBALS: RefCell<Option<Rc<AppGlobals>>> = RefCell::new(None);
}

impl AppGlobals {
    /// Creates a new `Application` instance.
    pub fn new() -> Rc<AppGlobals> {
        // Create glazier Application.
        // This ensures that we're not calling `Application::new()` multiple times before `run`.
        //let _ = glazier::Application::new().expect("an application should not already be active");

        // TODO: make sure that we're not making multiple applications

        let backend = AppBackend::new();
        let drawing = skia_backend::DrawingBackend::new(&backend);
        let compositor = Compositor::new(&backend);
        let app = Rc::new(AppGlobals {
            drawing,
            backend,
            compositor,
        });

        APP_GLOBALS.with(|g| g.replace(Some(app.clone())));
        app
    }

    pub fn try_get() -> Option<Rc<AppGlobals>> {
        APP_GLOBALS.with(|g| Some(g.borrow().as_ref()?.clone()))
    }

    pub fn get() -> Rc<AppGlobals> {
        AppGlobals::try_get().expect("an application should be active on this thread")
    }

    pub fn double_click_time(&self) -> Duration {
        self.backend.double_click_time()
    }

    /// Returns the vulkan device instance.
    #[cfg(feature = "vulkan")]
    pub fn gpu_device(&self) -> Arc<graal::Device> {
        self.0.drawing.borrow().device.clone()
    }

    /*/// Runs the application.
    ///
    /// Defers to `glazier::Application::run()`.
    pub fn run(self, app_handler: Option<Box<dyn AppHandler>>) {
        glazier::Application::global().run(app_handler);
        // TODO: cleanup
        GLOBAL_APP.with(|g| g.replace(None));
    }*/
}
