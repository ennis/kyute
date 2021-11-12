use kyute::{application, composable, widget::{Axis, Button, Flex, Text}, BoxConstraints, Cache, Data, Environment, Event, LayoutCtx, Measurements, PaintCtx, Rect, Widget, WidgetPod, Window, EventCtx};
use kyute_shell::{platform::Platform, winit::window::WindowBuilder};
use std::sync::Arc;

#[composable(uncached)]
fn root() -> WidgetPod<Window> {
    window()
}

#[composable(uncached)]
fn window() -> WidgetPod<Window> {
    todo!()
}

#[composable(uncached)]
fn vbox() -> WidgetPod<Flex> {
    todo!()
}

#[derive(Clone, Data)]
struct AppState {
    items: Arc<Vec<u32>>,
    value: f64,
}

impl Default for AppState {
    fn default() -> Self {
        AppState {
            items: Arc::new(vec![]),
            value: 0.0,
        }
    }
}

#[composable]
fn ui_function() -> WidgetPod {
    Cache::with_state(
        || AppState::default(),
        move |app_state| {
            eprintln!("recomputing ui_function");

            // "Add Item"
            let add_item_button = Button::new("Add Item".to_string());
            if add_item_button.clicked() {
                let new_item = app_state.items.len() as u32;
                Arc::make_mut(&mut app_state.items).push(new_item);
            }

            // List of items
            let mut item_list = vec![];
            for item in app_state.items.iter() {
                Cache::scoped(*item as usize,  || {
                    item_list.push(Text::new(format!("{}", item)).into());
                });
            }
            item_list.push(add_item_button.into());
            let flex = Flex::new(Axis::Vertical, item_list).into();

            // enclosing window
            Window::new(WindowBuilder::new().with_title("hello"), flex).into()
        },
    )
}

fn main() {
    let platform = Platform::new();

    tracing_subscriber::fmt()
        .compact()
        .with_target(false)
        //.with_level(false)
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        //.with_span_events(tracing_subscriber::fmt::format::FmtSpan::ACTIVE)
        .init();

    application::run(ui_function);
    Platform::shutdown();
}

// issue: how do you write a composable function that focuses "down" on some state but retains
// the ability to modify it?
// what about arbitrarily deep tree data structures?
/*#[composable]
fn item_gui(item: &mut Item) -> Widget {
    // don't modify state in closure, instead, just mark the call to `on_click` as dirty.
    Button::new("change_name").on_click(|| item.name = "Hello".into());

    // .on_click is actually:
    // #[composable] fn on_click() -> bool { }
    // which is cached
    // in the end, the root state entry will be marked as a dependency of the revision of the button
    item.clone()
}

#[composable]
fn gui() -> Widget {
    // parent cache entry now depends on state
    let mut items = Context::state(|| Vec::new());

    // this creates a new vbox every time...
    let mut vbox = Flex::new(Axis::Vertical);

    for item in items.iter_mut() {
        // ... but this call is cached
        Context::use_id(item.unique_id, || {
            let widget = item_gui(item);
            vbox.push(widget);
        });
    }

    Widget::new(vbox).into()
}
*/
