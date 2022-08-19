use kyute_shell::{
    application::Application,
    input::pointer::PointerInputEvent,
    window::{WindowBuilder, WindowHandler},
};
use std::sync::Arc;

struct Handler;

impl WindowHandler for Handler {
    fn pointer_up(&self, event: &PointerInputEvent) {
        // eprintln!("pointer_up: {:?}", event);
    }
    fn pointer_down(&self, event: &PointerInputEvent) {}
    fn pointer_move(&self, event: &PointerInputEvent) {}
}

fn main() {
    // create the window
    let win = WindowBuilder::new()
        .title("Hello kyute")
        .build(Arc::new(Handler))
        .unwrap();
    Application::instance().run();
}
