use crate::{
    cache,
    core::WindowPaintCtx,
    event::PointerEventKind,
    style::{BoxStyle, VisualState},
    theme,
    widget::{prelude::*, Container, Grid, Label},
    Color, GpuFrameCtx, Signal, State, UnitExt, ValueRef,
};

////////////////////////////////////////////////////////////////////////////////////////////////////
// Widget definition
////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone)]
pub struct Button {
    id: WidgetId,
    inner: Container<Label>,
    clicked: Signal<()>,
}

impl Button {
    /// Creates a new button with the specified label.
    #[composable]
    pub fn new(label: String) -> Button {
        Button {
            id: WidgetId::here(),
            inner: Container::new(Label::new(label))
                .min_height(21.dip())
                .content_padding(5.dip(), 5.dip(), 5.dip(), 5.dip())
                .baseline(17.dip())
                .box_style(theme::BUTTON)
                .alternate_box_style(VisualState::ACTIVE | VisualState::HOVER, theme::BUTTON_ACTIVE)
                .alternate_box_style(VisualState::HOVER, theme::BUTTON_HOVER),
            clicked: Signal::new(),
        }
    }

    /// Sets the style of this button.
    pub fn box_style(mut self, style: impl Into<BoxStyle>) -> Button {
        self.set_box_style(style);
        self
    }

    /// Sets the style of this button.
    pub fn set_box_style(&mut self, style: impl Into<BoxStyle>) {
        self.inner.set_box_style(style.into());
    }

    /// Sets the text color of this button.
    pub fn text_color(mut self, color: Color) -> Button {
        self.set_text_color(color);
        self
    }

    /// Sets the text color of this button.
    pub fn set_text_color(&mut self, color: Color) {
        self.inner.inner_mut().set_color(color);
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

////////////////////////////////////////////////////////////////////////////////////////////////////
// impl Widget
////////////////////////////////////////////////////////////////////////////////////////////////////

impl Widget for Button {
    fn widget_id(&self) -> Option<WidgetId> {
        Some(self.id)
    }

    fn layout(&self, ctx: &mut LayoutCtx, constraints: BoxConstraints, env: &Environment) -> Measurements {
        self.inner.layout(ctx, constraints, env)
    }

    fn event(&self, ctx: &mut EventCtx, event: &mut Event, env: &Environment) {
        if let Event::Pointer(p) = event {
            match p.kind {
                PointerEventKind::PointerDown => {
                    ctx.request_focus();
                    ctx.set_handled();
                    ctx.capture_pointer();
                    //self.active.set(true);
                }
                PointerEventKind::PointerUp => {
                    //self.active.set(false);
                    self.clicked.signal(());
                }
                PointerEventKind::PointerOver => {
                    //trace!("button PointerOver");
                }
                PointerEventKind::PointerOut => {
                    //trace!("button PointerOut");
                }
                _ => {}
            }
        }

        if !ctx.handled() {
            self.inner.route_event(ctx, event, env)
        }
    }

    fn paint(&self, ctx: &mut PaintCtx) {
        self.inner.paint(ctx)
    }
}

/*
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
*/
