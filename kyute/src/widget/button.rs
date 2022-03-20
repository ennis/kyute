use crate::{
    cache,
    event::PointerEventKind,
    style::{BoxStyle, VisualState},
    theme,
    widget::{prelude::*, Container, Grid, Label},
    Color, Signal, ValueRef,
};
use kyute_common::UnitExt;

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
            // TODO: ValueRef for container
            inner: Container::new(Label::new(label))
                .min_height(21.dip())
                .content_padding(5.dip(), 5.dip(), 5.dip(), 5.dip())
                .baseline(17.dip())
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
        if let Event::Pointer(p) = event {
            match p.kind {
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
            }
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

pub struct ButtonBox {
    grid: Grid,
}

impl ButtonBox {
    #[composable]
    pub fn new<I>(buttons: I)
    where
        I: IntoIterator<Item = Button>,
        I::IntoIter: ExactSizeIterator,
    {
        let iter = buttons.into_iter();
        let n = iter.len();
        for (i, mut b) in iter.enumerate() {
            match i {
                0 => {
                    b.set_box_style(theme::BOX_BUTTON_LEADING);
                }
                x if x == n => {
                    b.set_box_style(theme::BOX_BUTTON_INNER);
                }
                _ => {
                    b.set_box_style(theme::BOX_BUTTON_TRAILING);
                }
            }
        }
    }
}
