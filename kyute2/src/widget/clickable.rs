//! Clickable widget wrapper
use crate::widget::prelude::*;
use keyboard_types::{Key, KeyState};
use tracing::trace;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Default)]
struct ClickableState {
    active: bool,
    focus: bool,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Default)]
struct ClickableEvents {
    focused: Option<bool>,
    clicked: Option<()>,
    activated: Option<bool>,
}

/// Wraps an inner widget and allows the user to respond to clicks on it.
#[derive(Clone)]
pub struct Clickable<Inner> {
    id: WidgetId,
    inner: Inner,
    state: State<ClickableState>,
    events: Signal<ClickableEvents>,
    // This is a separate so that we don't necessarily invalidate
    // when the pointer enters/exits the widget.
    hovered: Signal<bool>,
}

impl<Inner: Widget + 'static> Clickable<Inner> {
    #[composable]
    pub fn new(inner: Inner) -> Clickable<Inner> {
        Clickable {
            id: WidgetId::here(),
            inner,
            state: State::default(),
            events: Signal::new(),
            hovered: Signal::new(),
        }
    }

    /*#[must_use]
    pub fn on_click(self, f: impl FnOnce()) -> Self {
        if self.clicked.signalled() {
            f();
        }
        self
    }*/

    /// Returns whether this button has been clicked.
    pub fn clicked(&self) -> bool {
        self.events.value().and_then(|events| events.clicked).is_some()
    }

    /// Returns whether this button is active (holding the mouse button over it).
    pub fn activated(&self) -> bool {
        self.events.value().and_then(|events| events.activated) == Some(true)
    }

    /// Returns whether this button is active (holding the mouse button over it).
    pub fn deactivated(&self) -> bool {
        self.events.value().and_then(|events| events.activated) == Some(false)
    }

    /*#[must_use]
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
    }*/

    /// Returns whether the pointer entered the clickable area.
    pub fn pointer_entered(&self) -> bool {
        self.hovered.value() == Some(true)
    }

    /// Returns whether the pointer exited the clickable area.
    pub fn pointer_exited(&self) -> bool {
        self.hovered.value() == Some(false)
    }

    /*#[must_use]
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
    }*/

    pub fn focus_gained(&self) -> bool {
        self.events.value().and_then(|events| events.focused) == Some(true)
    }

    pub fn focus_lost(&self) -> bool {
        self.events.value().and_then(|events| events.focused) == Some(false)
    }

    /*#[must_use]
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
    }*/

    /*/// Returns a reference to the inner widget.
    pub fn inner(&self) -> &Inner {
        &self.inner
    }

    /// Returns a mutable reference to the inner widget.
    pub fn inner_mut(&mut self) -> &mut Inner {
        &mut self.inner
    }*/
}

impl<Inner: Widget> Widget for Clickable<Inner> {
    type Element = ClickableElement<Inner::Element>;

    fn id(&self) -> WidgetId {
        self.id
    }

    fn build(self, cx: &mut TreeCtx, env: &Environment) -> Self::Element {
        ClickableElement {
            id: self.id,
            inner: self.inner.build(cx, env),
            state: self.state,
            events: self.events,
            hovered: self.hovered,
        }
    }

