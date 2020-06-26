//! winit-based application wrapper.
//!
//! Provides the `run_application` function that opens the main window and translates the incoming
//! events from winit into the events expected by a kyute [`NodeTree`](crate::node::NodeTree).
use crate::component::Component;
use crate::event::{
    Event, InputEvent, InputState, KeyboardEvent, PointerButton, PointerButtonEvent,
    PointerButtons, PointerEvent, PointerState, WheelDeltaMode, WheelEvent,
};
use crate::node::{DebugLayout, NodeTree, PaintOptions, RepaintRequest};
use crate::style::{Border, Brush, ColorRef, Shape, StateFilter, StyleCollection};
use crate::{
    style, BoxConstraints, BoxedWidget, Environment, Measurements, Point, Rect, Size, Visual,
    Widget, WidgetExt
};
use anyhow::Result;
use config::FileFormat;
use futures::channel::mpsc::unbounded;
use futures::channel::mpsc::Sender;
use futures::channel::mpsc::{Receiver, UnboundedReceiver};
use futures::executor::LocalPool;
use futures::executor::LocalSpawner;
use futures::task::LocalSpawnExt;
use futures::{SinkExt, StreamExt};
use generational_indextree::NodeId;
use kyute_shell::drawing::Color;
use kyute_shell::platform::Platform;
use kyute_shell::window::{PlatformWindow, WindowDrawContext};
use log::trace;
use log::warn;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::mem;
use std::rc::{Rc, Weak};
use std::time::Duration;
use std::time::Instant;
use winit::event::{VirtualKeyCode, WindowEvent};
use winit::event_loop::EventLoop;
use winit::event_loop::{ControlFlow, EventLoopProxy, EventLoopWindowTarget};
use winit::window::{WindowBuilder, WindowId};

/// Loads the application style.
fn load_application_style(cfg: &config::Config) -> Rc<StyleCollection> {
    let path = cfg
        .get_str("app_style")
        .expect("no `app_style` file specified in application settings (`Settings.toml`)");
    // load RON
    let f = File::open(&path).expect("failed to open the application style");
    let app_style: StyleCollection =
        ron::de::from_reader(f).expect("failed to load the application style");
    Rc::new(app_style)
}

