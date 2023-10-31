use kyute2::{
    drawing, make_uniform_data, shader, shader_paint,
    text::{TextSpan, TextStyle},
    theme::palette,
    widget::{grid::GridArea, Background, Frame, Grid, Null, Text, WidgetExt},
    Alignment, AppCtx, AppLauncher, AppWindowBuilder, Color, Environment, Stateful, UnitExt, Widget,
};
use kyute2_macros::grid_template;
use skia_safe as sk;
use std::{
    any::Any,
    cell::OnceCell,
    sync::Arc,
    time::{Duration, Instant},
};
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, Layer};

////////////////////////////////////////////////////////////////////////////////////////////////////

grid_template!(GRID: [START] 100px 1fr 1fr [END] / [TOP] 50px [BOTTOM] );

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
        Text::new(text),
    );

    /*let swatch = color_swatch().clickable();
    if swatch.clicked() {
        info!("swatch clicked");
    }*/

    let swatch = Stateful::new(
        || false,
        |cx| {
            Frame::new(50.0.into(), 50.0.into(), color_swatch())
                .clickable()
                .on_clicked(|cx| {
                    let state: &mut bool = cx.state_mut().unwrap();
                    eprintln!("swatch clicked: {state}");
                    *state = !*state;
                })
        },
    );

    grid.add(
        GridArea {
            row: Some(0),
            column: Some(1),
            row_span: 1,
            column_span: 1,
        },
        Alignment::START,
        Alignment::START,
        swatch,
    );

    Frame::new(100.percent(), 100.percent(), grid)
}

/// This function is run whenever the UI of a window needs to be rebuilt,
/// or the application receives a message that it is interested in.
fn application(window_ctx: &mut WindowCtx) {
    // build or rebuild the main window
    let contents = main_window_contents();
    let window_handle = AppWindowBuilder::new(contents)
        .title("Main window")
        .build(app_ctx, &Environment::default());
    if window_handle.close_requested() {
        app_ctx.quit();
    }
}

fn main() {
    tracing_subscriber::fmt()
        .compact()
        .with_target(false)
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    AppLauncher::new(application).run();
}
