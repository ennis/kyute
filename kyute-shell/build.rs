fn main() {
    windows::build!(
        Windows::Win32::Direct2D::*,
        Windows::Win32::Debug::{GetLastError, WIN32_ERROR},
        Windows::Win32::KeyboardAndMouseInput::{GetDoubleClickTime},
        Windows::Win32::SystemServices::{HINSTANCE, GENERIC_READ, GENERIC_ALL, SECURITY_ATTRIBUTES},
        Windows::Win32::WindowsAndMessaging::{HWND},
        Windows::Win32::Dxgi::*,
        Windows::Win32::Direct3D11::*,
        Windows::Win32::DirectWrite::*,
        Windows::Win32::Com::{CoCreateInstance, CoInitialize, CLSCTX},
        Windows::Win32::WindowsImagingComponent::*
    );
}
