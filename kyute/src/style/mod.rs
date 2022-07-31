//! Styling properties

use crate::{css, drawing, LayoutParams};
use bitflags::bitflags;
use cssparser::{ParseError, Parser, Token};
use once_cell::sync::Lazy;
use std::{convert::TryFrom, sync::Arc};

mod border;
mod box_shadow;
mod color;
mod image;
mod length;
mod predicate;
mod shape;
mod utils;

use crate::{
    css::{parse_from_str, parse_property_remainder},
    drawing::Paint,
};
pub use border::Border;
pub use box_shadow::{BoxShadow, BoxShadows};
pub use color::Color;
pub use image::Image;
use kyute::Environment;
use kyute_common::Atom;
pub use length::{Length, LengthOrPercentage, UnitExt};
use predicate::{parse_predicate, Predicate, Pseudoclass};
pub use shape::Shape;

bitflags! {
    /// Encodes the active states of a widget.
    #[derive(Default)]
    pub struct WidgetState: u8 {
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
    pub fn compute(&self, constraints: &LayoutParams, env: &Environment, computed_values: &mut ComputedStyle) {
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
            PropertyDeclaration::BorderBottomColor(ref specified) => {
                Arc::make_mut(&mut computed_values.border).border_bottom_color = specified.compute(env);
            }
            PropertyDeclaration::BorderTopColor(ref specified) => {
                Arc::make_mut(&mut computed_values.border).border_top_color = specified.compute(env);
            }
            PropertyDeclaration::BorderLeftColor(ref specified) => {
                Arc::make_mut(&mut computed_values.border).border_left_color = specified.compute(env);
            }
            PropertyDeclaration::BorderRightColor(ref specified) => {
                Arc::make_mut(&mut computed_values.border).border_right_color = specified.compute(env);
            }
            PropertyDeclaration::BorderImage(ref specified) => {
                Arc::make_mut(&mut computed_values.border).border_image = specified.compute_paint(env);
            }
            PropertyDeclaration::BorderStyle(specified) => {
                Arc::make_mut(&mut computed_values.border).border_style = Some(specified);
            }
            PropertyDeclaration::BackgroundImage(ref specified) => {
                Arc::make_mut(&mut computed_values.background).background_image = specified.compute_paint(env);
            }
            PropertyDeclaration::BackgroundColor(ref specified) => {
                Arc::make_mut(&mut computed_values.background).background_color = specified.clone();
            }
            PropertyDeclaration::BoxShadow(ref specified) => {
                Arc::make_mut(&mut computed_values.box_shadow).box_shadows =
                    specified.into_iter().map(|x| x.compute(&constraints, env)).collect();
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
#[derive(Clone)]
pub struct Style(Arc<StyleInner>);

struct StyleInner {
    /// State bits that this style depends on.
    variant_states: WidgetState,
    declarations: Vec<PredicatedPropertyDeclaration>,
}

struct PredicatedPropertyDeclaration {
    predicate: Option<Arc<Predicate>>,
    declaration: PropertyDeclaration,
}

static DEFAULT_STYLE: Lazy<Style> = Lazy::new(|| {
    Style(Arc::new(StyleInner {
        variant_states: WidgetState::DEFAULT,
        declarations: vec![],
    }))
});

impl Default for Style {
    fn default() -> Self {
        DEFAULT_STYLE.clone()
    }
}

impl Style {
    /// Creates a new style block.
    pub fn new() -> Self {
        Style(Arc::new(StyleInner {
            variant_states: WidgetState::DEFAULT,
            declarations: vec![],
        }))
    }

    pub fn variant_states(&self) -> WidgetState {
        self.0.variant_states
    }
}

impl Style {
    fn parse_impl<'i>(input: &mut Parser<'i, '_>) -> Result<Style, ParseError<'i, ()>> {
        let mut declarations = Vec::new();
        let mut variant_states = WidgetState::DEFAULT;

        while !input.is_exhausted() {
            let predicate = if input.try_parse(|input| input.expect_square_bracket_block()).is_ok() {
                let predicate = input.parse_nested_block(|input| {
                    input.expect_ident_matching("if")?;
                    parse_predicate(input)
                })?;
                variant_states |= predicate.variant_states();
                Some(Arc::new(predicate))
            } else {
                None
            };

            let prop_name = input.expect_ident()?.clone();
            input.expect_colon()?;

            let mut push_decl = |declaration| {
                declarations.push(PredicatedPropertyDeclaration {
                    predicate: predicate.clone(),
                    declaration,
                })
            };

            match &*prop_name {
                "background" => {
                    let background = parse_property_remainder(input, Image::parse_impl)?;
                    push_decl(PropertyDeclaration::BackgroundImage(background));
                }
                "border" => {
                    let border = parse_property_remainder(input, Border::parse_impl)?;
                    push_decl(PropertyDeclaration::BorderStyle(border.line_style));
                    push_decl(PropertyDeclaration::BorderTopWidth(border.widths[0]));
                    push_decl(PropertyDeclaration::BorderRightWidth(border.widths[1]));
                    push_decl(PropertyDeclaration::BorderBottomWidth(border.widths[2]));
                    push_decl(PropertyDeclaration::BorderLeftWidth(border.widths[3]));
                    push_decl(PropertyDeclaration::BorderLeftColor(border.color.clone()));
                    push_decl(PropertyDeclaration::BorderTopColor(border.color.clone()));
                    push_decl(PropertyDeclaration::BorderRightColor(border.color.clone()));
                    push_decl(PropertyDeclaration::BorderBottomColor(border.color.clone()));
                }
                "border-radius" => {
                    let radii = parse_property_remainder(input, border::border_radius)?;
                    push_decl(PropertyDeclaration::BorderTopLeftRadius(radii[0]));
                    push_decl(PropertyDeclaration::BorderTopRightRadius(radii[1]));
                    push_decl(PropertyDeclaration::BorderBottomRightRadius(radii[2]));
                    push_decl(PropertyDeclaration::BorderBottomLeftRadius(radii[3]));
                }
                "box-shadow" => {
                    let box_shadows = parse_property_remainder(input, box_shadow::parse_box_shadows)?;
                    push_decl(PropertyDeclaration::BoxShadow(box_shadows));
                }
                "padding" => {
                    let padding = parse_property_remainder(input, utils::padding)?;
                    push_decl(PropertyDeclaration::PaddingTop(padding[0]));
                    push_decl(PropertyDeclaration::PaddingRight(padding[1]));
                    push_decl(PropertyDeclaration::PaddingBottom(padding[2]));
                    push_decl(PropertyDeclaration::PaddingLeft(padding[3]));
                }
                "width" => {
                    let width = parse_property_remainder(input, css::parse_css_length_percentage)?;
                    push_decl(PropertyDeclaration::Width(width));
                }
                "height" => {
                    let height = parse_property_remainder(input, css::parse_css_length_percentage)?;
                    push_decl(PropertyDeclaration::Height(height));
                }
                "min-width" => {
                    let min_width = parse_property_remainder(input, css::parse_css_length_percentage)?;
                    push_decl(PropertyDeclaration::MinWidth(min_width));
                }
                "min-height" => {
                    let min_height = parse_property_remainder(input, css::parse_css_length_percentage)?;
                    push_decl(PropertyDeclaration::MinHeight(min_height));
                }
                "max-width" => {
                    let max_width = parse_property_remainder(input, css::parse_css_length_percentage)?;
                    push_decl(PropertyDeclaration::MaxWidth(max_width));
                }
                "max-height" => {
                    let max_height = parse_property_remainder(input, css::parse_css_length_percentage)?;
                    push_decl(PropertyDeclaration::MaxHeight(max_height));
                }
                _ => {
                    // unrecognized property
                    return Err(input.new_custom_error(()));
                }
            }
        }

        Ok(Style(Arc::new(StyleInner {
            variant_states,
            declarations,
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

    pub fn compute(&self, widget_state: WidgetState, constraints: &LayoutParams, env: &Environment) -> ComputedStyle {
        let mut result = ComputedStyle::default();
        result.inherited.font_size = constraints.parent_font_size;
        for declaration in self.0.declarations.iter() {
            if declaration
                .predicate
                .as_ref()
                .map(|pred| pred.eval(widget_state, constraints, env))
                .unwrap_or(true)
            {
                declaration.declaration.compute(constraints, env, &mut result);
            }
        }
        result
    }
}

/// From CSS value.
impl TryFrom<&str> for Style {
    type Error = ();
    fn try_from(css: &str) -> Result<Self, ()> {
        Style::parse(css).map_err(|err| {
            warn!("CSS syntax error: {:?}", err);
            ()
        })
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
    pub border_bottom_color: crate::Color,
    pub border_top_color: crate::Color,
    pub border_left_color: crate::Color,
    pub border_right_color: crate::Color,
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
            border_image: Paint::Color(Default::default()),
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
