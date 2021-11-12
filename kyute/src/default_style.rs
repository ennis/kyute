use crate::{
    style,
    style::{
        Angle, BlendMode, Border, BorderPosition, Brush, GradientType, Length, Shape, State,
        StateFilter, Style, StyleSet,
    },
    theme, Environment,
};
use kyute_shell::drawing::Color;

/// Creates a default application style
pub fn get_default_application_style() -> Environment {
    let FRAME_BG_SUNKEN_COLOR: Color = Color::new(0.227, 0.227, 0.227, 1.0);
    let FRAME_BG_NORMAL_COLOR: Color = Color::new(0.326, 0.326, 0.326, 1.0);
    let FRAME_BG_RAISED_COLOR: Color = Color::new(0.424, 0.424, 0.424, 1.0);
    let FRAME_FOCUS_COLOR: Color = Color::new(0.600, 0.600, 0.900, 1.0);
    let FRAME_BORDER_COLOR: Color = Color::new(0.130, 0.130, 0.130, 1.0);
    let BUTTON_BACKGROUND_TOP_COLOR: Color = Color::new(0.450, 0.450, 0.450, 1.0);
    let BUTTON_BACKGROUND_BOTTOM_COLOR: Color = Color::new(0.400, 0.400, 0.400, 1.0);
    let BUTTON_BACKGROUND_TOP_COLOR_HOVER: Color = Color::new(0.500, 0.500, 0.500, 1.0);
    let BUTTON_BACKGROUND_BOTTOM_COLOR_HOVER: Color = Color::new(0.450, 0.450, 0.450, 1.0);
    let BUTTON_BORDER_BOTTOM_COLOR: Color = Color::new(0.100, 0.100, 0.100, 1.0);
    let BUTTON_BORDER_TOP_COLOR: Color = Color::new(0.180, 0.180, 0.180, 1.0);
    let WIDGET_OUTER_GROOVE_BOTTOM_COLOR: Color = Color::new(1.000, 1.000, 1.000, 0.2);
    let WIDGET_OUTER_GROOVE_TOP_COLOR: Color = Color::new(1.000, 1.000, 1.000, 0.0);
    let FRAME_BG_SUNKEN_COLOR_HOVER: Color = Color::new(0.180, 0.180, 0.180, 1.0);

    //----------------------------------------------------------------------------------------------
    // buttons
    let button_style_set = StyleSet::builder()
        .with_shape(Shape::RoundedRect(Length::Dip(2.0)))
        .with_style(Style {
            fill: Some(Brush::Gradient {
                angle: Angle::degrees(90.0),
                ty: GradientType::Linear,
                stops: vec![
                    (0.0, BUTTON_BACKGROUND_BOTTOM_COLOR),
                    (1.0, BUTTON_BACKGROUND_TOP_COLOR),
                ],
                reverse: false,
            }),
            borders: vec![
                Border {
                    opacity: 1.0,
                    blend_mode: BlendMode::Normal,
                    position: BorderPosition::Inside(Length::zero()),
                    width: Length::Dip(1.0),
                    brush: Brush::SolidColor(BUTTON_BORDER_BOTTOM_COLOR),
                },
                Border {
                    opacity: 1.0,
                    blend_mode: BlendMode::Normal,
                    position: BorderPosition::Outside(Length::zero()),
                    width: Length::Dip(1.0),
                    brush: Brush::Gradient {
                        angle: Angle::degrees(90.0),
                        ty: GradientType::Linear,
                        stops: vec![
                            (0.0, WIDGET_OUTER_GROOVE_BOTTOM_COLOR),
                            (0.3, WIDGET_OUTER_GROOVE_TOP_COLOR),
                        ],
                        reverse: false,
                    },
                },
            ],
            ..Style::default()
        })
        .with_style(Style {
            state_filter: StateFilter {
                value: State::HOVER,
                mask: State::HOVER,
            },
            fill: Some(Brush::Gradient {
                angle: Angle::degrees(90.0),
                ty: GradientType::Linear,
                stops: vec![
                    (0.0, BUTTON_BACKGROUND_BOTTOM_COLOR_HOVER),
                    (1.0, BUTTON_BACKGROUND_TOP_COLOR_HOVER),
                ],
                reverse: false,
            }),
            ..Style::default()
        })
        .build();

    //----------------------------------------------------------------------------------------------
    // slider knobs
    let slider_knob_style_set = StyleSet::builder()
        .with_shape(Shape::Path(
            "M 0.5 0.5 L 10.5 0.5 L 10.5 5.5 L 5.5 10.5 L 0.5 5.5 Z".to_string(),
        ))
        .with_style(Style {
            fill: Some(Brush::Gradient {
                angle: Angle::degrees(90.0),
                ty: GradientType::Linear,
                stops: vec![
                    (0.0, BUTTON_BACKGROUND_BOTTOM_COLOR),
                    (1.0, BUTTON_BACKGROUND_TOP_COLOR),
                ],
                reverse: false,
            }),
            borders: vec![Border {
                opacity: 1.0,
                blend_mode: BlendMode::Normal,
                position: BorderPosition::Inside(Length::zero()),
                width: Length::Dip(1.0),
                brush: Brush::SolidColor(BUTTON_BORDER_BOTTOM_COLOR),
            }],
            ..Style::default()
        })
        .with_style(Style {
            state_filter: StateFilter {
                value: State::HOVER,
                mask: State::HOVER,
            },
            fill: Some(Brush::Gradient {
                angle: Angle::degrees(90.0),
                ty: GradientType::Linear,
                stops: vec![
                    (0.0, BUTTON_BACKGROUND_BOTTOM_COLOR_HOVER),
                    (1.0, BUTTON_BACKGROUND_TOP_COLOR_HOVER),
                ],
                reverse: false,
            }),
            ..Style::default()
        })
        .build();

    //----------------------------------------------------------------------------------------------
    // slider track
    let slider_track_style_set = StyleSet::builder()
        .with_shape(Shape::RoundedRect(style::Length::Dip(2.0)))
        .with_style(Style {
            fill: Some(Brush::SolidColor(FRAME_BG_SUNKEN_COLOR)),
            borders: vec![
                Border {
                    opacity: 1.0,
                    blend_mode: BlendMode::Normal,
                    position: BorderPosition::Inside(Length::zero()),
                    width: Length::Dip(1.0),
                    brush: Brush::SolidColor(BUTTON_BORDER_BOTTOM_COLOR),
                },
                Border {
                    opacity: 1.0,
                    blend_mode: BlendMode::Normal,
                    position: BorderPosition::Outside(Length::zero()),
                    width: Length::Dip(1.0),
                    brush: Brush::Gradient {
                        angle: Angle::degrees(90.0),
                        ty: GradientType::Linear,
                        stops: vec![
                            (0.0, WIDGET_OUTER_GROOVE_BOTTOM_COLOR),
                            (0.3, WIDGET_OUTER_GROOVE_TOP_COLOR),
                        ],
                        reverse: false,
                    },
                },
            ],
            ..Style::default()
        })
        .build();

    let text_edit_style_set = StyleSet::builder()
        .with_shape(Shape::Rect)
        .with_style(Style {
            fill: Some(Brush::SolidColor(FRAME_BG_SUNKEN_COLOR)),
            borders: vec![
                Border {
                    opacity: 1.0,
                    blend_mode: BlendMode::Normal,
                    position: BorderPosition::Inside(Length::zero()),
                    width: Length::Dip(1.0),
                    brush: Brush::SolidColor(BUTTON_BORDER_BOTTOM_COLOR),
                },
                Border {
                    opacity: 1.0,
                    blend_mode: BlendMode::Normal,
                    position: BorderPosition::Outside(Length::zero()),
                    width: Length::Dip(1.0),
                    brush: Brush::Gradient {
                        angle: Angle::degrees(90.0),
                        ty: GradientType::Linear,
                        stops: vec![
                            (0.0, WIDGET_OUTER_GROOVE_BOTTOM_COLOR),
                            (0.3, WIDGET_OUTER_GROOVE_TOP_COLOR),
                        ],
                        reverse: false,
                    },
                },
            ],
            ..Style::default()
        })
        .with_style(style::Style {
            state_filter: StateFilter {
                value: State::HOVER,
                mask: State::HOVER,
            },
            fill: Some(Brush::SolidColor(FRAME_BG_SUNKEN_COLOR)),
            ..Style::default()
        })
        .build();

    Environment::new()
        .add(theme::BUTTON_STYLE, button_style_set)
        .add(theme::SLIDER_KNOB_STYLE, slider_knob_style_set)
        .add(theme::SLIDER_TRACK_STYLE, slider_track_style_set)
        .add(theme::TEXT_EDIT_BACKGROUND_STYLE, text_edit_style_set)
        .add(theme::SLIDER_TRACK_Y, 9.0)
        .add(theme::SLIDER_TRACK_HEIGHT, 4.0)
        .add(theme::SLIDER_KNOB_WIDTH, 11.0)
        .add(theme::SLIDER_KNOB_HEIGHT, 11.0)
        .add(theme::SLIDER_KNOB_Y, 7.0)
        .add(theme::SLIDER_HEIGHT, 14.0)
        .add(theme::TEXT_EDIT_CARET_COLOR, Color::new(1.0,1.0,1.0,1.0))
        .add(theme::TEXT_EDIT_BORDER_COLOR, Color::new(0.0,0.0,0.0,1.0))
        .add(theme::TEXT_EDIT_BACKGROUND_COLOR, Color::new(1.0,1.0,1.0,1.0))
        .add(theme::TEXT_COLOR, Color::new(0.96,0.96,0.96,1.0))
        .add(theme::SELECTED_TEXT_BACKGROUND_COLOR, Color::new(0.6,0.6,0.8,1.0))
        .add(theme::SELECTED_TEXT_COLOR, Color::new(1.0,1.0,1.0,1.0))
}
