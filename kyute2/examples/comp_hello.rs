use kurbo::Insets;
use kyute2::{
    drawing,
    drawing::BoxShadow,
    make_uniform_data, shader, shader_paint,
    text::{TextSpan, TextStyle},
    theme,
    theme::palette,
    widget::{
        button, grid::GridArea, Background, BorderStyle, Frame, Grid, Null, RoundedRectBorder, ShapeDecoration, Text,
        WidgetExt,
    },
    Alignment, AppCtx, AppLauncher, AppWindowHandle, Color, Stateful, UnitExt, Widget,
};
use kyute2_macros::grid_template;
use skia_safe as sk;
use std::{
    any::Any,
    borrow::Cow,
    cell::OnceCell,
    sync::Arc,
    time::{Duration, Instant},
};
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, Layer};
use winit::window::WindowBuilder;

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

    let button = button("Click me").on_clicked(move |cx| {
        eprintln!("You did it!");
    });

    // issue: by threading the state implicitly through TreeCtx
    // there's no way to restrain the amount of state accessible by child widgets
    // and thus the amount of state they depend on.
    // -> we cannot assume that a widget is dependent on only a subset of the state in the stack.
    //    and the widget must be considered dirty if any item in the stack has changed.
    //
    // Solutions?
    // - pass the state explicitly: Widget becomes Widget<State>, `build(&mut self, state: &mut State) -> Self::Element` method passes the state down.
    //     - and now we've reinvented xilem (which isn't a bad thing in itself)
    //     - ... and also https://github.com/audulus/rui apparently
    //      They have a "ViewId" which is basically our ElementId
    //      Interestingly, they have only one trait "View".
    // - Inspired by https://github.com/audulus/rui =>

    grid.add(
        GridArea {
            row: Some(0),
            column: Some(1),
            row_span: 1,
            column_span: 1,
        },
        Alignment::CENTER,
        Alignment::CENTER,
        button,
    );

    Frame::new(100.percent(), 100.percent(), grid)
}

struct Application {
    main_window_handle: AppWindowHandle,
}

impl Application {
    fn update(&mut self, app_ctx: &mut AppCtx) {
        // build or rebuild the main window contents
        self.main_window_handle.update(app_ctx, main_window_contents());
        if self.main_window_handle.close_requested() {
            app_ctx.quit();
        }
    }
}

fn main() {
    tracing_subscriber::fmt()
        .compact()
        .with_target(false)
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let mut launcher = AppLauncher::new();
    let main_window_handle =
        launcher.with_app_ctx(|ctx| AppWindowHandle::new(ctx, WindowBuilder::new().with_title("Hello")));
    let mut app = Application { main_window_handle };
    launcher.run(move |ctx| app.update(ctx));
}
