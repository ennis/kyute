use glazier::{
    kurbo::Size, raw_window_handle::HasRawWindowHandle, AppHandler, Cursor, FileDialogToken, FileInfo, IdleToken,
    KeyEvent, PointerEvent, Region, Scalable, TimerToken, WinHandler, WindowHandle,
};
use kurbo::Point;
use kyute2::{composition, composition::ColorType, AppGlobals};
use skia_safe as sk;
use std::{
    any::Any,
    time::{Duration, Instant},
};
use tracing::trace_span;
use tracing_subscriber::{layer::SubscriberExt, Layer};

const WIDTH: usize = 2048;
const HEIGHT: usize = 1536;

const UI_UPDATE: IdleToken = IdleToken::new(0);

fn main() {
    tracing::subscriber::set_global_default(tracing_subscriber::registry().with(tracing_tracy::TracyLayer::new()))
        .expect("set up the subscriber");

    let app = AppGlobals::new();

    let window = glazier::WindowBuilder::new(glazier::Application::global())
        .transparent(true)
        .size((WIDTH as f64 / 2., HEIGHT as f64 / 2.).into())
        .handler(Box::new(WindowState::new()))
        .build()
        .unwrap();

    // frame latency situation:
    // - get monitor refresh rate
    // - estimate render time (CPU+GPU) in multiples of the monitor refresh rate (blank interval)
    // - call SetMaximumFrameLatency on the swap chain with the estimated render time as calculated above
    // - when an input event is received, wait on the swap chain (on the frame latency waitable object)
    //      - or possibly, start a timer that will be signalled when the swap chain is ready
    // - process all input events (either after the swap chain is ready, or concurrently)
    // - after (or when wait for swapchain is finished), start a render
    //      /!\ it is crucial that the timer event take priority over any input UI event, otherwise we'll miss the deadline
    //          alternatively, do rendering in a separate thread that wakes when an input event arrives
    //

    window.show();
    glazier::Application::global().run(None);
}

struct WindowState {
    handle: WindowHandle,
    size: Size,
    pos: Point,
    counter: u64,
    main_layer: Option<composition::LayerID>,
    last_render_time: Instant,
    num_frames: u64,
    synced_with_presentation: bool,
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
            pos: Default::default(),
            synced_with_presentation: false,
        }
    }

    fn schedule_render(&mut self) {
        self.handle.invalidate();
    }

    fn render(&mut self) {
        let layer = self.main_layer.unwrap();

        {
            let _span = trace_span!("RENDER").entered();
            let app = AppGlobals::get();
            let surf = app.compositor.acquire_drawing_surface(layer);
            let mut sk_surf = surf.surface();
            let canvas = sk_surf.canvas();
            canvas.clear(sk::Color4f::new(0.9, 0.9, 0.9, 1.0));
            let mut paint = sk::Paint::new(sk::Color4f::new(0.1, 0.4, 1.0, 1.0), None);
            //paint.set_stroke(true);
            paint.set_anti_alias(true);
            paint.set_stroke_width(10.0);
            paint.set_style(sk::PaintStyle::Stroke);
            canvas.clear(sk::Color4f::new(0.1, 0.1, 0.1, 1.0));
            let pos = self.pos;
            let x = pos.x as f32;
            let y = pos.y as f32;
            canvas.draw_circle((x, y), 100.0, &paint);
            paint.set_anti_alias(true);
            paint.set_stroke_width(1.0);
            paint.set_style(sk::PaintStyle::Stroke);
            canvas.draw_line((x + 0.5, y + 0.5 - 100.0), (x + 0.5, y + 0.5 + 100.0), &paint);
            canvas.draw_line((x + 0.5 + 100.0, y + 0.5), (x + 0.5 - 100.0, y + 0.5), &paint);
            app.compositor.release_drawing_surface(layer, surf);
        }

        // before processing any other events, wait for the compositor to be
        // ready for another frame
        {
            let _span = trace_span!("WAIT_FOR_SURFACE").entered();
            let app = AppGlobals::get();
            app.compositor.wait_for_surface(layer);
        }
        {
            let _span = trace_span!("SLEEP").entered();
            spin_sleep::sleep(Duration::from_micros(3500));
        }

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

        {
            let app = AppGlobals::get();
            let size = handle.get_size();
            let raw_window_handle = handle.raw_window_handle();
            let layer_id = app.compositor.create_surface_layer(size, ColorType::RGBAF16);
            unsafe {
                app.compositor.bind_layer(layer_id, raw_window_handle);
            }
            self.main_layer = Some(layer_id);
            app.compositor.wait_for_surface(layer_id);
        }

        self.schedule_render();
    }

    fn size(&mut self, size: Size) {
        self.size = size;
    }

    fn prepare_paint(&mut self) {}

    fn paint(&mut self, _: &Region) {
        let _span = trace_span!("PAINT").entered();
        self.synced_with_presentation = false;
        self.render();
        //self.render();
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
        let _span = trace_span!("event: keydown").entered();
        self.schedule_render();
        false
    }

    fn key_up(&mut self, event: KeyEvent) {
        //println!("keyup: {event:?}");
    }

    fn wheel(&mut self, event: &PointerEvent) {
        println!("wheel {event:?}");
    }

    fn pointer_move(&mut self, event: &PointerEvent) {
        let _span = trace_span!("event: pointer move").entered();
        self.handle.set_cursor(&Cursor::Arrow);
        self.pos = event.pos;
        //println!("pointer_move {event:?}");
        self.schedule_render();
    }

    fn pointer_down(&mut self, event: &PointerEvent) {
        let _span = trace_span!("event: pointer down").entered();
        self.schedule_render();
    }

    fn pointer_up(&mut self, event: &PointerEvent) {
        let _span = trace_span!("event: pointer up").entered();
        self.schedule_render();
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
            let app = AppGlobals::get();
            app.compositor.destroy_layer(layer);
        }
        glazier::Application::global().quit()
    }

    fn idle(&mut self, idle_token: IdleToken) {
        match idle_token {
            UI_UPDATE => {
                let _span = trace_span!("UI_UPDATE").entered();
                self.synced_with_presentation = false;
                self.render();
            }
            _ => {}
        }
    }

    fn as_any(&mut self) -> &mut dyn Any {
        self
    }
}
