//! Clickable widget wrapper
use keyboard_types::{Key, KeyState};

use crate::prelude::*;

#[derive(Copy, Clone, Default)]
pub struct ClickableState {
    pub active: bool,
    pub focus: bool,
    pub hovered: bool,
}

impl ClickableState {
    pub fn at(cx: &mut Ctx) -> ClickableState {
        State::at(cx).expect("ClickableState not found")
    }
}

pub struct Clickable {
    state: State<ClickableState>,
    on_click: Box<dyn Fn(&mut Ctx)>,
    content: WidgetPtr,
}

impl Clickable {
    /// Creates a new clickable widget.
    pub fn new(content: WidgetPtr, on_click: impl Fn(&mut Ctx) + 'static) -> WidgetPtr<Clickable> {
        WidgetPod::new_cyclic(move |weak| Clickable {
            state: State::new(ClickableState::default()),
            on_click: Box::new(on_click),
            content: content.with_parent(weak),
        })
    }
}

impl Widget for Clickable {
    fn mount(&mut self, cx: &mut Ctx) {
        self.content.mount(cx)
    }

    fn environment(&self) -> Environment {
        Environment::new().add(self.state.clone())
    }

    fn event(&mut self, cx: &mut Ctx, event: &mut Event) {
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
                        if self.state.get().active {
                            (self.on_click)(cx);
                            self.state.update(cx, |state| state.active = false);
                        }
                    }
                }
            }
            _ => {}
        }
    }

    fn hit_test(&mut self, result: &mut HitTestResult, position: Point) -> bool {
        self.content.hit_test(result, position)
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, constraints: &BoxConstraints) -> Geometry {
        self.content.layout(ctx, constraints)
    }

    fn paint(&mut self, cx: &mut PaintCtx) {
        self.content.paint(cx)
    }
}
