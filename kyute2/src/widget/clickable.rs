//! Clickable widget wrapper
use std::{any::Any, mem};

use keyboard_types::{Key, KeyState};
use tracing::trace;

use crate::{
    widget::{prelude::*, TreeCtx, WidgetPod, WidgetPtr},
    State,
};

type DefaultClickHandler = fn(&mut TreeCtx);

#[derive(Copy, Clone, Default)]
pub struct ClickableState {
    pub active: bool,
    pub focus: bool,
    pub hovered: bool,
}

impl ClickableState {
    pub fn at(cx: &mut TreeCtx) -> ClickableState {
        cx.find_ancestor::<Clickable>()
            .map(|clickable| clickable.widget.state.get(cx).clone())
            .unwrap_or_default()
    }
}

pub struct Clickable {
    content: WidgetPtr,
    state: State<ClickableState>,
    on_click: Box<dyn Fn(&mut TreeCtx)>,
}

impl Clickable {
    /// Creates a new clickable widget.
    pub fn new(content: impl Widget + 'static) -> Clickable {
        Clickable {
            content: WidgetPod::new(content),
            state: Default::default(),
            on_click: Box::new(|_cx| {
                trace!("Clickable::on_clicked");
            }),
        }
    }

    /// Sets the click handler.
    #[must_use]
    pub fn on_clicked<OnClicked>(self, on_clicked: OnClicked) -> Clickable
    where
        OnClicked: Fn(&mut TreeCtx) + 'static,
    {
        Clickable {
            content: self.content,
            state: self.state,
            on_click: Box::new(on_clicked),
        }
    }
}

impl Widget for Clickable {
    fn update(&self, cx: &mut TreeCtx) {
        self.content.update(cx);
    }

    fn event(&self, cx: &mut TreeCtx, event: &mut Event) {
        match event {
            Event::PointerDown(ref _p) => {
                eprintln!("clickable PointerDown");
                // this will notify anything that depends on the active flag
                self.state.update(cx, |state| state.active = true);
            }
            Event::PointerUp(ref _p) => {
                self.state.update(cx, |state| state.active = false);
                (self.on_click)(cx);
            }
            Event::PointerOver(ref _p) => {
                self.state.update(cx, |state| state.hovered = true);
            }
            Event::PointerOut(ref _p) => {
                self.state.update(cx, |state| state.hovered = false);
            }
            Event::Keyboard(ref key) => {
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
                            self.state.update(cx, |state| state.active = true);
                        }
                    }
                    KeyState::Up => {
                        if self.state.get_untracked().active {
                            (self.on_click)(cx);
                            self.state.update(cx, |state| state.active = false);
                        }
                    }
                }
            }
            _ => {}
        }
    }

    fn hit_test(&self, result: &mut HitTestResult, position: Point) -> bool {
        self.content.hit_test(result, position)
    }

    fn layout(&self, ctx: &mut LayoutCtx, constraints: &BoxConstraints) -> Geometry {
        self.content.layout(ctx, constraints)
    }

    fn paint(&self, cx: &mut PaintCtx) {
        self.content.paint(cx)
    }
}
