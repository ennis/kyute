use kyute_shell::MainEventLoop;
use kyute_shell::WindowEventTarget;
use kyute_shell::WindowCtx;
use kyute_shell::EventResult;
use kyute_shell::platform::PlatformWindow;
use kyute_shell::platform::OpenGlDrawContext;
use kyute::{Widget, Visual, BoxedWidget, WidgetExt};
use kyute::Painter;
use kyute::application::{Application, run_application};


struct SimpleApplication;

impl Application for SimpleApplication {
    type Action = ();

    fn update(&mut self, actions: &[()]) {
        unimplemented!()
    }

    fn view(&mut self) -> BoxedWidget<()>
    {
        use kyute::widget::*;
        Flex::new(Axis::Vertical)
            .push(
                Flex::new(Axis::Horizontal)
                    .push(Baseline::new(20.0, Button::new("Click me please")))
                    .push(Baseline::new(20.0, Button::new("Refrain")))
                    .push(Baseline::new(20.0, Text::new("Hello worlp"))),
            )
            .push(
                Flex::new(Axis::Horizontal)
                    .push(Baseline::new(25.0, Button::new("Line 2")))
                    .push(Baseline::new(25.0, Button::new("AAAA")))
                    .push(Baseline::new(25.0, Text::new("Hello agaim"))),
            ).boxed()
    }
}

fn main() {
    pretty_env_logger::init();

    run_application(SimpleApplication);
}