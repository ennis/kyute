use crate::{graal, graal::vk::Handle};
use skia_safe as sk;

fn skia_get_proc_addr(of: sk::gpu::vk::GetProcOf) -> sk::gpu::vk::GetProcResult {
    unsafe {
        let entry = graal::get_vulkan_entry();
        let instance = graal::get_vulkan_instance();

        match of {
            sk::gpu::vk::GetProcOf::Instance(instance, name) => entry
                .get_instance_proc_addr(graal::vk::Instance::from_raw(instance as u64), name)
                .unwrap() as sk::gpu::vk::GetProcResult,
            sk::gpu::vk::GetProcOf::Device(device, name) => instance
                .get_device_proc_addr(graal::vk::Device::from_raw(device as u64), name)
                .unwrap() as sk::gpu::vk::GetProcResult,
        }
    }
}

unsafe fn create_skia_vulkan_backend_context(device: &graal::Device) -> sk::gpu::vk::BackendContext<'static> {
    let vk_device = device.device.handle();
    let vk_instance = graal::get_vulkan_instance().handle();
    let vk_physical_device = device.physical_device();
    let (vk_queue, vk_queue_family_index) = device.graphics_queue();
    let instance_extensions = graal::get_instance_extensions();

    let mut ctx = sk::gpu::vk::BackendContext::new_with_extensions(
        vk_instance.as_raw() as *mut _,
        vk_physical_device.as_raw() as *mut _,
        vk_device.as_raw() as *mut _,
        (vk_queue.as_raw() as *mut _, vk_queue_family_index as usize),
        &skia_get_proc_addr,
        instance_extensions,
        &[],
    );

    ctx.set_max_api_version(sk::gpu::vk::Version::new(1, 0, 0));
    ctx
}

/// A combination of a `kyute_shell::Window` and a skia `BackendContext` and `RecordingContext`
pub(crate) struct SkiaWindow {
    pub(crate) window: kyute_shell::window::Window,
    pub(crate) skia_backend_context: skia_safe::gpu::vk::BackendContext<'static>,
    pub(crate) skia_recording_context: skia_safe::gpu::DirectContext,
}

impl SkiaWindow {
    /// Creates a `BackendContext` and a `RecordingContext` and wraps them in a `SkiaWindow`.
    pub(crate) fn new(window: kyute_shell::window::Window) -> SkiaWindow {
        let application = kyute_shell::application::Application::instance();
        let device = application.gpu_device().clone();
        let skia_backend_context = unsafe { create_skia_vulkan_backend_context(&device) };
        let recording_context_options = skia_safe::gpu::ContextOptions::new();
        let skia_recording_context =
            skia_safe::gpu::DirectContext::new_vulkan(&skia_backend_context, &recording_context_options)
                .expect("failed to create skia recording context");
        SkiaWindow {
            window,
            skia_recording_context,
            skia_backend_context,
        }
    }
}
