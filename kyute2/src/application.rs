use crate::{backend::AppBackend, composition::Compositor, skia};
use glazier::AppHandler;
use std::{
    cell::{RefCell, RefMut},
    rc::Rc,
    sync::Arc,
};

//==================================================================================================

pub(crate) struct AppInner {
    backend: AppBackend,
    drawing: RefCell<skia::DrawingBackend>,
    compositor: RefCell<Compositor>,
}

/// Application globals.
///
/// Stuff that would be too complicated/impractical/ugly to carry and pass around as parameters.
///
/// TODO rename to `AppGlobals`, don't try to cover it
#[derive(Clone)]
pub struct Application(Rc<AppInner>);

thread_local! {
    static GLOBAL_APP: RefCell<Option<Application>> = RefCell::new(None);
}

impl Application {
    /// Creates a new `Application` instance.
    pub fn new() -> Application {
        // Create glazier Application.
        // This ensures that we're not calling `Application::new()` multiple times before `run`.
        let _ = glazier::Application::new().expect("an application should not already be active");

        let backend = AppBackend::new();
        let drawing = skia::DrawingBackend::new(&backend);
        let compositor = Compositor::new(&backend);
        let app = Application(Rc::new(AppInner {
            drawing: RefCell::new(drawing),
            backend,
            compositor: RefCell::new(compositor),
        }));

        GLOBAL_APP.with(|g| g.replace(Some(app.clone())));
        app
    }

    pub fn try_global() -> Option<Application> {
        GLOBAL_APP.with(|g| Some(g.borrow().as_ref()?.clone()))
    }

    pub fn global() -> Application {
        Application::try_global().expect("an application should be active on this thread")
    }

    /// Returns the vulkan device instance.
    #[cfg(feature = "vulkan")]
    pub fn gpu_device(&self) -> Arc<graal::Device> {
        self.0.drawing.borrow().device.clone()
    }

    /// Returns a mutable reference to the compositor.
    pub fn compositor(&self) -> RefMut<Compositor> {
        self.0.compositor.borrow_mut()
    }

    /// Runs the application.
    ///
    /// Defers to `glazier::Application::run()`.
    pub fn run(self, app_handler: Option<Box<dyn AppHandler>>) {
        glazier::Application::global().run(app_handler);
        // TODO: cleanup
        GLOBAL_APP.with(|g| g.replace(None));
    }

    /// Returns the skia drawing backend.
    pub(crate) fn drawing(&self) -> RefMut<skia::DrawingBackend> {
        self.0.drawing.borrow_mut()
    }

    /// Returns the app backend.
    pub(crate) fn backend(&self) -> &AppBackend {
        &self.0.backend
    }
}
