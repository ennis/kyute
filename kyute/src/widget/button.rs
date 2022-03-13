use crate::{
    cache,
    event::PointerEventKind,
    style::{BoxStyle, VisualState},
    theme,
    widget::{prelude::*, Container, Label},
    Color, SideOffsets, Signal, ValueRef,
};

#[derive(Clone)]
pub struct Button {
    id: WidgetId,
    inner: Container<Label>,
    clicked: Signal<()>,
    // FIXME: I just want for the flag value to be retained across recomps; design something simpler
    active: (bool, cache::State<bool>),
}

impl Button {
    /// Creates a new button with the specified label.
    #[composable]
    pub fn new(label: String) -> Button {
        let active = cache::state(|| false);
        Button {
            id: WidgetId::here(),
            inner: Container::new(Label::new(label))
                .min_height(theme::BUTTON_HEIGHT)
                .content_padding(SideOffsets::new_all_same(5.0))
                .baseline(theme::BUTTON_LABEL_BASELINE)
                .box_style(theme::BUTTON)
                .alternate_box_style(VisualState::ACTIVE | VisualState::HOVER, theme::BUTTON_ACTIVE)
                .alternate_box_style(VisualState::HOVER, theme::BUTTON_HOVER),
            clicked: Signal::new(),
            active: (active.get(), active),
        }
    }

    /// Sets the style of this button.
    pub fn box_style(mut self, style: impl Into<ValueRef<BoxStyle>>) -> Button {
        self.set_box_style(style);
        self
    }

    /// Sets the style of this button.
    pub fn set_box_style(&mut self, style: impl Into<ValueRef<BoxStyle>>) {
        self.inner.set_box_style(style.into());
    }

    /// Sets the text color of this button.
    pub fn text_color(mut self, color: Color) -> Button {
        self.set_text_color(color);
        self
    }

    /// Sets the text color of this button.
    pub fn set_text_color(&mut self, color: Color) {
        self.inner.contents_mut().set_color(color);
    }

    /// Returns whether this button has been clicked.
    pub fn clicked(&self) -> bool {
        self.clicked.signalled()
    }

    /// Runs the function when the button has been clicked.
    pub fn on_clicked(self, f: impl FnOnce()) -> Self {
        if self.clicked.signalled() {
            f()
        }
        self
    }
}

impl Widget for Button {
    fn widget_id(&self) -> Option<WidgetId> {
        Some(self.id)
    }

    fn debug_name(&self) -> &str {
        std::any::type_name::<Self>()
    }

    fn event(&self, ctx: &mut EventCtx, event: &mut Event, env: &Environment) {
        match event {
            Event::Pointer(p) => match p.kind {
                PointerEventKind::PointerDown => {
                    ctx.request_focus();
                    ctx.request_redraw();
                    ctx.set_handled();
                    ctx.capture_pointer();
                    self.active.1.set(true);
                }
                PointerEventKind::PointerUp => {
                    ctx.request_redraw();
                    self.active.1.set(false);
                    self.clicked.signal(());
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

    fn layout(&self, ctx: &mut LayoutCtx, constraints: BoxConstraints, env: &Environment) -> Measurements {
        self.inner.layout(ctx, constraints, env)
    }

    fn paint(&self, ctx: &mut PaintCtx, env: &Environment) {
        if self.active.0 {
            ctx.active = true;
        }
        self.inner.paint(ctx, env)
    }
}
