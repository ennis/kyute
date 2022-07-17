//! Styling properties

use crate::{css, drawing, Color, LayoutConstraints};
use bitflags::bitflags;
use cssparser::{ParseError, Parser};
use once_cell::sync::Lazy;
use std::{convert::TryFrom, sync::Arc};

mod border;
mod box_shadow;
mod image;
mod length;
mod shape;
mod utils;

use crate::{
    css::{parse_from_str, parse_property_remainder},
    drawing::Paint,
};
pub use border::Border;
pub use box_shadow::{BoxShadow, BoxShadows};
pub use image::Image;
pub use length::{Length, LengthOrPercentage, UnitExt};
pub use shape::Shape;

bitflags! {
    /// Encodes the active visual states of a widget.
    #[derive(Default)]
    pub struct VisualState: u8 {
        /// Normal state.
        const DEFAULT  = 0;

        /// The widget has focus.
        ///
        /// Typically a border or a color highlight is drawn on the widget to signify the focused state.
        const FOCUS    = 1 << 0;

        /// The widget is "active" (e.g. pressed, for a button).
        const ACTIVE   = 1 << 1;

        /// A cursor is hovering atop the widget.
        const HOVER    = 1 << 2;

        /// The widget is disabled.
        ///
        /// Typically a widget is "greyed-out" when it is disabled.
        const DISABLED = 1 << 3;
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// Computed values
////////////////////////////////////////////////////////////////////////////////////////////////////

/*pub trait ToComputedValue {
    type ComputedValue;
    fn to_computed_value(&self, constraints: &LayoutConstraints) -> Self::ComputedValue;
}

impl ToComputedValue for Length {
    type ComputedValue = f64;

    fn to_computed_value(&self, constraints: &LayoutConstraints) -> f64 {
        match *self {
            Length::Px(x) => x / constraints.scale_factor,
            Length::Dip(x) => x,
            Length::Em(x) => x * constraints.parent_font_size,
            // FIXME: the reference length used for percentages depends on the property
            // This should be handled outside
            Length::Proportional(x) => x * constraints.max.width,
        }
    }
}

impl ToComputedValue for LengthOrPercentage {
    type ComputedValue = f64;

    fn to_computed_value(&self, constraints: &LayoutConstraints) -> f64 {
        match *self {
            LengthOrPercentage::Length(x) => x.to_computed_value(ctx, x),
            LengthOrPercentage::Percentage(x) => x * context.parent_length,
        }
    }
}*/

////////////////////////////////////////////////////////////////////////////////////////////////////
// Properties
////////////////////////////////////////////////////////////////////////////////////////////////////

/// Style property declaration.
#[derive(Clone, Debug)]
pub enum PropertyDeclaration {
    BorderBottomWidth(Length),
    BorderTopWidth(Length),
    BorderLeftWidth(Length),
    BorderRightWidth(Length),
    BorderTopLeftRadius(Length),
    BorderTopRightRadius(Length),
    BorderBottomRightRadius(Length),
    BorderBottomLeftRadius(Length),
    BorderBottomColor(Color),
    BorderTopColor(Color),
    BorderLeftColor(Color),
    BorderRightColor(Color),
    BorderImage(Image),
    BorderStyle(drawing::BorderStyle),
    BackgroundImage(Image),
    BackgroundColor(Color),
    BoxShadow(BoxShadows),
    MinWidth(LengthOrPercentage),
    MinHeight(LengthOrPercentage),
    MaxWidth(LengthOrPercentage),
    MaxHeight(LengthOrPercentage),
    Width(LengthOrPercentage),
    Height(LengthOrPercentage),
    PaddingLeft(LengthOrPercentage),
    PaddingRight(LengthOrPercentage),
    PaddingTop(LengthOrPercentage),
    PaddingBottom(LengthOrPercentage),
    FontSize(Length),
    RowGap(Length),
    ColumnGap(Length),
}

impl PropertyDeclaration {
    pub fn compute(&self, constraints: &LayoutConstraints, computed_values: &mut ComputedStyle) {
        match *self {
            PropertyDeclaration::BorderBottomWidth(specified) => {
                Arc::make_mut(&mut computed_values.border).border_bottom_width = specified.compute(&constraints);
            }
            PropertyDeclaration::BorderTopWidth(specified) => {
                Arc::make_mut(&mut computed_values.border).border_top_width = specified.compute(&constraints);
            }
            PropertyDeclaration::BorderLeftWidth(specified) => {
                Arc::make_mut(&mut computed_values.border).border_left_width = specified.compute(&constraints);
            }
            PropertyDeclaration::BorderRightWidth(specified) => {
                Arc::make_mut(&mut computed_values.border).border_right_width = specified.compute(&constraints);
            }
            PropertyDeclaration::BorderTopLeftRadius(specified) => {
                Arc::make_mut(&mut computed_values.border).border_top_left_radius = specified.compute(&constraints);
            }
            PropertyDeclaration::BorderTopRightRadius(specified) => {
                Arc::make_mut(&mut computed_values.border).border_top_right_radius = specified.compute(&constraints);
            }
            PropertyDeclaration::BorderBottomRightRadius(specified) => {
                Arc::make_mut(&mut computed_values.border).border_bottom_right_radius = specified.compute(&constraints);
            }
            PropertyDeclaration::BorderBottomLeftRadius(specified) => {
                Arc::make_mut(&mut computed_values.border).border_bottom_left_radius = specified.compute(&constraints);
            }
            PropertyDeclaration::BorderBottomColor(specified) => {
                Arc::make_mut(&mut computed_values.border).border_bottom_color = specified;
            }
            PropertyDeclaration::BorderTopColor(specified) => {
                Arc::make_mut(&mut computed_values.border).border_top_color = specified;
            }
            PropertyDeclaration::BorderLeftColor(specified) => {
                Arc::make_mut(&mut computed_values.border).border_left_color = specified;
            }
            PropertyDeclaration::BorderRightColor(specified) => {
                Arc::make_mut(&mut computed_values.border).border_right_color = specified;
            }
            PropertyDeclaration::BorderImage(ref specified) => {
                Arc::make_mut(&mut computed_values.border).border_image = specified.compute_paint();
            }
            PropertyDeclaration::BorderStyle(specified) => {
                Arc::make_mut(&mut computed_values.border).border_style = Some(specified);
            }
            PropertyDeclaration::BackgroundImage(ref specified) => {
                Arc::make_mut(&mut computed_values.background).background_image = specified.compute_paint();
            }
            PropertyDeclaration::BackgroundColor(ref specified) => {
                Arc::make_mut(&mut computed_values.background).background_color = specified.clone();
            }
            PropertyDeclaration::BoxShadow(ref specified) => {
                Arc::make_mut(&mut computed_values.box_shadow).box_shadows =
                    specified.into_iter().map(|x| x.compute(&constraints)).collect();
            }
            PropertyDeclaration::MinWidth(specified) => {
                // FIXME: if containing element is infinite, the value is ignored
                // TODO: finite_max_width may not be the value to use for %-lengths
                Arc::make_mut(&mut computed_values.layout).min_width = constraints
                    .finite_max_width()
                    .map(|w| specified.compute(constraints, w));
            }
            PropertyDeclaration::MinHeight(specified) => {
                Arc::make_mut(&mut computed_values.layout).min_height = constraints
                    .finite_max_height()
                    .map(|h| specified.compute(constraints, h));
            }
            PropertyDeclaration::MaxWidth(specified) => {
                Arc::make_mut(&mut computed_values.layout).max_width = constraints
                    .finite_max_width()
                    .map(|w| specified.compute(constraints, w));
            }
            PropertyDeclaration::MaxHeight(specified) => {
                Arc::make_mut(&mut computed_values.layout).max_height = constraints
                    .finite_max_height()
                    .map(|h| specified.compute(constraints, h));
            }
            PropertyDeclaration::Width(specified) => {
                Arc::make_mut(&mut computed_values.layout).width = constraints
                    .finite_max_width()
                    .map(|w| specified.compute(constraints, w));
            }
            PropertyDeclaration::Height(specified) => {
                Arc::make_mut(&mut computed_values.layout).height = constraints
                    .finite_max_height()
                    .map(|h| specified.compute(constraints, h));
            }
            PropertyDeclaration::PaddingLeft(specified) => {
                Arc::make_mut(&mut computed_values.layout).padding_left = constraints
                    .finite_max_width()
                    .map(|w| specified.compute(&constraints, w))
                    .unwrap_or(0.0);
            }
            PropertyDeclaration::PaddingRight(specified) => {
                Arc::make_mut(&mut computed_values.layout).padding_right = constraints
                    .finite_max_width()
                    .map(|w| specified.compute(&constraints, w))
                    .unwrap_or(0.0);
            }
            PropertyDeclaration::PaddingTop(specified) => {
                Arc::make_mut(&mut computed_values.layout).padding_top = constraints
                    .finite_max_height()
                    .map(|h| specified.compute(&constraints, h))
                    .unwrap_or(0.0);
            }
            PropertyDeclaration::PaddingBottom(specified) => {
                Arc::make_mut(&mut computed_values.layout).padding_bottom = constraints
                    .finite_max_height()
                    .map(|h| specified.compute(&constraints, h))
                    .unwrap_or(0.0);
            }
            PropertyDeclaration::FontSize(_specified) => {
                todo!()
            }
            PropertyDeclaration::RowGap(_specified) => {
                todo!()
            }
            PropertyDeclaration::ColumnGap(_specified) => {
                todo!()
            }
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// Styles
////////////////////////////////////////////////////////////////////////////////////////////////////

/// A set of style declarations, like:
///
///     border-width: 1px;
///     background: #fff;
///     border-radius: 10px;
///
///
#[derive(Clone)]
pub struct Style(Arc<StyleInner>);

struct StyleInner {
    declarations: Vec<PropertyDeclaration>,
}

static DEFAULT_STYLE: Lazy<Style> = Lazy::new(|| Style(Arc::new(StyleInner { declarations: vec![] })));

impl Default for Style {
    fn default() -> Self {
        DEFAULT_STYLE.clone()
    }
}

impl Style {
    /// Creates a new style block.
    pub fn new() -> Self {
        Style(Arc::new(StyleInner { declarations: vec![] }))
    }
}

impl Style {
    fn parse_impl<'i>(input: &mut Parser<'i, '_>) -> Result<Style, ParseError<'i, ()>> {
        let mut declarations = Vec::new();

        while !input.is_exhausted() {
            let prop_name = input.expect_ident()?.clone();
            input.expect_colon()?;
            match &*prop_name {
                "background" => {
                    let background = parse_property_remainder(input, Image::parse_impl)?;
                    declarations.push(PropertyDeclaration::BackgroundImage(background));
                }
                "border" => {
                    let border = parse_property_remainder(input, Border::parse_impl)?;
                    declarations.push(PropertyDeclaration::BorderStyle(border.line_style));
                    declarations.push(PropertyDeclaration::BorderLeftWidth(border.widths[0]));
                    declarations.push(PropertyDeclaration::BorderTopWidth(border.widths[1]));
                    declarations.push(PropertyDeclaration::BorderRightWidth(border.widths[2]));
                    declarations.push(PropertyDeclaration::BorderBottomWidth(border.widths[3]));
                    declarations.push(PropertyDeclaration::BorderLeftColor(border.color));
                    declarations.push(PropertyDeclaration::BorderTopColor(border.color));
                    declarations.push(PropertyDeclaration::BorderRightColor(border.color));
                    declarations.push(PropertyDeclaration::BorderBottomColor(border.color));
                }
                "border-radius" => {
                    let radii = parse_property_remainder(input, border::border_radius)?;
                    declarations.push(PropertyDeclaration::BorderTopLeftRadius(radii[0]));
                    declarations.push(PropertyDeclaration::BorderTopRightRadius(radii[1]));
                    declarations.push(PropertyDeclaration::BorderBottomRightRadius(radii[2]));
                    declarations.push(PropertyDeclaration::BorderBottomLeftRadius(radii[3]));
                }
                "box-shadow" => {
                    let box_shadows = parse_property_remainder(input, box_shadow::parse_box_shadows)?;
                    declarations.push(PropertyDeclaration::BoxShadow(box_shadows));
                }
                "padding" => {
                    let padding = parse_property_remainder(input, utils::padding)?;
                    declarations.push(PropertyDeclaration::PaddingTop(padding[0]));
                    declarations.push(PropertyDeclaration::PaddingRight(padding[1]));
                    declarations.push(PropertyDeclaration::PaddingBottom(padding[2]));
                    declarations.push(PropertyDeclaration::PaddingLeft(padding[3]));
                }
                "width" => {
                    let width = parse_property_remainder(input, css::parse_css_length_percentage)?;
                    declarations.push(PropertyDeclaration::Width(width));
                }
                "height" => {
                    let height = parse_property_remainder(input, css::parse_css_length_percentage)?;
                    declarations.push(PropertyDeclaration::Height(height));
                }
                "min-width" => {
                    let min_width = parse_property_remainder(input, css::parse_css_length_percentage)?;
                    declarations.push(PropertyDeclaration::MinWidth(min_width));
                }
                "min-height" => {
                    let min_height = parse_property_remainder(input, css::parse_css_length_percentage)?;
                    declarations.push(PropertyDeclaration::MinHeight(min_height));
                }
                "max-width" => {
                    let max_width = parse_property_remainder(input, css::parse_css_length_percentage)?;
                    declarations.push(PropertyDeclaration::MaxWidth(max_width));
                }
                "max-height" => {
                    let max_height = parse_property_remainder(input, css::parse_css_length_percentage)?;
                    declarations.push(PropertyDeclaration::MaxHeight(max_height));
                }
                _ => {
                    // unrecognized property
                    return Err(input.new_custom_error(()));
                }
            }
        }

        Ok(Style(Arc::new(StyleInner {
            //hash: None,
            declarations,
            //nested_rules: vec![],
        })))
    }

    pub fn parse(css: &str) -> Result<Self, ParseError<()>> {
        // for the ID, use the hash of the css source
        /*let source_hash = {
            let mut s = DefaultHasher::new();
            css.hash(&mut s);
            s.finish()
        };*/

        let style = parse_from_str(css, Self::parse_impl)?;
        //block_contents.hash = Some(source_hash);
        Ok(style)
    }

    pub fn compute(&self, constraints: &LayoutConstraints) -> ComputedStyle {
        let mut result = ComputedStyle::default();
        result.inherited.font_size = constraints.parent_font_size;
        for declaration in self.0.declarations.iter() {
            declaration.compute(constraints, &mut result);
        }
        result
    }
}

/// From CSS value.
impl TryFrom<&str> for Style {
    type Error = ();
    fn try_from(css: &str) -> Result<Self, ()> {
        Style::parse(css).map_err(|_| ())
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// Computed properties
////////////////////////////////////////////////////////////////////////////////////////////////////

/// Calculated background properties.
#[derive(Clone, Debug, Default)]
pub struct BackgroundProperties {
    pub background_image: Paint,
    pub background_color: Color,
}

/// Calculated box-shadow properties.
#[derive(Clone, Debug)]
pub struct BoxShadowProperties {
    pub box_shadows: Vec<drawing::BoxShadow>,
}

impl Default for BoxShadowProperties {
    fn default() -> Self {
        BoxShadowProperties { box_shadows: vec![] }
    }
}

/// Calculated box-shadow properties.
#[derive(Clone, Debug)]
pub struct BorderProperties {
    pub border_bottom_width: f64,
    pub border_top_width: f64,
    pub border_left_width: f64,
    pub border_right_width: f64,
    pub border_top_left_radius: f64,
    pub border_top_right_radius: f64,
    pub border_bottom_right_radius: f64,
    pub border_bottom_left_radius: f64,
    pub border_bottom_color: Color,
    pub border_top_color: Color,
    pub border_left_color: Color,
    pub border_right_color: Color,
    pub border_image: Paint,
    pub border_style: Option<drawing::BorderStyle>,
}

impl Default for BorderProperties {
    fn default() -> Self {
        BorderProperties {
            border_bottom_width: 0.0,
            border_top_width: 0.0,
            border_left_width: 0.0,
            border_right_width: 0.0,
            border_top_left_radius: 0.0,
            border_top_right_radius: 0.0,
            border_bottom_right_radius: 0.0,
            border_bottom_left_radius: 0.0,
            border_bottom_color: Default::default(),
            border_top_color: Default::default(),
            border_left_color: Default::default(),
            border_right_color: Default::default(),
            border_image: Paint::Color(Color::default()),
            border_style: Default::default(),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct LayoutProperties {
    pub top: Option<f64>,
    pub right: Option<f64>,
    pub bottom: Option<f64>,
    pub left: Option<f64>,
    pub z_index: f64,
    //pub flex_direction: f64,
    //pub flex_wrap: f64,
    //pub justify_content: f64,
    //pub align_content: f64,
    //pub align_items: f64,
    //pub flex_grow: f64,
    //pub flex_shrink: f64,
    pub align_self: f64,
    pub order: f64,
    pub flex_basis: f64,
    pub width: Option<f64>,
    pub min_width: Option<f64>,
    pub max_width: Option<f64>,
    pub height: Option<f64>,
    pub min_height: Option<f64>,
    pub max_height: Option<f64>,
    pub aspect_ratio: f64,
    pub padding_top: f64,
    pub padding_right: f64,
    pub padding_bottom: f64,
    pub padding_left: f64,
}

#[derive(Clone, Debug, Default)]
pub struct InheritedProperties {
    pub font_size: f64,
}

/// A set of calculated style properties.
#[derive(Clone, Debug)]
pub struct ComputedStyle {
    hash: Option<u64>,
    pub box_shadow: Arc<BoxShadowProperties>,
    pub background: Arc<BackgroundProperties>,
    pub border: Arc<BorderProperties>,
    pub layout: Arc<LayoutProperties>,
    pub inherited: InheritedProperties,
}

static DEFAULT_BOX_SHADOW_PROPERTIES: Lazy<Arc<BoxShadowProperties>> =
    Lazy::new(|| Arc::new(BoxShadowProperties::default()));
// `NonNull<skia_bindings::bindings::SkRuntimeEffect>` cannot be sent between threads safely
//static DEFAULT_BACKGROUND_PROPERTIES: Lazy<Arc<BackgroundProperties>> =
//    Lazy::new(|| Arc::new(BackgroundProperties::default()));
//static DEFAULT_BORDER_PROPERTIES: Lazy<Arc<BorderProperties>> = Lazy::new(|| Arc::new(BorderProperties::default()));
static DEFAULT_POSITION_PROPERTIES: Lazy<Arc<LayoutProperties>> = Lazy::new(|| Arc::new(LayoutProperties::default()));

impl Default for ComputedStyle {
    fn default() -> Self {
        ComputedStyle {
            hash: Some(0),
            box_shadow: DEFAULT_BOX_SHADOW_PROPERTIES.clone(),
            background: Arc::new(BackgroundProperties::default()),
            border: Arc::new(BorderProperties::default()),
            //background: DEFAULT_BACKGROUND_PROPERTIES.clone(),
            //border: DEFAULT_BORDER_PROPERTIES.clone(),
            layout: DEFAULT_POSITION_PROPERTIES.clone(),
            inherited: InheritedProperties { font_size: 16.0 },
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// Style
////////////////////////////////////////////////////////////////////////////////////////////////////

/*/// Style of a container.
#[derive(Clone, Debug)]
pub struct Style {
    pub border_radii: [Length; 4],
    pub border: Option<Border>,
    pub background: Option<Paint>,
    pub box_shadows: Vec<BoxShadow>,
}

impl Default for Style {
    fn default() -> Self {
        Style::new()
    }
}

impl Style {
    pub fn new() -> Style {
        Style {
            border_radii: [Length::Dip(0.0); 4],
            background: None,
            border: None,
            box_shadows: vec![],
        }
    }

    ///
    pub fn is_transparent(&self) -> bool {
        self.background.is_none() && self.border.is_none() && self.box_shadows.is_empty()
    }

    pub fn clip_rect(&self, bounds: Rect, scale_factor: f64) -> Rect {
        let mut rect = bounds;
        for box_shadow in self.box_shadows.iter() {
            if !box_shadow.inset {
                let mut shadow_rect = bounds;
                shadow_rect.origin.x += box_shadow.x_offset.to_dips(scale_factor, bounds.width());
                shadow_rect.origin.y += box_shadow.y_offset.to_dips(scale_factor, bounds.height());
                let spread = box_shadow.spread.to_dips(scale_factor, bounds.width());
                let radius = box_shadow.blur.to_dips(scale_factor, bounds.width());
                shadow_rect = shadow_rect.inflate(spread + radius, spread + radius);
                rect = rect.union(&shadow_rect);
            }
        }
        rect
    }

    /// Specifies the radius of the 4 corners of the box.
    pub fn radius(mut self, radius: impl Into<Length>) -> Self {
        let radius = radius.into();
        self.border_radii = [radius; 4];
        self
    }

    /// Specifies the radius of each corner of the box separately.
    pub fn radii(
        mut self,
        top_left: impl Into<Length>,
        top_right: impl Into<Length>,
        bottom_right: impl Into<Length>,
        bottom_left: impl Into<Length>,
    ) -> Self {
        self.border_radii = [
            top_left.into(),
            top_right.into(),
            bottom_right.into(),
            bottom_left.into(),
        ];
        self
    }

    /// Sets the brush used to fill the rectangle.
    pub fn background(mut self, paint: impl Into<Paint>) -> Self {
        self.background = Some(paint.into());
        self
    }

    /// Sets the border.
    pub fn border(mut self, border: Border) -> Self {
        self.border = Some(border);
        self
    }

    /// Adds a box shadow.
    pub fn box_shadow(mut self, box_shadow: BoxShadow) -> Self {
        self.box_shadows.push(box_shadow);
        self
    }

    /// Draws a box with this style in the given bounds.
    pub fn draw(&self, ctx: &mut PaintCtx, bounds: Rect) {
        let radii = radii_to_skia(ctx, bounds, &self.border_radii);
        let canvas = ctx.surface.canvas();

        // --- box shadows ---
        // TODO move in own function
        for box_shadow in self.box_shadows.iter() {
            let x_offset = box_shadow.x_offset.to_dips(ctx.scale_factor, bounds.size.width);
            let y_offset = box_shadow.y_offset.to_dips(ctx.scale_factor, bounds.size.height);
            let offset = Offset::new(x_offset, y_offset);
            let blur = box_shadow.blur.to_dips(ctx.scale_factor, bounds.size.width);
            let spread = box_shadow.spread.to_dips(ctx.scale_factor, bounds.size.width);
            let color = box_shadow.color;

            // setup skia paint (mask blur)
            let mut shadow_paint = sk::Paint::default();
            shadow_paint.set_mask_filter(sk::MaskFilter::blur(
                sk::BlurStyle::Normal,
                blur_radius_to_std_dev(blur),
                None,
            ));
            shadow_paint.set_color(color.to_skia().to_color());

            if !box_shadow.inset {
                // drop shadow
                // calculate base shadow shape rectangle (apply offset & spread)
                let mut rect = bounds.translate(offset).inflate(spread, spread);
                // TODO adjust radius
                let rrect = sk::RRect::new_rect_radii(rect.to_skia(), &radii);
                canvas.draw_rrect(rrect, &shadow_paint);
            } else {
                let inner_rect = bounds.translate(offset).inflate(-spread, -spread);
                let outer_rect = area_casting_shadow_in_hole(bounds, offset, blur, spread);
                // TODO adjust radius
                let inner_rrect = sk::RRect::new_rect_radii(inner_rect.to_skia(), &radii);
                let outer_rrect = sk::RRect::new_rect_radii(outer_rect.to_skia(), &radii);
                canvas.draw_drrect(outer_rrect, inner_rrect, &shadow_paint);
            }
        }

        // --- background ---
        if let Some(ref brush) = self.background {
            let mut paint = brush.to_sk_paint(bounds);
            paint.set_style(sk::PaintStyle::Fill);
            let rrect = sk::RRect::new_rect_radii(bounds.to_skia(), &radii);
            ctx.surface.canvas().draw_rrect(rrect, &paint);
        }

        // --- border ---
        if let Some(ref border) = self.border {
            border.draw(ctx, bounds, radii);
        }
    }
}

impl_env_value!(Style);*/
