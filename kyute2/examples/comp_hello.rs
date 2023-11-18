use std::{cell::OnceCell, sync::Arc};

use kurbo::Insets;
use tracing_subscriber::layer::SubscriberExt;
use winit::window::WindowBuilder;

use kyute2::{
    shader_paint,
    text::{TextSpan, TextStyle},
    theme::palette,
    widget::{
        button, grid::GridArea, Background, BorderStyle, Frame, Grid, RoundedRectBorder, ShapeDecoration, Text,
        WidgetExt,
    },
    window::PopupOptions,
    Alignment, AppLauncher, AppWindowHandle, PopupWindow, Size, Stateful, TreeCtx, UnitExt, Widget,
};
use kyute2_macros::grid_template;

////////////////////////////////////////////////////////////////////////////////////////////////////

grid_template!(GRID: [START] 200px 1fr 1fr [END] / [TOP] 100px [BOTTOM] );

fn color_swatch() -> impl Widget {
    let paint = shader_paint! {
        r#"
        layout(color) uniform float4 color;
        layout(color) uniform float3 cbColor;
        uniform int cbSize;
        
        float3 checkerboard(float2 fragcoord) {
            float2 p = floor(fragcoord / float(cbSize));
            return mix(float3(1.0), cbColor, mod(p.x + p.y, 2.0));
        }
        
        float4 main(float2 fragcoord) {
            float4 final = color;
            if (cbSize > 0) {
                final.rgb = mix(checkerboard(fragcoord), final.rgb, final.a);
                final.a = 1.0;
            }
            return final;
        }
        "#,
        color: [f32; 4] = [1.0, 0.0, 0.0, 0.5],
        cbColor: [f32; 3] = [0.5, 0.5, 0.5],
        cbSize: i32 = 5
    };
    Background::new(paint)
}

fn main_window_contents() -> impl Widget {
    let text_style = Arc::new(
        TextStyle::new()
            .font_size(20.0)
            .font_family("Courier New")
            .color(palette::PINK_200),
    );
    let text = TextSpan::new("Hello, world!", text_style);

    let mut grid = Grid::from_template(&GRID);
    grid.add(
        GridArea {
            row: Some(0),
            column: Some(0),
            row_span: 1,
            column_span: 1,
        },
        Alignment::START,
        Alignment::START,
        Text::new(text).decorate(
            ShapeDecoration::new()
                .border(RoundedRectBorder {
                    color: palette::PURPLE_900,
                    radius: 20.0,
                    dimensions: Insets::uniform(10.0),
                    style: BorderStyle::Solid,
                })
                .border(RoundedRectBorder {
                    color: palette::PURPLE_400,
                    radius: 20.0,
                    dimensions: Insets::uniform(10.0),
                    style: BorderStyle::Solid,
                })
                .border(RoundedRectBorder {
                    color: palette::RED_200,
                    radius: 20.0,
                    dimensions: Insets::uniform(10.0),
                    style: BorderStyle::Solid,
                }),
        ),
    );

    /*let swatch = color_swatch().clickable();
    if swatch.clicked() {
        info!("swatch clicked");
    }*/

    // Shape + shape modifiers?

    let swatch = Stateful::new(
        || false,
        |cx, state| {
            Frame::new(50.0.into(), 50.0.into(), color_swatch())
                .clickable()
                .on_clicked(move |cx| {
                    let state = &mut cx[state];
                    eprintln!("swatch clicked: {state}");
                    *state = !*state;
                })
        },
    );

    // Issue: opening a child window requires a TreeCtx, which, if the function doesn't get one,
    // requires creating a dummy widget so that its `build` method is called. In turn, this requires
    // a dummy element because all widgets produce elements.
    //
    // It would be better if we could open child windows without needing an "anchor" into the UI tree
    // of the parent window (because it's completely useless).

    //let menu = Stateful::new();

    #[derive(Default)]
    struct MenuState {
        open: bool,
    }

    let menu_button = Stateful::new(
        || MenuState::default(),
        |cx, state| {
            let b = button("Click me").on_clicked(move |cx| {
                let state = &mut cx[state];
                eprintln!("swatch clicked: {}", state.open);
                state.open = !state.open;
            });

            let window_widget = PopupWindow {
                content: move |cx: &mut TreeCtx| {
                    /*let text_style = Arc::new(
                        TextStyle::new()
                            .font_size(20.0)
                            .font_family("Courier New")
                            .color(palette::PINK_200),
                    );
                    let text = TextSpan::new("It's a menu!", text_style);
                    Text::new(text)*/
                    button("close it").on_clicked(move |cx| {
                        cx[state].open = false;
                    })
                },
                options: PopupOptions {
                    opened: cx[state].open,
                    size: Some(Size::new(200., 200.)),
                    position: None,
                },
            };

            b.overlay(window_widget)
        },
    );

    grid.add(
        GridArea {
            row: Some(0),
            column: Some(1),
            row_span: 1,
            column_span: 1,
        },
        Alignment::CENTER,
        Alignment::CENTER,
        menu_button,
    );

    Frame::new(100.percent(), 100.percent(), grid)
}

struct Application {
    main_window_handle: AppWindowHandle,
}

impl Application {
    fn update(&mut self, ctx: &mut TreeCtx) {
        // build or rebuild the main window contents
        self.main_window_handle.update(ctx, main_window_contents());
        if self.main_window_handle.close_requested() {
            ctx.quit();
        }
    }
}

fn main() {
    tracing_subscriber::fmt()
        .compact()
        .with_target(false)
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    /*use tracing_subscriber::layer::SubscriberExt;
    tracing::subscriber::set_global_default(tracing_subscriber::registry().with(tracing_tracy::TracyLayer::new()))
        .expect("set up the subscriber");*/

    let mut launcher = AppLauncher::new();
    let main_window_handle = launcher.with_ctx(|ctx| AppWindowHandle::new(ctx, "Hello", main_window_contents()));
    let mut app = Application { main_window_handle };
    launcher.run(move |ctx| app.update(ctx));
}
