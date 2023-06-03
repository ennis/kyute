use glazier::{
    kurbo::Size, raw_window_handle::HasRawWindowHandle, AppHandler, Cursor, FileDialogToken, FileInfo, IdleToken,
    KeyEvent, PointerEvent, Region, Scalable, TimerToken, WinHandler, WindowHandle,
};
use kyute2::{composition, composition::ColorType, Application};
use skia_safe as sk;
use std::{
    any::Any,
    time::{Duration, Instant},
};
use tracing_subscriber::{layer::SubscriberExt, Layer};

const WIDTH: usize = 2048;
const HEIGHT: usize = 1536;

fn main() {
    tracing_subscriber::fmt()
        .compact()
        .with_target(false)
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let app = Application::new();

    let window = glazier::WindowBuilder::new(glazier::Application::global())
        .transparent(true)
        .size((WIDTH as f64 / 2., HEIGHT as f64 / 2.).into())
        .handler(Box::new(WindowState::new()))
        .build()
        .unwrap();

    window.get_idle_handle().unwrap();

    window.show();
    app.run(None);
}

struct WindowState {
    handle: WindowHandle,
    size: Size,
    counter: u64,
    main_layer: Option<composition::LayerID>,
    last_render_time: Instant,
    num_frames: u64,
}

impl WindowState {
    pub fn new() -> Self {
        Self {
            handle: Default::default(),
            counter: 0,
            size: Size::new(800.0, 600.0),
            main_layer: None,
            last_render_time: Instant::now(),
            num_frames: 0,
        }
    }

    fn schedule_render(&self) {
        self.handle.invalidate();
    }

    fn render(&mut self) {
        let layer = self.main_layer.unwrap();
        let app = Application::global();
        let mut compositor = app.compositor();
        let surf = compositor.acquire_drawing_surface(layer);
        let mut sk_surf = surf.surface();
        let canvas = sk_surf.canvas();
        //canvas.clear(sk::Color4f::new(0.9, 0.9, 0.9, 1.0));

        let mut paint = sk::Paint::new(sk::Color4f::new(0.1, 0.4, 1.0, 1.0), None);
        //paint.set_stroke(true);
        paint.set_anti_alias(true);
        paint.set_stroke_width(10.0);
        paint.set_style(sk::PaintStyle::Stroke);
        //canvas.clear(sk::Color4f::new(0.0, 0.0, 0.0, 0.0));
        let pos = self.size / 2.0;
        //canvas.draw_circle((pos.width as f32, pos.height as f32), 100.0, &paint);

        compositor.release_drawing_surface(layer, surf);
        let now = Instant::now();
        let delta = now.duration_since(self.last_render_time);
        self.num_frames += 1;
        /*eprintln!(
            "avg frame time since launch = {}ms ({} FPS)",
            delta.as_millis() as f64 / self.num_frames as f64,
            self.num_frames as f64 / delta.as_secs_f64()
        );*/
    }
}

impl WinHandler for WindowState {
    fn connect(&mut self, handle: &WindowHandle) {
        self.handle = handle.clone();

        let app = Application::global();
        let size = handle.get_size();
        let raw_window_handle = handle.raw_window_handle();
        let mut compositor = app.compositor();
        let layer_id = compositor.create_surface_layer(size, ColorType::RGBAF16);

        unsafe {
            compositor.bind_layer(layer_id, raw_window_handle);
        }

        self.main_layer = Some(layer_id);
        self.schedule_render();
    }

    fn size(&mut self, size: Size) {
        self.size = size;
    }

    fn prepare_paint(&mut self) {}

    fn paint(&mut self, _: &Region) {
        self.render();
        //self.schedule_render();
    }

    fn command(&mut self, _id: u32) {}

    fn save_as(&mut self, _token: FileDialogToken, file: Option<FileInfo>) {
        println!("save file result: {file:?}");
    }

    fn open_file(&mut self, _token: FileDialogToken, file_info: Option<FileInfo>) {
        println!("open file result: {file_info:?}");
    }

    fn key_down(&mut self, event: KeyEvent) -> bool {
        println!("keydown: {event:?}");
        false
    }

    fn key_up(&mut self, event: KeyEvent) {
        println!("keyup: {event:?}");
    }

    fn wheel(&mut self, event: &PointerEvent) {
        println!("wheel {event:?}");
    }

    fn pointer_move(&mut self, _event: &PointerEvent) {
        self.handle.set_cursor(&Cursor::Arrow);
        //println!("pointer_move {event:?}");
    }

    fn pointer_down(&mut self, event: &PointerEvent) {
        println!("pointer_down {event:?}");
    }

    fn pointer_up(&mut self, event: &PointerEvent) {
        println!("pointer_up {event:?}");
    }

    fn timer(&mut self, id: TimerToken) {
        println!("timer fired: {id:?}");
    }

    fn got_focus(&mut self) {
        println!("Got focus");
    }

    fn lost_focus(&mut self) {
        println!("Lost focus");
    }

    fn request_close(&mut self) {
        self.handle.close();
    }

    fn destroy(&mut self) {
        if let Some(layer) = self.main_layer {
            let app = Application::global();
            let mut compositor = app.compositor();
            compositor.destroy_layer(layer);
        }
        glazier::Application::global().quit()
    }

    fn idle(&mut self, _: IdleToken) {}

    fn as_any(&mut self) -> &mut dyn Any {
        self
    }
}
