use crate::AppGlobals;
use glazier::WindowHandle;
use kyute_compose::Cache;
use std::{
    cell::RefCell,
    rc::Rc,
    sync::{Arc, Mutex},
    task::{Wake, Waker},
};
use tracing::{trace, warn};

////////////////////////////////////////////////////////////////////////////////////////////////////

struct AppStateInner {
    windows: Vec<WindowHandle>,
    cache: Cache,
    logic: Box<dyn FnMut(AppHandle)>,
}

#[derive(Clone)]
pub struct AppState(Rc<RefCell<AppStateInner>>);

impl AppState {}

struct AppHandler {
    app_state: AppState,
}

impl AppHandler {
    pub fn run_ui(&self, force: bool) {
        let mut app_state = self.app_state.0.borrow_mut();
        let app_state = &mut *app_state;
        if app_state.cache.is_dirty() || force {
            trace!("AppHandler: running app logic");
            let app = glazier::Application::global();
            let app_handle = AppHandle(app.get_handle().unwrap());
            app_state.cache.run(|| (app_state.logic)(app_handle.clone()));
        } else {
            //trace!("AppHandler: cache clean, app logic will be skipped");
        }
    }
}

pub(crate) const UPDATE_UI_CMD: u32 = 0;

impl glazier::AppHandler for AppHandler {
    fn command(&mut self, id: u32) {
        match id {
            UPDATE_UI_CMD => self.run_ui(false),
            _ => {
                warn!("AppHandler: unknown command {id}")
            }
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
#[derive(Clone)]
pub struct AppHandle(glazier::AppHandle);

impl AppHandle {
    pub(crate) fn schedule_update(&self) {
        self.0.run_on_main(|app_handler| {
            if let Some(app_handler) = app_handler {
                app_handler.command(UPDATE_UI_CMD)
            }
        });
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

/// Waker for the application event loop.
///
/// TODO document
struct AppWaker(Mutex<glazier::AppHandle>);

impl Wake for AppWaker {
    fn wake(self: Arc<Self>) {
        self.0.lock().unwrap().run_on_main(|app_handler| {
            if let Some(app_handler) = app_handler {
                app_handler.command(UPDATE_UI_CMD);
            }
        })
    }

    fn wake_by_ref(self: &Arc<Self>) {
        self.0.lock().unwrap().run_on_main(|app_handler| {
            if let Some(app_handler) = app_handler {
                app_handler.command(UPDATE_UI_CMD);
            }
        })
    }
}

pub struct AppLauncher {
    ui_fn: Box<dyn FnMut(AppHandle)>,
}

impl AppLauncher {
    pub fn new(ui_fn: impl FnMut(AppHandle) + 'static) -> AppLauncher {
        AppLauncher { ui_fn: Box::new(ui_fn) }
    }

    pub fn run(self) {
        AppGlobals::new();
        let app = glazier::Application::global();
        let app_handle = app.get_handle().unwrap();
        let waker = Waker::from(Arc::new(AppWaker(Mutex::new(app_handle))));
        let app_state = AppState(Rc::new(RefCell::new(AppStateInner {
            windows: vec![],
            cache: Cache::new(waker),
            logic: Box::new(self.ui_fn),
        })));
        let mut handler = AppHandler { app_state };
        // run UI at least once to create the initial windows
        handler.run_ui(true);
        app.run(Some(Box::new(handler)));
    }
}
