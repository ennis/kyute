use crate::{
    context::Ambient,
    drawing::BoxShadow,
    text::{TextSpan, TextStyle},
    theme,
    theme::Theme,
    widget::{
        prelude::*, Align, BorderStyle, Clickable, Constrained, DecoratedBox, Frame, Padding, RoundedRectBorder,
        ShapeDecoration, Text, WidgetExt,
    },
    Alignment, Color, Widget,
};
use kurbo::{Insets, Vec2};
use kyute2::widget::WidgetState;
use smallvec::smallvec;
use std::sync::Arc;

/*border-radius: 8px;
padding: 3px;
min-width: 80px;
min-height: 30px;

[$dark-mode] {
    background: rgb(88 88 88);
    border: solid 1px rgb(49 49 49);
    box-shadow: inset 0px 1px rgb(115 115 115), 0px 1px 2px -1px rgb(49 49 49);
    [:hover] background: rgb(100 100 100);
    [:focus] border: solid 1px #3895f2;
    [:active] background: rgb(60 60 60);
    [:active] box-shadow: none;
}

[!$dark-mode] {
    background: rgb(255 255 255);
    border: solid 1px rgb(180 180 180);
    box-shadow: 0px 1px 3px -1px rgb(180 180 180);
    [:hover] background: rgb(240 240 240);
    [:active] background: rgb(240 240 240);
    [:active] box-shadow: none;
    [:focus] border: solid 1px #3895f2;
}*/

pub fn button(label: &str) -> Clickable<impl Widget> {
    // FIXME: annoyingly we need to allocate to move the string in the closure
    // in that case it's not too bad because we're already allocating for the TextSpan
    let label = label.to_string();
    (move |cx: &mut TreeCtx| {
        let theme = Theme::ambient(cx).unwrap_or(&theme::DARK_THEME);
        let text_style = Arc::new(
            TextStyle::new()
                .font_size(theme.font_size)
                .font_family(theme.font_family)
                .color(theme.text_color),
        );
        let text = TextSpan::new(label, text_style);

        let state = WidgetState::ambient(cx).unwrap();

        let decoration = if theme.dark_mode {
            ShapeDecoration {
                fill: if state.hovered {
                    Color::from_rgb_u8(100, 100, 100).into()
                } else if state.active {
                    Color::from_rgb_u8(60, 60, 60).into()
                } else {
                    Color::from_rgb_u8(88, 88, 88).into()
                },
                border: RoundedRectBorder {
                    color: if state.focused {
                        theme.accent_color
                    } else {
                        Color::from_rgb_u8(49, 49, 49)
                    },
                    radius: 8.0,
                    dimensions: Insets::uniform(1.0),
                    style: BorderStyle::Solid,
                },
                shadows: if !state.active {
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
                fill: if state.hovered {
                    Color::from_rgb_u8(240, 240, 240).into()
                } else if state.active {
                    Color::from_rgb_u8(240, 240, 240).into()
                } else {
                    Color::from_rgb_u8(255, 255, 255).into()
                },
                border: RoundedRectBorder {
                    color: if state.focused {
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
        DecoratedBox {
            decoration,
            content: Padding {
                padding: Insets::uniform(3.0),
                content: Constrained {
                    constraints: BoxConstraints {
                        min: Size::new(72.0, 22.0),
                        ..Default::default()
                    },
                    content: Align {
                        x: Alignment::CENTER,
                        y: Alignment::CENTER,
                        width_factor: Some(0.0),
                        height_factor: Some(0.0),
                        content: Text::new(text),
                    },
                },
            },
        }
    })
    .clickable()
}
