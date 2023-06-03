//! Skia utilities for vulkan
use graal::vk::Handle;
use kurbo::Size;
use skia_safe as sk;
use skia_safe::{ColorSpace, ColorType, SurfaceProps};
use std::{mem, sync::Arc};

////////////////////////////////////////////////////////////////////////////////////////////////////
// DrawingContext
////////////////////////////////////////////////////////////////////////////////////////////////////

pub(crate) struct DrawingContext {
    image: graal::ImageInfo,
    sk_surface: sk::Surface,
}

impl DrawingContext {
    pub(crate) fn surface(&self) -> &sk::Surface {
        &self.sk_surface
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
//
////////////////////////////////////////////////////////////////////////////////////////////////////

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

/// Creates a GrBackendContext bound to the specified vulkan device.
///
/// # Safety
///
/// The device must not be destroyed before the BackendContext.
pub(crate) unsafe fn create_skia_vulkan_backend_context(
    device: &graal::Device,
) -> sk::gpu::vk::BackendContext<'static> {
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

    //ctx.set_max_api_version(sk::gpu::vk::Version::new(1, 0, 0));
    ctx.set_max_api_version(sk::gpu::vk::Version::new(1, 0, 0));
    ctx
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// GpuBackend
////////////////////////////////////////////////////////////////////////////////////////////////////

pub(crate) struct DrawingBackend {
    /// This is an Arc because it's still OK to send it across threads.
    pub(crate) device: Arc<graal::Device>,
    pub(crate) context: graal::Context,
    pub(crate) skia_backend_context: Option<sk::gpu::vk::BackendContext<'static>>,
    pub(crate) skia_recording_context: Option<sk::gpu::DirectContext>,
}

impl DrawingBackend {
    pub(crate) fn new(_app_backend: &AppBackend) -> DrawingBackend {
        // GPU device & context
        let (device, context) = unsafe { graal::create_device_and_context(None) };

        // Skia backend
        // SAFETY: we make sure that the GPU device is destroyed after the skia backend
        let skia_backend_context = unsafe { create_skia_vulkan_backend_context(&device) };

        let recording_context_options = sk::gpu::ContextOptions::new();
        let skia_recording_context =
            sk::gpu::DirectContext::new_vulkan(&skia_backend_context, &recording_context_options)
                .expect("failed to create skia recording context");

        DrawingBackend {
            device,
            context,
            skia_backend_context: Some(skia_backend_context),
            skia_recording_context: Some(skia_recording_context),
        }
    }

    /// Creates a surface backed by the specified vulkan image.
    ///
    /// # Safety
    ///
    /// The parameters must match the properties of the vulkan image:
    ///
    /// * `format`, `size` must be the same as specified during creation of the image
    /// * `color_type` must be compatible with `format`
    ///
    /// TODO: other preconditions
    pub(crate) unsafe fn create_surface_for_vulkan_image(
        &mut self,
        image: graal::ImageInfo,
        format: graal::vk::Format,
        size: Size,
        surface_origin: sk::gpu::SurfaceOrigin,
        color_type: ColorType,
        color_space: ColorSpace,
        surface_props: Option<SurfaceProps>,
    ) -> sk::Surface {
        // create the skia counterpart of the native surface (BackendRenderTarget and Surface)
        let skia_image_usage_flags = graal::vk::ImageUsageFlags::COLOR_ATTACHMENT
            | graal::vk::ImageUsageFlags::TRANSFER_SRC
            | graal::vk::ImageUsageFlags::TRANSFER_DST;
        let skia_image_info = sk::gpu::vk::ImageInfo {
            image: image.handle.as_raw() as *mut _,
            alloc: Default::default(),
            tiling: sk::gpu::vk::ImageTiling::OPTIMAL,
            layout: sk::gpu::vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
            format: unsafe { mem::transmute(format) },
            image_usage_flags: skia_image_usage_flags.as_raw(),
            sample_count: 1,
            level_count: 1,
            current_queue_family: sk::gpu::vk::QUEUE_FAMILY_IGNORED,
            protected: sk::gpu::Protected::No,
            ycbcr_conversion_info: Default::default(),
            sharing_mode: sk::gpu::vk::SharingMode::EXCLUSIVE,
        };
        let backend_render_target =
            sk::gpu::BackendRenderTarget::new_vulkan((size.width as i32, size.height as i32), 1, &skia_image_info);
        let sk_surface = sk::Surface::from_backend_render_target(
            self.skia_recording_context.as_mut().unwrap(),
            &backend_render_target,
            surface_origin,
            color_type,
            color_space,
            surface_props.as_ref(),
        )
        .unwrap();
        sk_surface
    }

    /// Flushes commands on the specified surface.
    ///
    /// # Arguments
    ///
    /// * surface the surface returned by `create_surface_for_vulkan_image`
    /// * image handle to the vulkan image backing the surface (the one that was passed to `create_surface_for_vulkan_image`)
    ///
    /// # Safety
    ///
    /// * `image` must specify a valid image (must not have been deleted)
    /// * `surface` must have been created by `create_surface_for_vulkan_image`
    /// * `image` must be the backing image for `surface` as specified in a prior call to `create_surface_for_vulkan_image`.
    pub(crate) unsafe fn flush_surface_for_vulkan_image(&mut self, mut surface: sk::Surface, image: graal::ImageInfo) {
        // flush the GPU frame
        //let _span = trace_span!("Flush skia surface").entered();
        let mut frame = graal::Frame::new();
        let pass = graal::PassBuilder::new()
            .name("flush SkSurface")
            .image_dependency(
                // FIXME we just assume how it's going to be used by skia
                image.id,
                graal::vk::AccessFlags::MEMORY_READ | graal::vk::AccessFlags::MEMORY_WRITE,
                graal::vk::PipelineStageFlags::ALL_COMMANDS,
                graal::vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                graal::vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
            )
            .submit_callback(move |_cctx, _, _queue| {
                surface.flush_and_submit();
            });
        frame.add_pass(pass);
        self.context.submit_frame(&mut (), frame, &Default::default());
    }
}

impl Drop for DrawingBackend {
    fn drop(&mut self) {
        // make sure that skia contexts are deleted before the vulkan device
        self.skia_backend_context.take();
        self.skia_recording_context.take();
    }
}
