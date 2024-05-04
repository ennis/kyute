use std::sync::Arc;

use kurbo::{Insets, Vec2};
use smallvec::smallvec;

use crate::{
    drawing::BoxShadow,
    text::{TextSpan, TextStyle},
    theme,
    theme::Theme,
    widget::{
        align::Align,
        clickable::{ACTIVE, FOCUSED, HOVERED},
        decoration::DecoratedBox,
        prelude::*,
        BorderStyle, Clickable, Constrained, Padding, RoundedRectBorder, ShapeDecoration, Text, WidgetExt, WidgetState,
    },
    with_cx, Alignment, Color, Widget,
};

pub fn button(label: &str) -> Clickable<impl Widget> {
    // FIXME: annoyingly we need to allocate to move the string in the closure
    // in that case it's not too bad because we're already allocating for the TextSpan
    let label = label.to_string();
    with_cx(move |cx: &mut TreeCtx| {
        let theme = &theme::DARK_THEME;
        let text_style = Arc::new(
            TextStyle::new()
                .font_size(theme.font_size)
                .font_family(theme.font_family)
                .color(theme.text_color),
        );
        let text = TextSpan::new(label.clone(), text_style);

        let hovered = *HOVERED.get(cx);
        let active = *ACTIVE.get(cx);
        let focused = *FOCUSED.get(cx);

        let decoration = if theme.dark_mode {
            ShapeDecoration {
                fill: if hovered {
                    Color::from_rgb_u8(100, 100, 100).into()
                } else if active {
                    Color::from_rgb_u8(60, 60, 60).into()
                } else {
                    Color::from_rgb_u8(88, 88, 88).into()
                },
                border: RoundedRectBorder {
                    color: if focused {
                        theme.accent_color
                    } else {
                        Color::from_rgb_u8(49, 49, 49)
                    },
                    radius: 8.0,
                    dimensions: Insets::uniform(1.0),
                    style: BorderStyle::Solid,
                },
                shadows: if !active {
                    smallvec![
                        BoxShadow {
                            color: Color::from_rgb_u8(115, 115, 115),
                            offset: Vec2::new(0.0, 1.0),
                            blur: 0.0,
                            spread: 0.0,
                            inset: true,
                        },
                        BoxShadow {
                            color: Color::from_rgb_u8(49, 49, 49),
                            offset: Vec2::new(0.0, 1.0),
                            blur: 2.0,
                            spread: -1.0,
                            inset: false
                        }
                    ]
                } else {
                    smallvec![]
                },
            }
        } else {
            ShapeDecoration {
                fill: if hovered {
                    Color::from_rgb_u8(240, 240, 240).into()
                } else if active {
                    Color::from_rgb_u8(240, 240, 240).into()
                } else {
                    Color::from_rgb_u8(255, 255, 255).into()
                },
                border: RoundedRectBorder {
                    color: if focused {
                        theme.accent_color
                    } else {
                        Color::from_rgb_u8(180, 180, 180)
                    },
                    radius: 8.0,
                    dimensions: Insets::uniform(1.0),
                    style: BorderStyle::Solid,
                },
                shadows: smallvec![BoxShadow {
                    color: Color::from_rgb_u8(180, 180, 180),
                    offset: Vec2::new(0.0, 1.0),
                    blur: 0.0,
                    spread: 0.0,
                    inset: false,
                }],
            }
        };
        DecoratedBox::new(
            decoration,
            Padding::new(
                Insets::uniform(3.0),
                Constrained {
                    constraints: BoxConstraints {
                        min: Size::new(72.0, 22.0),
                        ..Default::default()
                    },
                    content: Align::new(Alignment::CENTER, Alignment::CENTER, Text::new(text)),
                },
            ),
        )
    })
    .clickable()
}
