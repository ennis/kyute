//! Clickable widget wrapper
use crate::{event::PointerEventKind, widget::prelude::*, Signal};
use keyboard_types::{Key, KeyState, Modifiers};

/// Wraps an inner widget and allows the user to respond to clicks on it.
#[derive(Clone)]
pub struct Clickable<Inner> {
    id: WidgetId,
    inner: Inner,
    clicked: Signal<()>,
    active: Signal<bool>,
    hovered: Signal<bool>,
    focused: Signal<bool>,
}

impl<Inner: Widget + 'static> Clickable<Inner> {
    #[composable]
    pub fn new(inner: Inner) -> Clickable<Inner> {
        Clickable {
            id: WidgetId::here(),
            inner,
            clicked: Signal::new(),
            active: Signal::new(),
            hovered: Signal::new(),
            focused: Signal::new(),
        }
    }

    #[cfg_attr(debug_assertions, track_caller)]
    #[must_use]
    pub fn on_click(self, f: impl FnOnce()) -> Self {
        if self.clicked.signalled() {
            f();
        }
        self
    }

    /// Returns whether this button has been clicked.
    pub fn clicked(&self) -> bool {
        self.clicked.signalled()
    }

    /// Returns whether this button is active (holding the mouse button over it).
    pub fn activated(&self) -> bool {
        self.active.value() == Some(true)
    }

    /// Returns whether this button is active (holding the mouse button over it).
    pub fn deactivated(&self) -> bool {
        self.active.value() == Some(false)
    }

    #[must_use]
    pub fn on_activated(self, f: impl FnOnce()) -> Self {
        if self.activated() {
            f();
        }
        self
    }

    #[must_use]
    pub fn on_deactivated(self, f: impl FnOnce()) -> Self {
        if self.deactivated() {
            f();
        }
        self
    }

    /// Returns whether the pointer entered the clickable area.
    pub fn pointer_entered(&self) -> bool {
        self.hovered.value() == Some(true)
    }

    /// Returns whether the pointer exited the clickable area.
    pub fn pointer_exited(&self) -> bool {
        self.hovered.value() == Some(false)
    }

    #[must_use]
    pub fn on_pointer_entered(self, f: impl FnOnce()) -> Self {
        if self.pointer_entered() {
            f();
        }
        self
    }

    #[must_use]
    pub fn on_pointer_exited(self, f: impl FnOnce()) -> Self {
        if self.pointer_exited() {
            f();
        }
        self
    }

    pub fn focus_gained(&self) -> bool {
        self.focused.value() == Some(true)
    }

    pub fn focus_lost(&self) -> bool {
        self.focused.value() == Some(false)
    }

    #[must_use]
    pub fn on_focus_changed(self, f: impl FnOnce(bool)) -> Self {
        if let Some(focus) = self.focused.value() {
            f(focus);
        }
        self
    }

    #[must_use]
    pub fn on_focus_gained(self, f: impl FnOnce()) -> Self {
        if self.focus_gained() {
            f()
        }
        self
    }

    #[must_use]
    pub fn on_focus_lost(self, f: impl FnOnce()) -> Self {
        if self.focus_lost() {
            f()
        }
        self
    }

    /// Returns a reference to the inner widget.
    pub fn inner(&self) -> &Inner {
        &self.inner
    }

    /// Returns a mutable reference to the inner widget.
    pub fn inner_mut(&mut self) -> &mut Inner {
        &mut self.inner
    }
}

impl<Inner: Widget + 'static> Widget for Clickable<Inner> {
    fn widget_id(&self) -> Option<WidgetId> {
        Some(self.id)
    }

    fn layout(&self, ctx: &mut LayoutCtx, constraints: &LayoutConstraints, env: &Environment) -> Layout {
        self.inner.layout(ctx, constraints, env)
    }

    fn event(&self, ctx: &mut EventCtx, event: &mut Event, env: &Environment) {
        match event {
            Event::BuildFocusChain { chain } => {
                // clickable items are by default focusable
                chain.push(self.id);
            }
            Event::Pointer(p) => match p.kind {
                PointerEventKind::PointerDown => {
                    ctx.request_focus();
                    ctx.set_handled();
                    ctx.capture_pointer();
                    self.active.signal(true);
                }
                PointerEventKind::PointerUp => {
                    self.active.signal(false);
                    self.clicked.signal(());
                }
                PointerEventKind::PointerOver => {
                    eprintln!("clickable PointerOver");
                    self.hovered.signal(true);
                }
                PointerEventKind::PointerOut => {
                    eprintln!("clickable PointerOut");
                    self.hovered.signal(false);
                }
                _ => {}
            },
            Event::Keyboard(key) => {
                if key.state == KeyState::Down {
                    let press = match key.key {
                        Key::Enter => true,
                        Key::Character(ref s) if s == " " => true,
                        _ => false,
                    };

                    if press {
                        ctx.set_handled();
                        self.active.signal(true);
                        self.clicked.signal(());
                    }

                    if key.key == Key::Tab {
                        if key.modifiers.contains(Modifiers::SHIFT) {
                            ctx.focus_prev();
                        } else {
                            ctx.focus_next();
                        }
                    }
                }
                if key.state == KeyState::Up {
                    self.active.signal(false);
                }
            }
            Event::FocusGained => {
                eprintln!("clickable FocusGained");
                self.focused.signal(true)
            }
            Event::FocusLost => {
                eprintln!("clickable FocusLost");
                self.focused.signal(false)
            }
            _ => {}
        }

        if !ctx.handled() {
            self.inner.route_event(ctx, event, env);
        }
    }

    fn paint(&self, ctx: &mut PaintCtx) {
        self.inner.paint(ctx)
    }
}
