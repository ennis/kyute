use kyute::{
    application, get_default_application_style, theme, widget::ButtonResult, CompositionCtx,
    Environment,
};
use kyute_shell::{drawing::Color, platform::Platform};

fn gui(cx: &mut CompositionCtx) {
    use kyute::widget as w;

    // Using functions directly is a bit impractical in rust, because of the lack of optional parameters

    // returns a MutableState<f64>

    cx.with_environment(get_default_application_style(), |cx| {
        cx.with_state(
            || 0.0,
            |cx, counter| {
                w::window(cx, "Kyute main window", |cx| {
                    w::vbox(cx, |cx| {
                        w::button(cx, &format!("click me: {}", counter));
                        w::button(cx, &format!("click me again: {}", counter))
                            .on_click(|| *counter += 42.0);
                        w::slider(cx, 0.0, 100.0, *counter).on_value_change(|v| *counter = v);

                        cx.with_state(String::new, |cx, str| {
                            w::text_line_edit(cx, str).on_text_changed(|s| *str = s.to_string());
                            w::text(cx, str);
                        });
                    });
                });
            },
        );
    });

    w::EnvWrapper::new(get_default_application_style())
        .contents(w::StateWrapper::new::<f64>(|| 0.0).contents());
}

/*
// three ways:
// - raw rust code
// - macro-assisted
// - full DSL
#[composable]
fn gui2(state: &mut State) {
    use kyute::widget as w;

    view!{
        w::window(.title = "Kyute main window")
        {
            // let and let mut bindings (internal state)
            let mut counter = 0.0;
            // for loops

            for i in 0..10 {
                w::text(.text = format!("label #{}", i))
                + w::padding(0.5)
                + w::align(baseline
            }

            w::vbox {
                w::button(&format!("click me: {}", counter));
                w::button(&format!("click me again: {}", counter)).on_click(|| *counter += 42.0);
            }
        }
    }

    with_environment!(get_default_application_style(),
        {
            #[state] let mut counter = 0.0;
            w::window("Kyute main window") {
                w::vbox {
                    w::button(&format!("click me: {}", counter));
                    w::button(&format!("click me again: {}", counter)).on_click(|| *counter += 42.0);
                }
            }
        }
    );
}*/

/*// key difference with druid: we have a concrete reference to the node,
// and access stuff from it, instead of "building a function that will take a node as an argument"
// (roughly), using lenses and stuff.
//
// However, we do need to pass the CompositionCtx around, which is annoying.
fn node_row(cx: &mut CompositionCtx, depth: u32, node: &mut Node) {

    w::vbox(cx, |cx| {
        // name row
        w::hbox(cx, |cx| {
            w::label(cx, "name");
            w::text_box(cx, &node.name).on_text_changed(Action::RenameNode);  // RenameNode is handled by the function above
        });

        // button to add a child node
        w::button(cx, "Add child")
            .on_click(|| {
                let mut new_node = node.clone();
                new_node.children.push(Node::new());
                new_node
            });


        // children
        for n in node.children.iter_mut() {
            node_row(cx, depth + 1, n);
        }
    });
}*/

fn main() {
    Platform::init();
    tracing_subscriber::fmt()
        .compact()
        .with_target(false)
        //.with_level(false)
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        //.with_span_events(tracing_subscriber::fmt::format::FmtSpan::ACTIVE)
        .init();
    application::run(gui);
}
