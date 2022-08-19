use graal::{vk::Handle, ImageInfo};
use kyute_common::SizeI;
use kyute_shell::{
    animation::{CompositionLayer, CompositionSurface},
    application::Application,
    window::Window,
};
use skia_safe as sk;
use skia_safe::Color4f;
use std::mem;
use windows::Win32::Graphics::Dxgi::DXGI_PRESENT_DO_NOT_WAIT;
use winit::{
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

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

fn main() {
    let _ = Application::instance();

    //let event_loop = EventLoop::new();
    let window = Window::new(&event_loop, WindowBuilder::new(), None).unwrap();

    let comp_surface = CompositionSurface::new(SizeI::new(512, 512));
    let root_layer = CompositionLayer::new();
    root_layer.set_content(&comp_surface);
    window.set_root_layer(&root_layer);

    let gr_device = Application::instance().gpu_device();
    let skia_backend_context = unsafe { create_skia_vulkan_backend_context(gr_device) };
    let recording_context_options = skia_safe::gpu::ContextOptions::new();
    let mut skia_recording_context =
        skia_safe::gpu::DirectContext::new_vulkan(&skia_backend_context, &recording_context_options)
            .expect("failed to create skia recording context");

    // run event loop
    let mut frame_count = 0;
    let mut prev_frame = graal::GpuFuture::default();
    event_loop.run(move |event, elwt, control_flow| {
        *control_flow = ControlFlow::Wait;
        match event {
            // --- WINDOW EVENT PROCESSING ---------------------------------------------------------
            winit::event::Event::WindowEvent {
                window_id,
                event: winit_event,
            } => {}
            // --- MAIN EVENTS CLEARED -------------------------------------------------------------
            winit::event::Event::MainEventsCleared => {}
            // --- REPAINT -------------------------------------------------------------------------
            winit::event::Event::RedrawRequested(_) => {
                let buf_index = unsafe { comp_surface.swap_chain.GetCurrentBackBufferIndex() };
                eprintln!("RedrawRequested {buf_index}, frame={frame_count}");
                let target_image = &comp_surface.buffers[buf_index as usize];

                let mut gr_ctx = Application::instance().lock_gpu_context();
                let mut frame = gr_ctx.start_frame(graal::FrameCreateInfo {
                    collect_debug_info: false,
                    happens_after: prev_frame,
                });
                let skia_image_usage_flags = graal::vk::ImageUsageFlags::COLOR_ATTACHMENT
                    | graal::vk::ImageUsageFlags::TRANSFER_SRC
                    | graal::vk::ImageUsageFlags::TRANSFER_DST;

                // create the skia render pass
                {
                    let mut ui_render_pass = frame.start_graphics_pass("UI render");
                    // FIXME we just assume how it's going to be used by skia
                    ui_render_pass.add_image_dependency(
                        target_image.id,
                        graal::vk::AccessFlags::MEMORY_READ | graal::vk::AccessFlags::MEMORY_WRITE,
                        graal::vk::PipelineStageFlags::ALL_COMMANDS,
                        graal::vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                        graal::vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                    );

                    let mut recording_context = skia_recording_context.clone();
                    ui_render_pass.set_submit_callback(move |_cctx, _, _queue| {
                        // create skia BackendRenderTarget and Surface
                        let skia_image_info = sk::gpu::vk::ImageInfo {
                            image: target_image.handle.as_raw() as *mut _,
                            alloc: Default::default(),
                            tiling: sk::gpu::vk::ImageTiling::OPTIMAL,
                            layout: sk::gpu::vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                            format: sk::gpu::vk::Format::R8G8B8A8_UNORM,
                            image_usage_flags: skia_image_usage_flags.as_raw(),
                            sample_count: 1,
                            level_count: 1,
                            current_queue_family: sk::gpu::vk::QUEUE_FAMILY_IGNORED,
                            protected: sk::gpu::Protected::No,
                            ycbcr_conversion_info: Default::default(),
                            sharing_mode: sk::gpu::vk::SharingMode::EXCLUSIVE,
                        };
                        let render_target =
                            sk::gpu::BackendRenderTarget::new_vulkan((512 as i32, 512 as i32), 1, &skia_image_info);
                        let mut surface = sk::Surface::from_backend_render_target(
                            &mut recording_context,
                            &render_target,
                            sk::gpu::SurfaceOrigin::TopLeft,
                            sk::ColorType::RGBA8888, // ???
                            sk::ColorSpace::new_srgb(),
                            Some(&sk::SurfaceProps::new(Default::default(), sk::PixelGeometry::RGBH)),
                        )
                        .unwrap();

                        let canvas = surface.canvas();
                        //canvas.clear(Color4f::new(0.1, 0.4, 1.0, 1.0));
                        let mut paint = sk::Paint::new(Color4f::new(0.1, 0.4, 1.0, 1.0), None);
                        //paint.set_stroke(true);
                        paint.set_anti_alias(true);
                        paint.set_stroke_width(10.0);
                        paint.set_style(sk::PaintStyle::Stroke);
                        canvas.clear(Color4f::new(0.0, 0.0, 0.0, 0.0));
                        canvas.draw_circle((200.0, 200.0), 100.0, &paint);

                        surface.flush_and_submit();
                    });

                    ui_render_pass.finish();
                    frame.finish(&mut ());
                }

                // present
                unsafe {
                    comp_surface.swap_chain.Present(0, 0).expect("Present failed");
                }
            }
            _ => (),
        }
    })
}