/// Creates a default application style
fn write_default_application_style() {
    use style::{
        Angle, BlendMode, BorderPosition, Brush, ColorRef, GradientType, Length, Palette,
        PaletteIndex, Shape, State, Style, StyleSet,
    };

    const FRAME_BG_SUNKEN_COLOR: PaletteIndex = PaletteIndex(0);
    const FRAME_BG_NORMAL_COLOR: PaletteIndex = PaletteIndex(1);
    const FRAME_BG_RAISED_COLOR: PaletteIndex = PaletteIndex(2);
    const FRAME_FOCUS_COLOR: PaletteIndex = PaletteIndex(3);
    const FRAME_BORDER_COLOR: PaletteIndex = PaletteIndex(4);
    const BUTTON_BACKGROUND_TOP_COLOR: PaletteIndex = PaletteIndex(5);
    const BUTTON_BACKGROUND_BOTTOM_COLOR: PaletteIndex = PaletteIndex(6);
    const BUTTON_BACKGROUND_TOP_COLOR_HOVER: PaletteIndex = PaletteIndex(7);
    const BUTTON_BACKGROUND_BOTTOM_COLOR_HOVER: PaletteIndex = PaletteIndex(8);
    const BUTTON_BORDER_BOTTOM_COLOR: PaletteIndex = PaletteIndex(9);
    const BUTTON_BORDER_TOP_COLOR: PaletteIndex = PaletteIndex(10);
    const WIDGET_OUTER_GROOVE_BOTTOM_COLOR: PaletteIndex = PaletteIndex(11);
    const WIDGET_OUTER_GROOVE_TOP_COLOR: PaletteIndex = PaletteIndex(12);
    const FRAME_BG_SUNKEN_COLOR_HOVER: PaletteIndex = PaletteIndex(13);

    let palette = Palette {
        entries: vec![
            Color::new(0.227, 0.227, 0.227, 1.0), // FRAME_BG_SUNKEN_COLOR
            Color::new(0.326, 0.326, 0.326, 1.0), // FRAME_BG_NORMAL_COLOR
            Color::new(0.424, 0.424, 0.424, 1.0), // FRAME_BG_RAISED_COLOR
            Color::new(0.600, 0.600, 0.900, 1.0), // FRAME_FOCUS_COLOR
            Color::new(0.130, 0.130, 0.130, 1.0), // FRAME_BORDER_COLOR
            Color::new(0.450, 0.450, 0.450, 1.0), // BUTTON_BACKGROUND_TOP_COLOR
            Color::new(0.400, 0.400, 0.400, 1.0), // BUTTON_BACKGROUND_BOTTOM_COLOR
            Color::new(0.500, 0.500, 0.500, 1.0), // BUTTON_BACKGROUND_TOP_COLOR_HOVER
            Color::new(0.450, 0.450, 0.450, 1.0), // BUTTON_BACKGROUND_BOTTOM_COLOR_HOVER
            Color::new(0.100, 0.100, 0.100, 1.0), // BUTTON_BORDER_BOTTOM_COLOR
            Color::new(0.180, 0.180, 0.180, 1.0), // BUTTON_BORDER_TOP_COLOR
            Color::new(1.000, 1.000, 1.000, 0.2), // WIDGET_OUTER_GROOVE_BOTTOM_COLOR
            Color::new(1.000, 1.000, 1.000, 0.0), // WIDGET_OUTER_GROOVE_TOP_COLOR
            Color::new(0.180, 0.180, 0.180, 1.0), // FRAME_BG_SUNKEN_COLOR_HOVER
        ],
    };

    let button_style_set = style::StyleSet {
        shape: Shape::RoundedRect(style::Length::Dip(2.0)),
        styles: vec![
            style::Style {
                fill: Some(Brush::Gradient {
                    angle: Angle::degrees(90.0),
                    ty: GradientType::Linear,
                    stops: vec![
                        (0.0, ColorRef::Palette(BUTTON_BACKGROUND_BOTTOM_COLOR)),
                        (1.0, ColorRef::Palette(BUTTON_BACKGROUND_TOP_COLOR)),
                    ],
                    reverse: false,
                }),
                borders: vec![
                    Border {
                        opacity: 1.0,
                        blend_mode: BlendMode::Normal,
                        position: BorderPosition::Inside(Length::zero()),
                        width: Length::Dip(1.0),
                        brush: Brush::SolidColor(ColorRef::Palette(BUTTON_BORDER_BOTTOM_COLOR)),
                    },
                    Border {
                        opacity: 1.0,
                        blend_mode: BlendMode::Normal,
                        position: BorderPosition::Outside(Length::zero()),
                        width: Length::Dip(1.0),
                        brush: Brush::Gradient {
                            angle: Angle::degrees(90.0),
                            ty: GradientType::Linear,
                            stops: vec![
                                (0.0, ColorRef::Palette(WIDGET_OUTER_GROOVE_BOTTOM_COLOR)),
                                (0.3, ColorRef::Palette(WIDGET_OUTER_GROOVE_TOP_COLOR)),
                            ],
                            reverse: false,
                        },
                    },
                ],
                ..Style::default()
            },
            style::Style {
                state_filter: StateFilter {
                    value: State::HOVER,
                    mask: State::HOVER,
                },
                fill: Some(Brush::Gradient {
                    angle: Angle::degrees(90.0),
                    ty: GradientType::Linear,
                    stops: vec![
                        (0.0, ColorRef::Palette(BUTTON_BACKGROUND_BOTTOM_COLOR_HOVER)),
                        (1.0, ColorRef::Palette(BUTTON_BACKGROUND_TOP_COLOR_HOVER)),
                    ],
                    reverse: false,
                }),
                ..Style::default()
            },
        ],
    };

    let slider_knob = style::StyleSet {
        shape: Shape::Path("M 0.5 0.5 L 10.5 0.5 L 10.5 5.5 L 5.5 10.5 L 0.5 5.5 Z".to_string()),
        styles: vec![
            style::Style {
                fill: Some(Brush::Gradient {
                    angle: Angle::degrees(90.0),
                    ty: GradientType::Linear,
                    stops: vec![
                        (0.0, ColorRef::Palette(BUTTON_BACKGROUND_BOTTOM_COLOR)),
                        (1.0, ColorRef::Palette(BUTTON_BACKGROUND_TOP_COLOR)),
                    ],
                    reverse: false,
                }),
                borders: vec![Border {
                    opacity: 1.0,
                    blend_mode: BlendMode::Normal,
                    position: BorderPosition::Inside(Length::zero()),
                    width: Length::Dip(1.0),
                    brush: Brush::SolidColor(ColorRef::Palette(BUTTON_BORDER_BOTTOM_COLOR)),
                }],
                ..Style::default()
            },
            style::Style {
                state_filter: StateFilter {
                    value: State::HOVER,
                    mask: State::HOVER,
                },
                fill: Some(Brush::Gradient {
                    angle: Angle::degrees(90.0),
                    ty: GradientType::Linear,
                    stops: vec![
                        (0.0, ColorRef::Palette(BUTTON_BACKGROUND_BOTTOM_COLOR_HOVER)),
                        (1.0, ColorRef::Palette(BUTTON_BACKGROUND_TOP_COLOR_HOVER)),
                    ],
                    reverse: false,
                }),
                ..Style::default()
            },
        ],
    };

    let slider_track = style::StyleSet {
        shape: Shape::RoundedRect(style::Length::Dip(2.0)),
        styles: vec![style::Style {
            fill: Some(Brush::SolidColor(ColorRef::Palette(FRAME_BG_SUNKEN_COLOR))),
            borders: vec![
                Border {
                    opacity: 1.0,
                    blend_mode: BlendMode::Normal,
                    position: BorderPosition::Inside(Length::zero()),
                    width: Length::Dip(1.0),
                    brush: Brush::SolidColor(ColorRef::Palette(BUTTON_BORDER_BOTTOM_COLOR)),
                },
                Border {
                    opacity: 1.0,
                    blend_mode: BlendMode::Normal,
                    position: BorderPosition::Outside(Length::zero()),
                    width: Length::Dip(1.0),
                    brush: Brush::Gradient {
                        angle: Angle::degrees(90.0),
                        ty: GradientType::Linear,
                        stops: vec![
                            (0.0, ColorRef::Palette(WIDGET_OUTER_GROOVE_BOTTOM_COLOR)),
                            (0.3, ColorRef::Palette(WIDGET_OUTER_GROOVE_TOP_COLOR)),
                        ],
                        reverse: false,
                    },
                },
            ],
            ..Style::default()
        }],
    };

    let text_box_style_set = style::StyleSet {
        shape: Shape::Rect,
        styles: vec![
            style::Style {
                fill: Some(Brush::SolidColor(ColorRef::Palette(FRAME_BG_SUNKEN_COLOR))),
                borders: vec![
                    Border {
                        opacity: 1.0,
                        blend_mode: BlendMode::Normal,
                        position: BorderPosition::Inside(Length::zero()),
                        width: Length::Dip(1.0),
                        brush: Brush::SolidColor(ColorRef::Palette(BUTTON_BORDER_BOTTOM_COLOR)),
                    },
                    Border {
                        opacity: 1.0,
                        blend_mode: BlendMode::Normal,
                        position: BorderPosition::Outside(Length::zero()),
                        width: Length::Dip(1.0),
                        brush: Brush::Gradient {
                            angle: Angle::degrees(90.0),
                            ty: GradientType::Linear,
                            stops: vec![
                                (0.0, ColorRef::Palette(WIDGET_OUTER_GROOVE_BOTTOM_COLOR)),
                                (0.3, ColorRef::Palette(WIDGET_OUTER_GROOVE_TOP_COLOR)),
                            ],
                            reverse: false,
                        },
                    },
                ],
                ..Style::default()
            },
            style::Style {
                state_filter: StateFilter {
                    value: State::HOVER,
                    mask: State::HOVER,
                },
                fill: Some(Brush::SolidColor(ColorRef::Palette(FRAME_BG_SUNKEN_COLOR))),
                ..Style::default()
            },
        ],
    };

    let mut style_sets = HashMap::new();
    style_sets.insert("button".to_string(), button_style_set);
    style_sets.insert("text_box".to_string(), text_box_style_set);
    style_sets.insert("slider_knob".to_string(), slider_knob);
    style_sets.insert("slider_track".to_string(), slider_track);

    let style_collection = StyleCollection {
        style_sets,
        palettes: vec![palette],
    };

    // serialize
    let s = ron::ser::to_string_pretty(&style_collection, ron::ser::PrettyConfig::new())
        .expect("failed to export string");
    let mut file = File::create("default_style.ron").expect("could not open default_style.ron");
    file.write_all(s.as_bytes())
        .expect("could not write default style");
    log::info!("default style written to default_style.ron");
}