    fn update(self, cx: &mut TreeCtx, element: &mut Self::Element, env: &Environment) -> ChangeFlags {
        // TODO: not sure we need to update the signals here?
        element.events = self.events;
        element.state = self.state;
        element.hovered = self.hovered;
        self.inner.update(cx, &mut element.inner, env)
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
pub struct ClickableElement<Inner> {
    id: WidgetId,
    inner: Inner,
    state: State<ClickableState>,
    events: Signal<ClickableEvents>,
    hovered: Signal<bool>,
}

impl<Inner: Element> Element for ClickableElement<Inner> {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, params: &LayoutParams) -> Geometry {
        self.inner.layout(ctx, params)
    }

    fn event(&mut self, ctx: &mut EventCtx, event: &mut Event) -> ChangeFlags {
        let mut change_flags = ChangeFlags::empty();

        // We don't capture anything, so forward to children first.
        if let Some(target) = event.next_target() {
            assert_eq!(self.inner.id(), target);
            change_flags |= self.inner.event(ctx, event);
        }

        if event.handled {
            return change_flags;
        }

        match event.kind {
            EventKind::PointerDown(ref p) => {
                ctx.request_focus(self.id);
                ctx.request_pointer_capture(self.id);
                event.handled = true;

                self.state.update_with(|state| {
                    state.active = true;
                    true
                });
                self.events.signal(ClickableEvents {
                    activated: Some(true),
                    ..Default::default()
                });

                // TODO: for now return "PAINT" because we make the assumption that the only
                // thing affected by widget state is the painting. Notably, we assume that layout
                // doesn't depend on the widget state.
                ChangeFlags::PAINT
            }
            EventKind::PointerUp(ref p) => {
                event.handled = true;
                self.state.update_with(|state| {
                    state.active = false;
                    true
                });
                self.events.signal(ClickableEvents {
                    activated: Some(false),
                    clicked: Some(()),
                    ..Default::default()
                });
                ChangeFlags::PAINT
            }
            EventKind::PointerOver(ref p) => {
                //eprintln!("clickable PointerOver");
                self.hovered.signal(true);
                ChangeFlags::NONE
                //ChangeFlags::PAINT
            }
            EventKind::PointerOut(ref p) => {
                //eprintln!("clickable PointerOut");
                self.hovered.signal(false);
                ChangeFlags::NONE
                //ChangeFlags::PAINT
            }
            EventKind::Keyboard(ref key) => {
                match key.state {
                    KeyState::Down => {
                        let press = match key.key {
                            Key::Enter => true,
                            Key::Character(ref s) if s == " " => true,
                            _ => false,
                        };

                        if press {
                            event.handled = true;

                            self.state.update_with(|state| {
                                state.active = true;
                                true
                            });
                            self.events.signal(ClickableEvents {
                                activated: Some(true),
                                clicked: Some(()),
                                ..Default::default()
                            });
                            //ctx.request_relayout();
                        }

                        /*if key.key == Key::Tab {
                            if key.modifiers.contains(Modifiers::SHIFT) {
                                ctx.focus_prev();
                            } else {
                                ctx.focus_next();
                            }
                        }*/
                        ChangeFlags::PAINT
                    }
                    KeyState::Up => {
                        self.state.update_with(|state| {
                            state.active = false;
                            true
                        });
                        self.events.signal(ClickableEvents {
                            activated: Some(false),
                            clicked: Some(()),
                            ..Default::default()
                        });
                        ChangeFlags::PAINT
                    }
                }
            }
            /*EventKind::FocusGained => {
                eprintln!("clickable FocusGained");
                self.focus.set(true);
                self.focused.signal(true);
                ctx.request_relayout();
            }
            EventKind::FocusLost => {
                eprintln!("clickable FocusLost");
                self.focus.set(false);
                self.focused.signal(false);
                ctx.request_relayout();
            }*/
            _ => ChangeFlags::NONE,
        }
    }

    fn natural_size(&mut self, axis: Axis, params: &LayoutParams) -> f64 {
        self.inner.natural_size(axis, params)
    }

    fn natural_baseline(&mut self, params: &LayoutParams) -> f64 {
        self.inner.natural_baseline(params)
    }

    fn hit_test(&self, ctx: &mut HitTestResult, position: Point) -> bool {
        let hit = self.inner.hit_test(ctx, position);
        if hit {
            ctx.add(self.id);
        }
        hit
    }

    fn paint(&mut self, ctx: &mut PaintCtx) {
        self.inner.paint(ctx)
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn debug(&self, visitor: &mut DebugWriter) {
        visitor.type_name("ClickableElement");
        visitor.property("id", self.id);
        visitor.property("state", self.state.get());
        visitor.child("inner", &self.inner);
    }
}
