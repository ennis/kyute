use std::{cell::OnceCell, sync::Arc};

use kurbo::Insets;
use tracing_subscriber::layer::SubscriberExt;

use kyute2::{
    shader_paint,
    text::{TextSpan, TextStyle},
    theme::palette,
    widget::{
        button,
        grid::{FlowDirection, GridArea, GridItem, GridItemAlignment, GridOptions, TableGridStyle, TrackSize},
        Background, BorderStyle, Frame, Grid, RoundedRectBorder, ShapeDecoration, Text, WidgetExt,
    },
    window::{Anchor, PopupOptions, PopupPosition},
    Alignment, AppLauncher, AppWindowHandle, ChangeFlags, Point, PopupTarget, Size, Stateful, TreeCtx, UnitExt, Widget,
};

//use kyute2_macros::grid_template;

////////////////////////////////////////////////////////////////////////////////////////////////////

//grid_template!(GRID: [START] 200px 1fr 1fr [END] / [TOP] 100px [BOTTOM] );

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

    /*let swatch = color_swatch().clickable();
    if swatch.clicked() {
        info!("swatch clicked");
    }*/

    //rich_format!("Hello, {0}!")

    #[derive(Default)]
    struct MenuState {
        open: bool,
    }

    let menu_button = Stateful::new(
        || MenuState::default(),
        |cx, state| {
            PopupTarget {
                content: button("Click me").on_clicked(move |cx| {
                    //popup.open()
                    let state = &mut cx[state];
                    eprintln!("swatch clicked: {}", state.open);
                    state.open = !state.open;

                    //PopupTarget::open_popup(cx)
                }),
                popup_content: move |cx: &mut TreeCtx| {
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
                on_dismiss: move |cx: &mut TreeCtx| {
                    cx[state].open = false;
                },
                options: PopupOptions {
                    opened: cx[state].open,
                    size: Some(Size::new(200., 200.)),
                    position: Some(PopupPosition {
                        parent_anchor: Anchor::Relative(Point::new(1.0, 0.5)),
                        popup_anchor: Anchor::Relative(Point::new(0.0, 0.5)),
                    }),
                },
            }
        },
    );

    let grid = Grid {
        options: GridOptions {
            flow: FlowDirection::Row,
            columns: vec![TrackSize::fixed(200.0), TrackSize::flex(1.0), TrackSize::flex(1.0)].into(),
            rows: vec![TrackSize::fixed(100.0)].into(),
            row_gap: 1.0,
            column_gap: 1.0,
            ..Default::default()
        },
        style: TableGridStyle,
        items: vec![
            GridItem {
                area: Default::default(),
                alignment: Default::default(),
                content: Box::new(
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
                ),
            },
            GridItem {
                area: Default::default(),
                alignment: GridItemAlignment::CENTER,
                content: Box::new(menu_button),
            },
        ],
    };

    Frame::new(100.percent(), 100.percent(), grid)
}

struct Application {
    main_window_handle: AppWindowHandle,
}

impl Application {
    fn update(&mut self, ctx: &mut TreeCtx) -> ChangeFlags {
        // build or rebuild the main window contents
        let change_flags = self.main_window_handle.update(ctx, main_window_contents());
        if self.main_window_handle.close_requested() {
            ctx.quit();
        }
        change_flags
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
