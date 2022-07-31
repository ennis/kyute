//! Clickable widget wrapper
use crate::{cache, event::PointerEventKind, widget::prelude::*, Signal, State};
use keyboard_types::{Key, KeyState, Modifiers};
use kyute::style::WidgetState;

/// Wraps an inner widget and allows the user to respond to clicks on it.
#[derive(Clone)]
pub struct Clickable<Inner> {
    id: WidgetId,
    inner: Inner,
    clicked: Signal<()>,
    active: State<bool>,
    focus: State<bool>,
    activated: Signal<bool>,
    hovered: Signal<bool>,
    focused: Signal<bool>,
}

impl<Inner: Widget + 'static> Clickable<Inner> {
    #[composable]
    pub fn new(inner: Inner) -> Clickable<Inner> {
        Clickable {
            id: WidgetId::here(),
            inner,
            active: cache::state(|| false),
            focus: cache::state(|| false),
            clicked: Signal::new(),
            activated: Signal::new(),
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
        self.activated.value() == Some(true)
    }

    /// Returns whether this button is active (holding the mouse button over it).
    pub fn deactivated(&self) -> bool {
        self.activated.value() == Some(false)
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

    fn layout(&self, ctx: &mut LayoutCtx, params: &LayoutParams, env: &Environment) -> Layout {
        let mut widget_state = params.widget_state;
        widget_state.set(WidgetState::ACTIVE, self.active.get());
        widget_state.set(WidgetState::FOCUS, self.focus.get());
        self.inner.layout(
            ctx,
            &LayoutParams {
                widget_state,
                ..*params
            },
            env,
        )
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
                    self.active.set(true);
                    self.activated.signal(true);
                    ctx.request_relayout();
                }
                PointerEventKind::PointerUp => {
                    self.active.set(false);
                    self.activated.signal(false);
                    self.clicked.signal(());
                    ctx.request_relayout();
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
                        self.active.set(true);
                        self.activated.signal(true);
                        self.clicked.signal(());
                        ctx.request_relayout();
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
                    self.active.set(false);
                    self.activated.signal(false);
                }
            }
            Event::FocusGained => {
                eprintln!("clickable FocusGained");
                self.focus.set(true);
                self.focused.signal(true);
                ctx.request_relayout();
            }
            Event::FocusLost => {
                eprintln!("clickable FocusLost");
                self.focus.set(false);
                self.focused.signal(false);
                ctx.request_relayout();
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
