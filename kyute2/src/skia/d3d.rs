//! Skia utilities for its direct 3D backend
use crate::{backend::AppBackend, Application, Size};
use skia_safe as sk;
use skia_safe::{
    gpu::{
        d3d::{
            cp, ID3D12CommandQueue as Sk_ID3D12CommandQueue, ID3D12Device as Sk_ID3D12Device,
            ID3D12Resource as Sk_ID3D12Resource, IDXGIAdapter1 as Sk_IDXGIAdapter1, TextureResourceInfo,
            D3D12_RESOURCE_STATES,
        },
        ContextOptions, Protected,
    },
    ColorSpace, ColorType, SurfaceProps,
};
use std::{mem, sync::Arc};
use windows::{
    core::Interface,
    Win32::Graphics::{
        Direct3D12::{ID3D12Resource, D3D12_RESOURCE_STATE_COMMON, D3D12_RESOURCE_STATE_RENDER_TARGET},
        Dxgi::Common::DXGI_FORMAT,
    },
};

pub(crate) struct DrawingBackend {
    pub(crate) backend_context: Option<sk::gpu::d3d::BackendContext>,
    pub(crate) direct_context: Option<sk::gpu::DirectContext>,
}

impl DrawingBackend {
    pub(crate) fn new(app_backend: &AppBackend) -> DrawingBackend {
        // we use the windows crate, skia_safe uses winapi...
        // SAFETY: transfer of ownership from windows types to wio::ComPtr
        let backend_context = unsafe {
            let adapter = cp::from_raw(
                app_backend.adapter.clone().expect("no adapter selected").into_raw() as *mut Sk_IDXGIAdapter1
            );
            let device = cp::from_raw(app_backend.d3d12_device.0.clone().into_raw() as *mut Sk_ID3D12Device);
            let queue =
                cp::from_raw(app_backend.d3d12_command_queue.0.clone().into_raw() as *mut Sk_ID3D12CommandQueue);

            sk::gpu::d3d::BackendContext {
                adapter,
                device,
                queue,
                memory_allocator: None,
                protected_context: Protected::No,
            }
        };

        let direct_context = unsafe {
            // SAFETY: backend_context is valid I guess?
            sk::gpu::DirectContext::new_d3d(&backend_context, None).expect("failed to create D3D context")
        };

        DrawingBackend {
            backend_context: Some(backend_context),
            direct_context: Some(direct_context),
        }
    }

    /// Creates a surface backed by the specified D3D texture resource.
    ///
    /// # Safety
    ///
    /// The parameters must match the properties of the vulkan image:
    ///
    /// * `format`, `size` must be the same as specified during creation of the image
    /// * `color_type` must be compatible with `format`
    ///
    /// TODO: other preconditions
    pub(crate) unsafe fn create_surface_for_texture(
        &mut self,
        image: ID3D12Resource,
        format: DXGI_FORMAT,
        size: Size,
        surface_origin: sk::gpu::SurfaceOrigin,
        color_type: ColorType,
        color_space: ColorSpace,
        surface_props: Option<SurfaceProps>,
    ) -> sk::Surface {
        let resource = unsafe { cp::from_raw(image.into_raw() as *mut Sk_ID3D12Resource) };
        let mut texture_resource_info = TextureResourceInfo::from_resource(resource);
        texture_resource_info.format = unsafe {
            // SAFETY: same type in different bottles
            mem::transmute(format)
        };

        /*let texture_resource_info = TextureResourceInfo {
            resource,
            alloc: None,
            resource_state: D3D12_RESOURCE_STATE_RENDER_TARGET.0 as u32, // FIXME: either pass in parameters or document assumption
            format: unsafe {
                // SAFETY: same type in different bottles
                mem::transmute(format)
            },
            sample_count: 1, // FIXME pass in parameters
            level_count: 1,  // FIXME pass in parameters
            sample_quality_pattern: 0,
            protected: Protected::No,
        };*/

        let backend_render_target =
            sk::gpu::BackendRenderTarget::new_d3d((size.width as i32, size.height as i32), &texture_resource_info);
        let sk_surface = sk::Surface::from_backend_render_target(
            self.direct_context.as_mut().unwrap(),
            &backend_render_target,
            surface_origin,
            color_type,
            color_space,
            surface_props.as_ref(),
        )
        .expect("should not have failed idk");
        sk_surface
    }

    /*/// Flushes commands on the specified surface.
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
    }*/
}
