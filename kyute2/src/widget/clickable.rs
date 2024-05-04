//! Clickable widget wrapper
use std::mem;

use keyboard_types::{Key, KeyState};
use tracing::trace;

use crate::{
    widget::{prelude::*, WidgetState, WidgetVisitor},
    with_ambient, AmbientKey, State,
};

type DefaultClickHandler = fn(&mut TreeCtx);

pub const ACTIVE: AmbientKey<bool> = AmbientKey::new("kyute.clickable.active");
pub const FOCUSED: AmbientKey<bool> = AmbientKey::new("kyute.clickable.focused");
pub const HOVERED: AmbientKey<bool> = AmbientKey::new("kyute.clickable.hovered");

pub struct Clickable<T, OnClick = DefaultClickHandler> {
    id: WidgetId,
    content: T,
    // idea: instead of being relative to the current widget, state entries
    // could store the path tree relative to the root,
    // this way, all updates would be dispatched from a single location (the root)
    //
    // The flow would be like this:
    // - dispatch event to widget
    // - control goes back to root event loop
    // - root event loop sees that there are pending updates
    // - root event loop dispatches the updates
    active: State<bool>,
    focus: State<bool>,
    hovered: State<bool>,
    on_click: OnClick,
}

impl<T> Clickable<T, DefaultClickHandler> {
    /// Creates a new clickable widget.
    pub fn new(content: T) -> Clickable<T, DefaultClickHandler> {
        Clickable {
            id: WidgetId::next(),
            content,
            active: Default::default(),
            focus: Default::default(),
            hovered: Default::default(),
            on_click: |_cx| {
                trace!("Clickable::on_clicked");
            },
        }
    }

    /// Sets the click handler.
    #[must_use]
    pub fn on_clicked<OnClicked>(self, on_clicked: OnClicked) -> Clickable<T, OnClicked>
    where
        OnClicked: FnOnce(&mut TreeCtx),
    {
        Clickable {
            id: self.id,
            content: self.content,
            active: Default::default(),
            focus: Default::default(),
            hovered: Default::default(),
            on_click: on_clicked,
        }
    }
}

impl<T: Widget, OnClick> Widget for Clickable<T, OnClick>
where
    OnClick: FnMut(&mut TreeCtx),
{
    fn id(&self) -> WidgetId {
        self.id
    }

    fn visit_child(&mut self, cx: &mut TreeCtx, id: WidgetId, visitor: &mut WidgetVisitor) {
        with_ambient(cx, FOCUSED, &mut self.focus, |cx| {
            with_ambient(cx, ACTIVE, &mut self.active, |cx| {
                with_ambient(cx, HOVERED, &mut self.hovered, |cx| {
                    if self.content.id() == id {
                        visitor(cx, &mut self.content);
                    }
                });
            });
        });
    }

    fn update(&mut self, cx: &mut TreeCtx) -> ChangeFlags {
        with_ambient(cx, FOCUSED, &mut self.focus, |cx| {
            with_ambient(cx, ACTIVE, &mut self.active, |cx| {
                with_ambient(cx, HOVERED, &mut self.hovered, |cx| {
                    cx.update(&mut self.content);
                });
            });
        });

        ChangeFlags::empty()
    }

    fn event(&mut self, cx: &mut TreeCtx, event: &mut Event) -> ChangeFlags {
        let mut change_flags = ChangeFlags::empty();

        match event.kind {
            EventKind::PointerDown(ref _p) => {
                eprintln!("clickable PointerDown");
                event.handled = true;
                // this will notify anything that depends on the active flag
                self.active.set(cx, true);
            }
            EventKind::PointerUp(ref _p) => {
                event.handled = true;
                self.active.set(cx, false);
                // TODO that's a bit verbose
                with_ambient(cx, FOCUSED, &mut self.focus, |cx| {
                    with_ambient(cx, ACTIVE, &mut self.active, |cx| {
                        with_ambient(cx, HOVERED, &mut self.hovered, |cx| {
                            (self.on_click)(cx);
                        });
                    });
                });
            }
            EventKind::PointerOver(ref _p) => {
                self.hovered.set(cx, true);
            }
            EventKind::PointerOut(ref _p) => {
                self.hovered.set(cx, false);
            }
            EventKind::Keyboard(ref key) => {
                match key.state {
                    KeyState::Down => {
                        // activate a clickable with Enter or the space bar
                        // but delay the click until the key is released
                        let press = match key.key {
                            Key::Enter => true,
                            Key::Character(ref s) if s == " " => true,
                            _ => false,
                        };

                        if press {
                            self.active.set(cx, true);
                        }
                    }
                    KeyState::Up => {
                        event.handled = true;
                        if *self.active.get() {
                            // TODO verbosity
                            with_ambient(cx, FOCUSED, &mut self.focus, |cx| {
                                with_ambient(cx, ACTIVE, &mut self.active, |cx| {
                                    with_ambient(cx, HOVERED, &mut self.hovered, |cx| {
                                        (self.on_click)(cx);
                                    });
                                });
                            });
                            self.active.set(cx, false);
                        }
                    }
                }
            }
            _ => {}
        }

        change_flags
    }

    fn hit_test(&self, result: &mut HitTestResult, position: Point) -> bool {
        result.hit_test_child(&self.content, position)
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, constraints: &BoxConstraints) -> Geometry {
        ctx.layout(&mut self.content, constraints)
    }

    fn paint(&mut self, cx: &mut PaintCtx) {
        cx.paint(&mut self.content);
    }

    /*fn natural_width(&mut self, height: f64) -> f64 {
        self.content.natural_width(height)
    }

    fn natural_height(&mut self, width: f64) -> f64 {
        self.content.natural_height(width)
    }

    fn natural_baseline(&mut self, params: &BoxConstraints) -> f64 {
        self.content.natural_baseline(params)
    }

    fn hit_test(&self, ctx: &mut HitTestResult, position: Point) -> bool {
        let hit = self.content.hit_test(ctx, position);
        if hit {
            ctx.add(self.id);
        }
        hit
    }

    fn paint(&mut self, ctx: &mut PaintCtx) {
        ctx.paint(&mut self.content);
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn debug(&self, visitor: &mut DebugWriter) {
        visitor.type_name("ClickableElement");
        visitor.property("id", self.id);
        visitor.property("state", self.state);
        visitor.property("events", self.events);
        visitor.child("inner", &self.content);
    }*/
}
