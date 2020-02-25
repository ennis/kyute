use crate::event::Event;
use crate::model::LensExt;
use crate::model::{Data, Lens, Revision};
use crate::paint::RenderContext;
use crate::view::{EventCtx, View};
use piet::kurbo::Line;
use piet::kurbo::Rect;
use piet::kurbo::RoundedRect;
use piet::{Color, FontBuilder, RenderContext as RenderContextT, Text, TextLayoutBuilder};
use std::marker::PhantomData;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum CheckboxState {
    Unchecked,
    PartiallyChecked,
    Checked,
}

pub struct Checkbox<S: Data, Label: Lens<S, String>, State: Lens<S, CheckboxState>> {
    label: Label,
    state: State,
    _phantom: PhantomData<S>,
}

impl<S: Data, Label: Lens<S, String>, State: Lens<S, CheckboxState>> Checkbox<S, Label, State> {
    pub fn new(label: Label, state: State) -> Self {
        Checkbox {
            label,
            state,
            _phantom: PhantomData,
        }
    }
}

impl<S: Data, Label: Lens<S, String>, State: Lens<S, CheckboxState>> View<S>
    for Checkbox<S, Label, State>
{
    type Action = CheckboxState;

    fn event(&mut self, _e: &Event, _a: &mut EventCtx<CheckboxState>) {
        unimplemented!()
    }

    fn update(&mut self, _s: &Revision<S>) {}

    fn paint(&mut self, s: &S, ctx: &mut RenderContext) -> bool {
        let checked = self.state.get(s);
        draw_checkbox(checked, "test", ctx);
        false
    }
}

const BG_COLOR: Color = Color::rgb8(0x27, 0x28, 0x22);
const FG_COLOR: Color = Color::rgb8(0xf0, 0xf0, 0xea);

const NODE_BORDER_COLOR_A: Color = Color::rgb8(102, 102, 102);
const NODE_BORDER_COLOR_B: Color = Color::rgb8(51, 51, 51);
const NODE_BG_COLOR: Color = Color::rgb8(78, 78, 78);
const NODE_SELECTED_BG_COLOR: Color = Color::rgb8(89, 124, 148);

fn draw_checkbox(_state: CheckboxState, _label: &str, ctx: &mut RenderContext) {
    let (width, height) = (640.0, 480.0);

    let rect = Rect::new(0.0, 0.0, width, height);
    ctx.fill(rect, &BG_COLOR);
    ctx.stroke(Line::new((10.0, 50.0), (90.0, 90.0)), &FG_COLOR, 1.0);

    let (x, y) = (200.5, 200.5);
    let (w, h) = (145.0, 30.0);
    let rect = RoundedRect::new(x, y, x + w, y + h, 3.0);
    let outer_rect = RoundedRect::new(x - 1.0, y - 1.0, x + w + 1.0, y + h + 1.0, 3.0);
    let xsep = x + (w * 0.75).round();
    let sep = Line::new((xsep, y), (xsep, y + h));
    let sep2 = Line::new((xsep + 1.0, y), (xsep + 1.0, y + h));

    ctx.fill(rect, &NODE_BG_COLOR);
    ctx.stroke(sep, &NODE_BORDER_COLOR_A, 1.0);
    ctx.stroke(rect, &NODE_BORDER_COLOR_A, 1.0);
    ctx.stroke(outer_rect, &NODE_BORDER_COLOR_B, 1.0);
    ctx.stroke(sep2, &NODE_BORDER_COLOR_B, 1.0);

    let font_size = 16.0;
    let text_origin = (x + 30.0, y + 0.5 * (h + font_size - 3.0));

    let font = ctx
        .text()
        .new_font_by_name("Segoe UI", font_size)
        .build()
        .unwrap();
    let layout = ctx.text().new_text_layout(&font, "Load").build().unwrap();
    //let w: f64 = layout.width().into();
    ctx.draw_text(&layout, text_origin, &FG_COLOR);
}
