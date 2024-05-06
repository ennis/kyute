//! Clickable widget wrapper
use std::{any::Any, mem};

use keyboard_types::{Key, KeyState};
use tracing::trace;

use crate::{
    widget::{prelude::*, TreeCtx, WidgetPod, WidgetPtr},
    State,
};

type DefaultClickHandler = fn(&mut TreeCtx);

/*
pub const ACTIVE: AmbientKey<bool> = AmbientKey::new("kyute.clickable.active");
pub const FOCUSED: AmbientKey<bool> = AmbientKey::new("kyute.clickable.focused");
pub const HOVERED: AmbientKey<bool> = AmbientKey::new("kyute.clickable.hovered");
*/

pub struct Clickable<OnClick = DefaultClickHandler> {
    content: WidgetPtr,
    active: State<bool>,
    focus: State<bool>,
    hovered: State<bool>,
    on_click: OnClick,
}

impl Clickable<DefaultClickHandler> {
    /// Creates a new clickable widget.
    pub fn new(content: impl Widget + 'static) -> Clickable<DefaultClickHandler> {
        Clickable {
            content: WidgetPod::new(content),
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
    pub fn on_clicked<OnClicked>(self, on_clicked: OnClicked) -> Clickable<OnClicked>
    where
        OnClicked: FnOnce(&mut TreeCtx),
    {
        Clickable {
            content: self.content,
            active: Default::default(),
            focus: Default::default(),
            hovered: Default::default(),
            on_click: on_clicked,
        }
    }
}

impl<OnClick> Widget for Clickable<OnClick>
where
    OnClick: Fn(&mut TreeCtx),
{
    fn update(&self, cx: &mut TreeCtx) {
        self.content.update(cx);
    }

    fn provide(&self, key: &'static str) -> Option<&dyn Any> {
        match key {
            "kyute.clickable.active" => Some(&self.active),
            "kyute.clickable.focused" => Some(&self.focus),
            "kyute.clickable.hovered" => Some(&self.hovered),
            _ => None,
        }
    }

    fn event(&self, cx: &mut TreeCtx, event: &mut Event) {
        match event {
            Event::PointerDown(ref _p) => {
                eprintln!("clickable PointerDown");
                // this will notify anything that depends on the active flag
                self.active.set(cx, true);
            }
            Event::PointerUp(ref _p) => {
                self.active.set(cx, false);
                (self.on_click)(cx);
            }
            Event::PointerOver(ref _p) => {
                self.hovered.set(cx, true);
            }
            Event::PointerOut(ref _p) => {
                self.hovered.set(cx, false);
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
                            self.active.set(cx, true);
                        }
                    }
                    KeyState::Up => {
                        //event.handled = true;
                        if *self.active.get(cx) {
                            (self.on_click)(cx);
                            self.active.set(cx, false);
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
