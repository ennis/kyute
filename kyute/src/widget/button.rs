use crate::{
    event::PointerEventKind,
    layout::Alignment,
    style,
    style::VisualState,
    widget::{prelude::*, Label, WidgetExt},
    Signal, UnitExt,
};
use std::cell::Cell;

////////////////////////////////////////////////////////////////////////////////////////////////////
// Widget definition
////////////////////////////////////////////////////////////////////////////////////////////////////

type ButtonInner = impl Widget;

pub struct Button {
    id: WidgetId,
    inner: ButtonInner,
    clicked: Signal<()>,
    active: Cell<bool>,
}

fn button_inner(label: String) -> ButtonInner {
    Label::new(label)
        // TODO: if the padding cannot be honored because of size constraints, should this be adjusted automatically?
        .padding(5.dip(), 5.dip(), 5.dip(), 5.dip())
        .min_height(21.dip())
        .min_width(80.dip())
        .horizontal_alignment(Alignment::CENTER)
        .background("rgb(4 4 255)", style::Shape::rectangle())
}

impl Button {
    /// Creates a new button with the specified label.
    #[composable]
    pub fn new(label: impl Into<String>) -> Button {
        Button {
            id: WidgetId::here(),
            inner: button_inner(label.into()),
            clicked: Signal::new(),
            active: Cell::new(false),
        }
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

    fn layout(&self, ctx: &mut LayoutCtx, constraints: &LayoutConstraints, env: &Environment) -> Layout {
        self.inner.layout(ctx, constraints, env)
    }

    fn event(&self, ctx: &mut EventCtx, event: &mut Event, env: &Environment) {
        if let Event::Pointer(p) = event {
            match p.kind {
                PointerEventKind::PointerDown => {
                    ctx.request_focus();
                    ctx.set_handled();
                    ctx.capture_pointer();
                    self.active.set(true);
                }
                PointerEventKind::PointerUp => {
                    self.active.set(false);
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
        if self.active.get() {
            ctx.with_visual_state(VisualState::ACTIVE, |ctx| {
                self.inner.paint(ctx);
            })
        } else {
            self.inner.paint(ctx)
        }
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