pub(crate) type NodeTreeHandle = Rc<RefCell<Option<NodeTree>>>;

/// Context needed to open a window.
pub struct WindowCtx<'a> {
    /// Platform services (text, drawing, etc.).
    pub(crate) platform: &'a Platform,
    /// Handle to the node tree passed to the async tasks spawned by components.
    pub(crate) tree_handle: NodeTreeHandle,
    /// Event loop.
    pub(crate) event_loop: &'a EventLoopWindowTarget<()>,
    /// The style collection of the application.
    pub(crate) style: Rc<StyleCollection>,
    /// Open windows, mapped to their corresponding node in the node tree.
    pub(crate) windows: &'a mut HashMap<WindowId, NodeId>,
    /// Spawner to spawn the tasks in charge of forwarding comments to the components.
    pub(crate) spawner: LocalSpawner,
}

pub fn run<W: Widget>(mut root_widget: impl FnMut() -> W + 'static) {
    // winit event loop
    let event_loop = EventLoop::new();
    // platform-specific, window-independent initialization
    let platform = unsafe { Platform::init() };

    // load application settings
    let mut cfg = config::Config::new();
    cfg.merge(config::File::with_name("Settings").format(config::FileFormat::Toml));
    write_default_application_style();
    let app_style = load_application_style(&cfg);

    let mut pool = LocalPool::new();
    let spawner = pool.spawner();
    //let winit_task = spawner.spawn_local(winit_event_handler(platform, app_style,event_loop.create_proxy(), events_rx));

    // WindowId -> NodeId
    let mut window_nodes = HashMap::new();
    let mut tree = NodeTree::new();
    // Handle to the tree used in futures. We don't wrap the tree itself with Rc<RefCell<>>
    // because that's supremely annoying in NodeTree::layout (borrow_mut and weird derefs everywhere).
    // Instead, we pass a Rc handle to a "cell": when polling the futures (run_until_stalled),
    // we temporarily move the tree into this cell, and move it back once all futures have been polled.
    let mut temp_tree = Rc::new(RefCell::new(None));

    // perform the initial layout
    // the root available space is infinite, technically, this can produce infinitely big visuals,
    // but this is never the case for visuals under windows (they are constrained by the size of the window).
    // Note that there's no window created by default. The user should create window widgets to
    // have something to render to.
    let mut win_ctx = WindowCtx {
        platform: &platform,
        tree_handle: temp_tree.clone(),
        event_loop: &event_loop,
        style: app_style.clone(),
        windows: &mut window_nodes,
        spawner
    };
    tree.layout(
        root_widget().boxed(),
        Size::zero(),
        &BoxConstraints::new(..,..),
        Environment::new(),
        &mut win_ctx,
    );
    let spawner = win_ctx.spawner;

    // run event loop
    event_loop.run(move |event, elwt, control_flow| {
        *control_flow = ControlFlow::Wait;

        let mut win_ctx = WindowCtx {
            platform: &platform,
            event_loop: &elwt,
            style: app_style.clone(),
            windows: &mut window_nodes,
            spawner: spawner.clone(),
            tree_handle: temp_tree.clone(),
        };

        // helper to extract a visual from the node tree
        fn replace_visual(
            tree: &mut NodeTree,
            node_id: NodeId,
            with: Option<Box<dyn Visual>>,
        ) -> Option<Box<dyn Visual>> {
            tree.arena
                .get_mut(node_id)
                .and_then(|n| mem::replace(&mut n.get_mut().visual, with))
        }

        // fwd to window components
        match event {
            winit::event::Event::WindowEvent { window_id, event } => {
                // deliver event to the target window in the node tree
                if let Some(node_id) = win_ctx.windows.get(&window_id).cloned() {
                    // see RedrawRequested for more comments
                    let mut v = replace_visual(&mut tree, node_id, None);
                    v.as_mut()
                        .map(|v| v.window_event(&mut win_ctx, &event, &mut tree, node_id));
                    replace_visual(&mut tree, node_id, v);
                }

                if let WindowEvent::Resized(size) = event {
                    // A window has been resized.
                    // Note that, currently, we have no way of relayouting each window separately
                    // (since Application::view() returns the widget tree for all windows).
                    // For now, just relayout everything.
                    tree.layout(
                        root_widget().boxed(),
                        Size::zero(),
                        &BoxConstraints::new(..,..),
                        Environment::new(),
                        &mut win_ctx,
                    );
                }
            }

            winit::event::Event::RedrawRequested(window_id) => {
                // A window needs to be repainted
                if let Some(node_id) = win_ctx.windows.get(&window_id).cloned() {
                    // sleight-of-hand: extract the visual from the tree, then put it back
                    // so that we can give a mut ref to the tree to Visual::window_paint
                    let mut v = replace_visual(&mut tree, node_id, None);
                    // now call the `window_paint` procedure on the visual, if it exists
                    v.as_mut()
                        .map(|v| v.window_paint(&mut win_ctx, &mut tree, node_id));
                    // put back the visual
                    replace_visual(&mut tree, node_id, v);
                } else {
                    log::warn!("repaint for unregistered window")
                }
            }
            _ => (),
        }

        // after having processed the event, poll all pending futures.
        // the "replace_with" trick is because we need to move the tree out and place it into the
        // shared cell, but we don't want to create a new tree
        replace_with::replace_with_or_abort(&mut tree, |tree| {
            // move the node tree into the temp cell visible to the tasks
            temp_tree.borrow_mut().replace(tree);
            // poll futures
            pool.run_until_stalled();
            // put the tree back into place
            temp_tree.borrow_mut().take().unwrap()
        })
    })
}
