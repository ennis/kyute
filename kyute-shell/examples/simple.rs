use keyboard_types::KeyboardEvent;
use kyute_common::SizeI;
use kyute_shell::{
    application::Application,
    input::pointer::PointerInputEvent,
    window::{WindowBuilder, WindowHandle, WindowHandler},
};
use std::sync::Arc;

struct Handler;

impl WindowHandler for Handler {
    fn connect(&self, window_handle: WindowHandle) {
        //todo!()
    }

    fn scale_factor_changed(&self, scale_factor: f64) {
        eprintln!("scale_factor_changed: {:?}", scale_factor);
    }

    fn resize(&self, size: SizeI) {
        eprintln!("resize: {:?}", size);
    }

    fn pointer_up(&self, event: &PointerInputEvent) {
        eprintln!("pointer_up: {:?}", event);
    }

    fn pointer_down(&self, event: &PointerInputEvent) {
        eprintln!("pointer_down: {:?}", event);
    }

    fn pointer_move(&self, event: &PointerInputEvent) {
        eprintln!("pointer_move: {:?}", event);
    }

    fn key_up(&self, event: &KeyboardEvent) {
        eprintln!("key_up: {:?}", event);
    }

    fn key_down(&self, event: &KeyboardEvent) {
        eprintln!("key_down: {:?}", event);
    }

    fn close_requested(&self) {
        eprintln!("close_requested");
    }
}

fn main() {
    // create the window
    let win = WindowBuilder::new()
        .title("Hello kyute")
        .build(Arc::new(Handler))
        .unwrap();
    Application::instance().run();
}
