pub mod api;

use api::gl;
use api::gl::types::*;
use api::Gl;
use std::os::raw::c_void;
use std::ptr;

/// Sets up the OpenGL debug output so that we have more information in case the interop fails.
pub(crate) unsafe fn init_debug_callback(gl: &Gl) {
    gl.Enable(gl::DEBUG_OUTPUT);
    gl.Enable(gl::DEBUG_OUTPUT_SYNCHRONOUS);

    if gl.DebugMessageCallback.is_loaded() {
        extern "system" fn debug_callback(
            source: GLenum,
            gltype: GLenum,
            _id: GLuint,
            severity: GLenum,
            _length: GLsizei,
            message: *const GLchar,
            _user_param: *mut c_void,
        ) {
            unsafe {
                use std::ffi::CStr;
                let message = CStr::from_ptr(message);
                eprintln!("{:?}", message);
                match source {
                    gl::DEBUG_SOURCE_API => eprintln!("Source: API"),
                    gl::DEBUG_SOURCE_WINDOW_SYSTEM => eprintln!("Source: Window System"),
                    gl::DEBUG_SOURCE_SHADER_COMPILER => eprintln!("Source: Shader Compiler"),
                    gl::DEBUG_SOURCE_THIRD_PARTY => eprintln!("Source: Third Party"),
                    gl::DEBUG_SOURCE_APPLICATION => eprintln!("Source: Application"),
                    gl::DEBUG_SOURCE_OTHER => eprintln!("Source: Other"),
                    _ => (),
                }

                match gltype {
                    gl::DEBUG_TYPE_ERROR => eprintln!("Type: Error"),
                    gl::DEBUG_TYPE_DEPRECATED_BEHAVIOR => eprintln!("Type: Deprecated Behaviour"),
                    gl::DEBUG_TYPE_UNDEFINED_BEHAVIOR => eprintln!("Type: Undefined Behaviour"),
                    gl::DEBUG_TYPE_PORTABILITY => eprintln!("Type: Portability"),
                    gl::DEBUG_TYPE_PERFORMANCE => eprintln!("Type: Performance"),
                    gl::DEBUG_TYPE_MARKER => eprintln!("Type: Marker"),
                    gl::DEBUG_TYPE_PUSH_GROUP => eprintln!("Type: Push Group"),
                    gl::DEBUG_TYPE_POP_GROUP => eprintln!("Type: Pop Group"),
                    gl::DEBUG_TYPE_OTHER => eprintln!("Type: Other"),
                    _ => (),
                }

                match severity {
                    gl::DEBUG_SEVERITY_HIGH => eprintln!("Severity: high"),
                    gl::DEBUG_SEVERITY_MEDIUM => eprintln!("Severity: medium"),
                    gl::DEBUG_SEVERITY_LOW => eprintln!("Severity: low"),
                    gl::DEBUG_SEVERITY_NOTIFICATION => eprintln!("Severity: notification"),
                    _ => (),
                }
                panic!();
            }
        }
        gl.DebugMessageCallback(Some(debug_callback), ptr::null());
    }
}
