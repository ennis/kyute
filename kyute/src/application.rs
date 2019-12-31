use bitflags::bitflags;
use miniqt_sys::*;
use std::ffi::CString;
use std::os::raw::c_char;
use std::sync::Once;
use std::{env, mem};

bitflags! {
    #[derive(Default)]
    pub struct ProcessEventFlags: u32 {
         const ALL_EVENTS = QEventLoop_ProcessEventsFlag_AllEvents as u32;
         const EXCLUDE_USER_INPUT_EVENTS = QEventLoop_ProcessEventsFlag_ExcludeUserInputEvents as u32;
         const EXCLUDE_SOCKET_NOTIFIERS = QEventLoop_ProcessEventsFlag_ExcludeSocketNotifiers as u32;
         const WAIT_FOR_MORE_EVENTS = QEventLoop_ProcessEventsFlag_WaitForMoreEvents as u32;
    }
}

pub struct Application {
    _raw: *mut QApplication,
    argv_len: usize,
    argv_capacity: usize,
    argc: *mut i32,
    argv: *mut *mut c_char,
}

impl Application {
    pub fn new() -> Application {
        unsafe {
            // for some reason Qt wants it to stay valid for the entire lifetime of the QApplication object
            // The data of argv should have a stable address as long as the vector is not resized.
            let mut argv_vec = env::args()
                .map(|arg| CString::new(arg).unwrap().into_raw())
                .collect::<Vec<*mut c_char>>();
            let argv_len = argv_vec.len();
            let argv_capacity = argv_vec.capacity();
            let argv = argv_vec.as_mut_ptr();
            mem::forget(argv_vec); // we will rebuild it from raw parts in the destructor
                                   // box argc so that it has a stable address and stash it along the application wrapper
            let argc = Box::into_raw(Box::new(argv_len as i32));

            Application {
                _raw: QApplication_new(argc, argv),
                argv_len,
                argv_capacity,
                argc,
                argv,
            }
        }
    }

    pub fn process_events(flags: ProcessEventFlags) {
        unsafe {
            QCoreApplication_processEvents(flags.bits() as u32);
        }
    }
}

impl Drop for Application {
    fn drop(&mut self) {
        // Safety: we initialized self.argc with Box::into_raw
        let _argc = unsafe { Box::<i32>::from_raw(self.argc) };

        // Safety: we initialized self.argv with data from a Vec that we mem::forgot
        let _argv = unsafe { Vec::from_raw_parts(self.argv, self.argv_len, self.argv_capacity) };

        // TODO The docs of QAppplication say:
        // "Note: argc and argv might be changed as Qt removes command line arguments that it recognizes."
        // This means that some pointers in argv might have been set to NULL, and we can't recover
        // the memory for them with CString::from_raw.
        // Since we don't really know WTF Qt is doing with argv, we just leak the memory of the
        // CStrings created in Application::new(). It's a very small amount of memory,
        // and there should only be one instance of QApplication during program execution.
        // However, it might show up in Valgrind and other leak detectors, which is annoying.
    }
}

// Qt Application initialization flag
static QT_APPLICATION_INIT_ONCE: Once = Once::new();

/// Call this function before calling a Qt function or constructor to ensure that the main
/// application object is created.
pub fn ensure_qt_initialized() {
    QT_APPLICATION_INIT_ONCE.call_once(|| {
        let app = Application::new();
        mem::forget(app);
    });
}
