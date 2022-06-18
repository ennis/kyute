//! Border description.
use crate::{
    style::{color::css_color, values::color::css_color},
    Color, Length, SideOffsets, Size, UnitExt,
};
use cssparser::{ParseError, Parser, Token};

/// CSS border style
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, serde::Deserialize)]
pub enum BorderStyle {
    #[serde(rename = "solid")]
    Solid,
    #[serde(rename = "dotted")]
    Dotted,
}

impl Default for BorderStyle {
    fn default() -> Self {
        BorderStyle::Solid
    }
}

/// CSS border shorthand
#[derive(Clone, Debug)]
pub struct Border {
    /// Left,top,right,bottom border widths.
    pub width: Length,
    pub color: Color,
    pub line_style: BorderStyle,
}

impl Border {
    pub(crate) fn parse_impl<'i>(input: &mut Parser<'i, '_>) -> Result<Border, ParseError<'i, ()>> {
        let mut line_width = None;
        let mut line_style = None;
        let mut color = None;

        loop {
            if line_width.is_none() {
                let width = input.try_parse(|input| {
                    if input.try_parse(|i| i.expect_ident_matching("thin")).is_ok() {
                        Ok(1.dip())
                    } else if input.try_parse(|i| i.expect_ident_matching("medium")).is_ok() {
                        Ok(2.dip())
                    } else if input.try_parse(|i| i.expect_ident_matching("thick")).is_ok() {
                        Ok(3.dip())
                    } else {
                        input.try_parse(length)
                    }
                });

                if let Ok(width) = width {
                    line_width = Some(width);
                    continue;
                }
            }

            if line_style.is_none() {
                let style = input.try_parse::<_, _, ParseError<'i, ()>>(|input| match input.next()? {
                    Token::Ident(ident) if &**ident == "solid" => Ok(BorderStyle::Solid),
                    Token::Ident(ident) if &**ident == "dotted" => Ok(BorderStyle::Dotted),
                    token => {
                        let token = token.clone();
                        Err(input.new_unexpected_token_error(token))
                    }
                });

                if let Ok(style) = style {
                    line_style = Some(style);
                    continue;
                }
            }

            if color.is_none() {
                if let Ok(c) = input.try_parse(css_color) {
                    color = Some(c);
                    continue;
                }
            }

            break;
        }

        if line_width.is_none() && line_style.is_none() && color.is_none() {
            return Err(input.new_custom_error(()));
        }

        let line_width = line_width.unwrap_or(Length::zero());

        Ok(Border {
            width: line_width,
            color: color.unwrap_or_default(),
            line_style: line_style.unwrap_or_default(),
        })
    }

    /*pub fn parse(css: &str) -> Result<Self, ParseError<()>> {
        parse_from_str(css, Self::parse_impl)
    }*/
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// border
////////////////////////////////////////////////////////////////////////////////////////////////////

/// border-radius
pub(crate) fn border_radius<'i>(input: &mut Parser<'i, '_>) -> Result<[Length; 4], ParseError<'i, ()>> {
    // <length-percentage>{1,4} [ / <length-percentage>{1,4} ]?
    // (but we don't support the '/' part, yet.)

    let length1 = length_percentage(input)?;
    let length2 = input.try_parse(length_percentage).ok();
    let length3 = input.try_parse(length_percentage).ok();
    let length4 = input.try_parse(length_percentage).ok();

    let radii = match (length1, length2, length3, length4) {
        (radius, None, None, None) => [radius; 4],
        (top_left_and_bottom_right, Some(top_right_and_bottom_left), None, None) => [
            top_left_and_bottom_right,
            top_right_and_bottom_left,
            top_left_and_bottom_right,
            top_right_and_bottom_left,
        ],
        (top_left, Some(top_right_and_bottom_left), Some(bottom_right), None) => [
            top_left,
            top_right_and_bottom_left,
            bottom_right,
            top_right_and_bottom_left,
        ],
        (top_left, Some(top_right), Some(bottom_right), Some(bottom_left)) => {
            [top_left, top_right, bottom_right, bottom_left]
        }
        _ => unreachable!(),
    };
    Ok(radii)
}

/*
impl Border {
    /// Creates a new border description with the specified side widths.
    pub fn new(left: Length, top: Length, right: Length, bottom: Length) -> Border {
        Border {
            widths: [left, top, right, bottom],
            paint: Paint::SolidColor(Color::new(0.0, 0.0, 0.0, 1.0)),
            line_style: Default::default(),
            blend_mode: BlendMode::SrcOver,
        }
    }

    pub fn new_all_same(width: Length) -> Border {
        Border {
            widths: [width, width, width, width],
            paint: Paint::SolidColor(Color::new(0.0, 0.0, 0.0, 1.0)),
            line_style: Default::default(),
            blend_mode: BlendMode::SrcOver,
        }
    }

    pub fn paint(mut self, paint: impl Into<Paint>) -> Self {
        self.paint = paint.into();
        self
    }

    pub fn blend(mut self, blend_mode: BlendMode) -> Self {
        self.blend_mode = blend_mode;
        self
    }

    pub fn calculate_widths(&self, scale_factor: f64, available_space: Size) -> [f64; 4] {
        [
            self.widths[0].to_dips(scale_factor, available_space.height), // top
            self.widths[1].to_dips(scale_factor, available_space.width),  // right
            self.widths[2].to_dips(scale_factor, available_space.height), // bottom
            self.widths[3].to_dips(scale_factor, available_space.width),  // left
        ]
    }

    /// Draws the described border in the given paint context, around the specified bounds.
    pub fn draw(&self, ctx: &mut PaintCtx, bounds: Rect, radii: [sk::Vector; 4]) {
        let rounded = !(radii[0].is_zero() && radii[1].is_zero() && radii[2].is_zero() && radii[3].is_zero());
        let [t, r, b, l] = self.calculate_widths(ctx.scale_factor, bounds.size);

        let mut rrect = if rounded {
            sk::RRect::new_rect_radii(bounds.to_skia(), &radii)
        } else {
            sk::RRect::new_rect(bounds.to_skia())
        };

        let canvas = ctx.surface.canvas();
        let mut paint = self.paint.to_sk_paint(bounds);
        paint.set_style(sk::PaintStyle::Fill);
        paint.set_blend_mode(self.blend_mode.to_skia());

        let inset_x = 0.5 * (l + r);
        let offset_x = 0.5 * (l - r);
        let inset_y = 0.5 * (t + b);
        let offset_y = 0.5 * (t - b);
        let inset_rrect = rrect
            .with_offset(Offset::new(offset_x, offset_y).to_skia())
            .with_inset(Offset::new(inset_x, inset_y).to_skia());
        canvas.draw_drrect(rrect, inset_rrect, &paint);
    }
}
*/
