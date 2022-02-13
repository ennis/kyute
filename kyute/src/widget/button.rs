use crate::{
    align_boxes, composable,
    core2::{EventCtx, LayoutCtx, PaintCtx},
    event::PointerEventKind,
    state::Signal,
    style::{Border, BoxStyle, ColorRef, UnitExt, ValueRef},
    theme,
    widget::{Container, Label},
    Alignment, BoxConstraints, EnvKey, Environment, Event, Measurements, Rect, SideOffsets, Size,
    Widget, WidgetPod,
};
use tracing::trace;

/// Button styling information.
#[derive(Clone, Debug, Default)]
pub struct ButtonStyle {
    pub box_style: BoxStyle,
    pub label_color: ColorRef,
}

impl_env_value!(ButtonStyle);

#[derive(Clone)]
pub struct Button {
    inner: WidgetPod<Container<Label>>,
    clicked: Signal<()>,
}

impl Button {
    /// Creates a new button with the specified label.
    #[composable]
    pub fn new(label: String) -> Button {
        Button {
            inner: WidgetPod::new(
                Container::new(Label::new(label))
                    .min_height(theme::BUTTON_HEIGHT)
                    .content_padding(SideOffsets::new_all_same(5.0))
                    .baseline(theme::BUTTON_LABEL_BASELINE)
                    .box_style(theme::BUTTON),
            ),
            clicked: Signal::new(),
        }
    }

    /*/// Sets the style of this button.
    pub fn style(mut self, style: impl Into<ValueRef<ButtonStyle>>) -> Button {
        self.style = style.into();
        self
    }*/

    /// Returns whether this button has been clicked.
    pub fn clicked(&self) -> bool {
        self.clicked.signalled()
    }
}

impl Widget for Button {
    fn debug_name(&self) -> &str {
        std::any::type_name::<Self>()
    }

    fn event(&self, ctx: &mut EventCtx, event: &mut Event, env: &Environment) {
        match event {
            Event::Pointer(p) => match p.kind {
                PointerEventKind::PointerDown => {
                    //trace!("button clicked");
                    self.clicked.signal(ctx, ());
                    ctx.request_focus();
                    ctx.request_redraw();
                    ctx.set_handled();
                }
                PointerEventKind::PointerOver => {
                    //trace!("button PointerOver");
                    ctx.request_redraw();
                }
                PointerEventKind::PointerOut => {
                    //trace!("button PointerOut");
                    ctx.request_redraw();
                }
                _ => {}
            },
            _ => {}
        }

        if !ctx.handled() {
            self.inner.event(ctx, event, env)
        }
    }

    fn layout(
        &self,
        ctx: &mut LayoutCtx,
        constraints: BoxConstraints,
        env: &Environment,
    ) -> Measurements {
        let m = self.inner.layout(ctx, constraints, env);
        m
    }

    fn paint(&self, ctx: &mut PaintCtx, bounds: Rect, env: &Environment) {
        self.inner.paint(ctx, bounds, env)
    }
}

/*let background_gradient = LinearGradient::new()
.angle(90.0.degrees())
.stop(BUTTON_BACKGROUND_BOTTOM_COLOR, 0.0)
.stop(BUTTON_BACKGROUND_TOP_COLOR, 1.0);*/

/*ctx.draw_styled_box(
    bounds,
    &BoxStyle::new()
        .fill(background_gradient.clone())
        .border(
            Border::new(1.dip())
                .inside(0.dip())
                .paint(BUTTON_BORDER_BOTTOM_COLOR)
                .opacity(1.0),
        )
        .border(
            Border::new(1.dip()).outside(0.dip()).paint(
                LinearGradient::new()
                    .angle(90.0.degrees())
                    .stop(WIDGET_OUTER_GROOVE_BOTTOM_COLOR, 0.0)
                    .stop(WIDGET_OUTER_GROOVE_TOP_COLOR, 0.3),
            ),
        ),
    env,
);*/
