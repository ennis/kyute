use kyute_shell::{
    drawing::{Brush, Color, DrawContext, DrawTextOptions, Point, Size},
    platform::Platform,
    text::{TextFormat, TextLayout},
    window::{Window, WindowDrawContext},
    winit::{
        event::{Event, WindowEvent},
        event_loop::{ControlFlow, EventLoop},
        window::WindowBuilder,
    },
};

fn main() {
    // platform-specific, window-independent initialization
    Platform::init();

    // winit event loop
    let event_loop = EventLoop::new();

    let mut window_builder = WindowBuilder::new();
    let mut window = Window::new(&event_loop, window_builder, None).unwrap();

    event_loop.run(move |event, elwt, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::WindowEvent { window_id, event } => match event {
                WindowEvent::CloseRequested => {
                    *control_flow = ControlFlow::Exit;
                }
                WindowEvent::Resized(size) => {
                    window.resize(size.into());
                    window.window().request_redraw();
                }
                _ => {}
            },
            Event::RedrawRequested(window_id) => {
                {
                    let mut dc = WindowDrawContext::new(&mut window);
                    let text_format = TextFormat::builder().size(12.0).build().unwrap();
                    let text =
                        TextLayout::new("Hello world", &text_format, Size::new(200.0, 100.0))
                            .unwrap();
                    let text_color = Brush::solid_color(&dc, Color::new(1.0, 1.0, 1.0, 1.0));
                    dc.clear(Color::new(0.0, 0.1, 0.2, 1.0));
                    dc.draw_text_layout(
                        Point::new(10.0, 10.0),
                        &text,
                        &text_color,
                        DrawTextOptions::default(),
                    );
                }
                window.present();
            }
            Event::MainEventsCleared => {
                //window.window().request_redraw();
            }
            _ => (),
        }
    });
}
