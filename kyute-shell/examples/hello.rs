use kyute_shell::{
    drawing::{Brush, Color, DrawContext, DrawTextOptions, Point, Size},
    platform::Platform,
    text::{TextFormat, TextLayout},
    window::{DrawSurface, PlatformWindow},
    winit::{
        event::{Event, WindowEvent},
        event_loop::{ControlFlow, EventLoop},
        platform::windows::WindowBuilderExtWindows,
        window::WindowBuilder,
    },
};

fn main() {
    // winit event loop
    let event_loop = EventLoop::new();
    // platform-specific, window-independent initialization
    let platform = unsafe { Platform::init() };

    let mut window_builder = WindowBuilder::new();
    let mut window = PlatformWindow::new(&event_loop, window_builder, None).unwrap();
    let mut draw_surface =
        DrawSurface::new(window.swap_chain_size(), window.window().scale_factor());

    event_loop.run(move |event, elwt, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::WindowEvent { window_id, event } => match event {
                WindowEvent::CloseRequested => {
                    *control_flow = ControlFlow::Exit;
                }
                WindowEvent::Resized(size) => {
                    window.resize(size.into());
                    draw_surface.resize(size.into(), window.window().scale_factor());
                }
                _ => {}
            },
            Event::RedrawRequested(window_id) => {
                let mut context = window.gpu_context().lock().unwrap();
                let swapchain = window.swap_chain();
                let target_image = unsafe { context.acquire_next_image(swapchain.id) };
                let mut frame = context.start_frame(graal::FrameCreateInfo::default());
                frame.finish();

                // acquire the draw surface
                //draw_surface.

                /*{
                    let mut dc = WindowDrawContext::new(&mut window);
                    let text_format = TextFormat::builder().size(12.0).build().unwrap();
                    let text =
                        TextLayout::new("Hello world", &text_format, Size::new(200.0, 100.0))
                            .unwrap();
                    let text_color = Brush::new_solid_color(&dc, Color::new(1.0, 1.0, 1.0, 1.0));
                    dc.clear(Color::new(0.0, 0.1, 0.2, 1.0));
                    dc.draw_text_layout(
                        Point::new(10.0, 10.0),
                        &text,
                        &text_color,
                        DrawTextOptions::default(),
                    );
                }*/
            }
            Event::MainEventsCleared => {
                // window.window().request_redraw();
            }
            _ => (),
        }
    });
}
