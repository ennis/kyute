use kyute::application::{run_application, Application};
use kyute::{BoxedWidget, WidgetExt};

struct SimpleApplication;

impl Application for SimpleApplication {
    type Action = ();

    fn update(&mut self, _actions: &[()]) {
        unimplemented!()
    }

    fn view(&mut self) -> BoxedWidget<()> {
        use kyute::widget::*;
        Flex::new(Axis::Vertical)
            .push(
                Flex::new(Axis::Horizontal)
                    .push(Baseline::new(20.0, Text::new("Horizontal Flex: ")))
                    .push(Baseline::new(20.0, Button::new("Button A")))
                    .push(Baseline::new(20.0, Button::new("Button B")))
                    .push(Baseline::new(20.0, Text::new("Baseline alignment test"))),
            )
            .push(
                Flex::new(Axis::Vertical)
                    .push(Button::new("Vertical Flex:"))
                    .push(Button::new("Button A"))
                    .push(Button::new("Button B"))
                    .push(Text::new("Hello world")),
            )
            .boxed()
    }
}

fn main() {
    pretty_env_logger::init();

    run_application(SimpleApplication);
}
