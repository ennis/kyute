use crate::{CompositionCtx, WindowWidget};
use kyute_shell::{window::PlatformWindow, winit::window::WindowBuilder};
use std::sync::Arc;

pub fn window(cx: &mut CompositionCtx, title: &str, contents: impl FnMut(&mut CompositionCtx)) {
    cx.emit_node(
        |cx| {
            let builder = WindowBuilder::new().with_title(title);
            let window = PlatformWindow::new(cx.event_loop(), builder, None)
                .expect("failed to create window");
            cx.set_window(window);
            WindowWidget::new()
        },
        |cx, w| {
            if w.title() != title {
                w.set_title(title);
            }
        },
        contents,
    );
}

/*
pub fn window2(cx: &mut Cx,
              title: &str,
              contents: impl FnOnce(&mut Cx))
{
    ctx.emit_node(
        |ctx| {
            let window_builder = WindowBuilder::new().with_title(title);
            let window = PlatformWindow::new(ctx.event_loop(), window_builder, None)
                .expect("failed to create window");
            WindowWidget::from_platform_window(window)
        },
        |ctx, window_widget| {
            if window_widget.title() != title {
                window_widget.set_title(title);
            }
        },
        contents
    );
}
*/
