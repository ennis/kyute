//! Drawing code for GUI elements.
mod utils;
pub mod values;

use crate::{Color, EnvRef, Length, Offset, Rect, RectExt, UnitExt};
use cssparser::{ParseError, Parser};
use kyute_common::{LengthOrPercentage, Size};
use std::{
    cell::RefCell,
    collections::{hash_map::DefaultHasher, HashMap},
    convert::{TryFrom, TryInto},
    hash::{Hash, Hasher},
    marker::PhantomData,
    sync::Arc,
};

use crate::{
    style::{
        utils::{parse_from_str, parse_property_remainder},
        values::box_shadow::BoxShadow,
    },
    LengthOrPercentage,
};

////////////////////////////////////////////////////////////////////////////////////////////////////
// Style properties & rules
////////////////////////////////////////////////////////////////////////////////////////////////////

/// Error emitted when a style property value has an unexpected type.
#[derive(Copy, Clone, Debug, thiserror::Error)]
pub struct PropertyValueTypeMismatch;

pub struct StyleCache {
    defaults: Arc<ComputedValues>,
    cache: RefCell<HashMap<u64, Arc<ComputedValues>>>,
}

impl StyleCache {
    pub fn new() -> StyleCache {
        StyleCache {
            defaults: Arc::new(ComputedValues::default()),
            cache: RefCell::new(HashMap::new()),
        }
    }
}

/// Context used to calculate the final value ("used value") of a property.
#[derive(Clone, Debug)]
pub struct StyleCtx {
    /// Scale factor (pixel density ratio) of the target surface.
    pub scale_factor: f64,
    pub base_width: f64,
    pub base_height: f64,
    pub font_size: f64,
}

pub trait ToComputedValue {
    type ComputedValue;
    fn to_computed_value(&self, context: &StyleCtx) -> Self::ComputedValue;
}

impl ToComputedValue for Length {
    type ComputedValue = f64;

    fn to_computed_value(&self, context: &StyleCtx) -> f64 {
        match *self {
            Length::Px(x) => x / context.scale_factor,
            Length::Dip(x) => x,
            Length::Em(x) => x * context.font_size,
        }
    }
}

impl ToComputedValue for LengthOrPercentage {
    type ComputedValue = f64;

    fn to_computed_value(&self, context: &StyleCtx) -> f64 {
        match *self {
            LengthOrPercentage::Length(x) => x.to_computed_value(ctx, x),
            LengthOrPercentage::Percentage(x) => x * context.parent_length,
        }
    }
}

/// Calculated background properties.
#[derive(Clone, Debug)]
pub struct BackgroundProperties {
    pub background_image: values::image::Image,
    pub background_color: Color,
}

/// Calculated box-shadow properties.
#[derive(Clone, Debug)]
pub struct BoxShadowProperties {
    pub box_shadows: Vec<values::box_shadow::ComputedBoxShadow>,
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
    pub border_image: values::image::Image,
    pub border_style: values::border::BorderStyle,
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
            border_image: values::image::Image::Color(Color::default()),
            border_style: Default::default(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct PositionProperties {
    pub top: f64,
    pub right: f64,
    pub bottom: f64,
    pub left: f64,
    pub z_index: f64,
    pub flex_direction: f64,
    pub flex_wrap: f64,
    pub justify_content: f64,
    pub align_content: f64,
    pub align_items: f64,
    pub flex_grow: f64,
    pub flex_shrink: f64,
    pub align_self: f64,
    pub order: f64,
    pub flex_basis: f64,
    pub width: f64,
    pub min_width: f64,
    pub max_width: f64,
    pub height: f64,
    pub min_height: f64,
    pub max_height: f64,
    pub row_gap: f64,
    pub column_gap: f64,
    pub aspect_ratio: f64,
}

#[derive(Clone, Debug, Default)]
pub struct GridProperties {
    pub template_rows: values::grid::ComputedTrackList,
    pub template_columns: values::grid::ComputedTrackList,
    pub row_gap: f64,
    pub column_gap: f64,
}

#[derive(Clone, Debug, Default)]
pub struct GridPositionProperties {
    pub row_start: values::grid::Line,
    pub row_end: values::grid::Line,
    pub column_start: values::grid::Line,
    pub column_end: values::grid::Line,
}

/// A set of calculated style properties.
#[derive(Clone, Debug)]
pub struct ComputedValues {
    hash: Option<u64>,
    pub box_shadow: Arc<BoxShadowProperties>,
    pub background: Arc<BackgroundProperties>,
    pub border: Arc<BorderProperties>,
    pub grid: Arc<GridProperties>,
    pub grid_position: Arc<GridPositionProperties>,
    pub position: Arc<PositionProperties>,
}

impl Default for ComputedValues {
    fn default() -> Self {
        ComputedValues {
            hash: Some(0),
            box_shadow: Arc::new(BoxShadowProperties::default()),
            background: Arc::new(BackgroundProperties::default()),
            border: Arc::new(BorderProperties::default()),
            grid: Arc::new(GridProperties::default()),
            grid_position: Arc::new(GridPositionProperties::default()),
        }
    }
}

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
    BorderImage(values::image::Image),
    BorderStyle(values::border::BorderStyle),
    BackgroundImage(values::image::Image),
    BackgroundColor(Color),
    BoxShadow(values::box_shadow::BoxShadows),
    MinWidth(Length),
    MinHeight(Length),
    MaxWidth(Length),
    MaxHeight(Length),
    Width(Length),
    Height(Length),
    PaddingLeft(Length),
    PaddingRight(Length),
    PaddingTop(Length),
    PaddingBottom(Length),
    FontSize(Length),
    GridRowStart(values::grid::Line),
    GridRowEnd(values::grid::Line),
    GridColumnStart(values::grid::Line),
    GridColumnEnd(values::grid::Line),
    GridTemplateRows(values::grid::TrackList),
    GridTemplateColumns(values::grid::TrackList),
    RowGap(Length),
    ColumnGap(Length),
}

impl PropertyDeclaration {
    pub fn compute(&self, context: &StyleCtx, computed_values: &mut ComputedValues) {
        match self {
            PropertyDeclaration::BorderBottomWidth(specified) => {
                Arc::make_mut(&mut computed_values.border).border_bottom_width = specified.to_computed_value(context);
            }
            PropertyDeclaration::BorderTopWidth(specified) => {
                Arc::make_mut(&mut computed_values.border).border_top_width = specified.to_computed_value(context);
            }
            PropertyDeclaration::BorderLeftWidth(specified) => {
                Arc::make_mut(&mut computed_values.border).border_left_width = specified.to_computed_value(context);
            }
            PropertyDeclaration::BorderRightWidth(specified) => {
                Arc::make_mut(&mut computed_values.border).border_right_width = specified.to_computed_value(context);
            }
            PropertyDeclaration::BorderTopLeftRadius(specified) => {
                Arc::make_mut(&mut computed_values.border).border_top_left_radius =
                    specified.to_computed_value(context);
            }
            PropertyDeclaration::BorderTopRightRadius(specified) => {
                Arc::make_mut(&mut computed_values.border).border_top_right_radius =
                    specified.to_computed_value(context);
            }
            PropertyDeclaration::BorderBottomRightRadius(specified) => {
                Arc::make_mut(&mut computed_values.border).border_bottom_right_radius =
                    specified.to_computed_value(context);
            }
            PropertyDeclaration::BorderBottomLeftRadius(specified) => {
                Arc::make_mut(&mut computed_values.border).border_bottom_left_radius =
                    specified.to_computed_value(context);
            }
            PropertyDeclaration::BorderBottomColor(specified) => {
                todo!()
            }
            PropertyDeclaration::BorderTopColor(specified) => {
                todo!()
            }
            PropertyDeclaration::BorderLeftColor(specified) => {
                todo!()
            }
            PropertyDeclaration::BorderRightColor(specified) => {
                todo!()
            }
            PropertyDeclaration::BorderImage(specified) => {
                Arc::make_mut(&mut computed_values.border).border_image = specified.clone();
            }
            PropertyDeclaration::BorderStyle(specified) => {}
            PropertyDeclaration::BackgroundImage(specified) => {
                Arc::make_mut(&mut computed_values.background).background_image = specified.clone();
            }
            PropertyDeclaration::BackgroundColor(specified) => {
                Arc::make_mut(&mut computed_values.background).background_color = specified.clone();
            }
            PropertyDeclaration::BoxShadow(specified) => {
                Arc::make_mut(&mut computed_values.box_shadow).box_shadows = specified.to_computed_value(context);
            }
            PropertyDeclaration::MinWidth(specified) => {
                todo!()
            }
            PropertyDeclaration::MinHeight(specified) => {
                todo!()
            }
            PropertyDeclaration::MaxWidth(specified) => {
                todo!()
            }
            PropertyDeclaration::MaxHeight(specified) => {
                todo!()
            }
            PropertyDeclaration::Width(specified) => {
                todo!()
            }
            PropertyDeclaration::Height(specified) => {
                todo!()
            }
            PropertyDeclaration::PaddingLeft(specified) => {
                todo!()
            }
            PropertyDeclaration::PaddingRight(specified) => {
                todo!()
            }
            PropertyDeclaration::PaddingTop(specified) => {
                todo!()
            }
            PropertyDeclaration::PaddingBottom(specified) => {
                todo!()
            }
            PropertyDeclaration::FontSize(specified) => {
                todo!()
            }
            PropertyDeclaration::GridRowStart(specified) => {
                Arc::make_mut(&mut computed_values.grid_position).row_start = specified.clone();
            }
            PropertyDeclaration::GridRowEnd(specified) => {
                Arc::make_mut(&mut computed_values.grid_position).row_end = specified.clone();
            }
            PropertyDeclaration::GridColumnStart(specified) => {
                Arc::make_mut(&mut computed_values.grid_position).column_start = specified.clone();
            }
            PropertyDeclaration::GridColumnEnd(specified) => {
                Arc::make_mut(&mut computed_values.grid_position).column_end = specified.clone();
            }
            PropertyDeclaration::GridTemplateRows(specified) => {
                Arc::make_mut(&mut computed_values.grid).template_rows = specified.to_computed_value(context);
            }
            PropertyDeclaration::GridTemplateColumns(specified) => {
                Arc::make_mut(&mut computed_values.grid).template_columns = specified.to_computed_value(context);
            }
            PropertyDeclaration::RowGap(specified) => {
                Arc::make_mut(&mut computed_values.grid).row_gap = specified.to_computed_value(context);
            }
            PropertyDeclaration::ColumnGap(specified) => {
                Arc::make_mut(&mut computed_values.grid).column_gap = specified.to_computed_value(context);
            }
        }
    }
}

fn chain_computed_values_hash(parent_hash: u64, rule_id: u64) -> u64 {
    let mut hasher = DefaultHasher::new();
    hasher.write_u64(parent_hash);
    hasher.write_u64(rule_id);
    hasher.finish()
}

impl ComputedValues {
    fn inherited(&self, cache: &StyleCache) -> ComputedValues {
        ComputedValues {
            hash: self.hash,
            box_shadow: cache.defaults.box_shadow.clone(),
            background: cache.defaults.background.clone(),
            border: cache.defaults.border.clone(),
            grid: cache.defaults.grid.clone(),
            grid_position: cache.defaults.grid_position.clone(),
        }
    }

    pub fn compute(
        cache: &StyleCache,
        parent_style: &ComputedValues,
        rules: &[&StyleRule],
        context: &StyleCtx,
    ) -> Arc<ComputedValues> {
        // compute the hash
        let mut hash = parent_style.hash;
        if hash.is_some() {
            for rule in rules.iter() {
                if let Some(rule_hash) = rule.contents.hash {
                    hash = Some(chain_computed_values_hash(hash.unwrap(), rule_hash));
                } else {
                    hash = None;
                    break;
                }
            }
        }

        if let Some(hash) = hash {
            // try to find cached values
            if let Some(values) = cache.cache.borrow().get(&hash) {
                return values.clone();
            }
        }

        // compute
        let mut computed = parent_style.inherited(cache);
        for rule in rules.iter() {
            for decl in rule.contents.declarations.iter() {
                decl.compute(context, &mut computed);
            }
        }

        let computed = Arc::new(computed);

        // store in cache
        if let Some(hash) = hash {
            cache.cache.borrow_mut().insert(hash, computed.clone());
        }

        computed
    }
}

/// CSS selector.
pub struct Selector {}

/// CSS rule.
pub struct StyleRule {
    pub selector: Selector,
    pub contents: StyleBlockContents,
}

impl StyleRule {
    pub fn inline(contents: StyleBlockContents) -> StyleRule {
        StyleRule {
            selector: Selector {},
            contents,
        }
    }
}

/// CSS declaration block, possibly with nested rules.
pub struct StyleBlockContents {
    /// Hash of the CSS source.
    hash: Option<u64>,
    pub declarations: Vec<PropertyDeclaration>,
    pub nested_rules: Vec<StyleRule>,
}

impl Default for StyleBlockContents {
    fn default() -> Self {
        StyleBlockContents {
            hash: None,
            declarations: vec![],
            nested_rules: vec![],
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// Parsers
////////////////////////////////////////////////////////////////////////////////////////////////////

impl StyleBlockContents {
    fn parse_impl<'i>(input: &mut Parser<'i, '_>) -> Result<StyleBlockContents, ParseError<'i, ()>> {
        let mut properties = Vec::new();

        while !input.is_exhausted() {
            let prop_name = input.expect_ident()?.clone();
            input.expect_colon()?;
            match &*prop_name {
                "background" => {
                    let background = parse_property_remainder(input, values::image::Image::parse_impl)?;
                    properties.push(PropertyDeclaration::BackgroundImage(background));
                }
                "border" => {
                    let border = parse_property_remainder(input, values::border::Border::parse_impl)?;
                    properties.push(PropertyDeclaration::BorderLeftWidth(border.width));
                    properties.push(PropertyDeclaration::BorderTopWidth(border.width));
                    properties.push(PropertyDeclaration::BorderRightWidth(border.width));
                    properties.push(PropertyDeclaration::BorderLeftWidth(border.width));
                    properties.push(PropertyDeclaration::BorderLeftColor(border.color));
                    properties.push(PropertyDeclaration::BorderTopColor(border.color));
                    properties.push(PropertyDeclaration::BorderRightColor(border.color));
                    properties.push(PropertyDeclaration::BorderBottomColor(border.color));
                }
                "border-radius" => {
                    let radii = parse_property_remainder(input, values::border::border_radius)?;
                    properties.push(PropertyDeclaration::BorderTopLeftRadius(radii[0]));
                    properties.push(PropertyDeclaration::BorderTopRightRadius(radii[1]));
                    properties.push(PropertyDeclaration::BorderBottomRightRadius(radii[2]));
                    properties.push(PropertyDeclaration::BorderBottomLeftRadius(radii[3]));
                }
                "box-shadow" => {
                    let box_shadows =
                        parse_property_remainder(input, |input| input.parse_comma_separated(BoxShadow::parse_impl))?;
                    properties.push(PropertyDeclaration::BoxShadow(box_shadows));
                }
                "grid-template" => {
                    let template = parse_property_remainder(input, values::grid::Template::parse_impl)?;
                    properties.push(PropertyDeclaration::GridTemplateRows(template.rows));
                    properties.push(PropertyDeclaration::GridTemplateColumns(template.columns));
                }
                "grid-area" => {
                    let area = parse_property_remainder(input, values::grid::Area::parse_impl)?;
                    properties.push(PropertyDeclaration::GridRowStart(area.row.start));
                    properties.push(PropertyDeclaration::GridRowEnd(area.row.end));
                    properties.push(PropertyDeclaration::GridColumnStart(area.column.start));
                    properties.push(PropertyDeclaration::GridColumnEnd(area.column.end));
                }
                "row-gap" => {
                    let row_gap = parse_property_remainder(input, values::length::length)?;
                    properties.push(PropertyDeclaration::RowGap(row_gap));
                }
                "column-gap" => {
                    let column_gap = parse_property_remainder(input, values::length::length)?;
                    properties.push(PropertyDeclaration::ColumnGap(column_gap));
                }
                _ => {
                    // unrecognized property
                    return Err(input.new_custom_error(()));
                }
            }
        }

        Ok(StyleBlockContents {
            hash: None,
            declarations: properties,
            nested_rules: vec![],
        })
    }

    pub fn parse(css: &str) -> Result<Self, ParseError<()>> {
        // for the ID, use the hash of the css source
        let source_hash = {
            let mut s = DefaultHasher::new();
            css.hash(&mut s);
            s.finish()
        };

        let mut block_contents = parse_from_str(css, Self::parse_impl)?;
        block_contents.hash = Some(source_hash);
        Ok(block_contents)
    }
}

//--------------------------------------------------------------------------------------------------

/*/// Path visual.
pub struct Path {
    path: sk::Path,
    stroke: Option<Paint>,
    fill: Option<Paint>,
    box_shadow: Option<BoxShadow>,
}

impl Path {
    pub fn new(svg_path: &str) -> Path {
        Path {
            path: svg_path_to_skia(svg_path).expect("invalid path syntax"),
            stroke: None,
            fill: None,
            box_shadow: None,
        }
    }

    /// Sets the brush used to fill the path.
    pub fn fill(mut self, paint: impl Into<Paint>) -> Self {
        self.fill = Some(paint.into());
        self
    }

    /// Sets the brush used to stroke the path.
    pub fn stroke(mut self, paint: impl Into<Paint>) -> Self {
        self.fill = Some(paint.into());
        self
    }

    pub fn draw(&self, ctx: &mut PaintCtx, bounds: Rect) {
        // fill
        let canvas = ctx.surface.canvas();
        if let Some(ref brush) = self.fill {
            let mut paint = brush.to_sk_paint(bounds);
            paint.set_style(sk::PaintStyle::Fill);
            canvas.save();
            canvas.translate(bounds.top_left().to_skia());
            canvas.draw_path(&self.path, &paint);
            canvas.restore();
        }

        // stroke
        if let Some(ref stroke) = self.stroke {
            let mut paint = stroke.to_sk_paint(bounds);
            paint.set_style(sk::PaintStyle::Stroke);
            canvas.save();
            canvas.translate(bounds.top_left().to_skia());
            canvas.draw_path(&self.path, &paint);
            canvas.restore();
        }
    }
}*/

////////////////////////////////////////////////////////////////////////////////////////////////////

/*
/// Style of a container.
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

fn radii_to_skia(ctx: &mut PaintCtx, bounds: Rect, radii: &[Length; 4]) -> [sk::Vector; 4] {
    // FIXME: height-relative sizes
    let radii_dips = [
        radii[0].to_dips(ctx.scale_factor, bounds.size.width),
        radii[1].to_dips(ctx.scale_factor, bounds.size.width),
        radii[2].to_dips(ctx.scale_factor, bounds.size.width),
        radii[3].to_dips(ctx.scale_factor, bounds.size.width),
    ];

    // TODO x,y radii
    [
        sk::Vector::new(radii_dips[0] as sk::scalar, radii_dips[0] as sk::scalar),
        sk::Vector::new(radii_dips[1] as sk::scalar, radii_dips[1] as sk::scalar),
        sk::Vector::new(radii_dips[2] as sk::scalar, radii_dips[2] as sk::scalar),
        sk::Vector::new(radii_dips[3] as sk::scalar, radii_dips[3] as sk::scalar),
    ]
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

impl_env_value!(Style);

/// From CSS value.
impl TryFrom<&str> for Style {
    type Error = ();
    fn try_from(css: &str) -> Result<Self, ()> {
        Style::parse(css).map_err(|_| ())
    }
}

//--------------------------------------------------------------------------------------------------
pub trait PaintCtxExt {
    fn draw_styled_box(&mut self, bounds: Rect, box_style: &Style);
}

impl<'a> PaintCtxExt for PaintCtx<'a> {
    fn draw_styled_box(&mut self, bounds: Rect, box_style: &Style) {
        box_style.draw(self, bounds)
    }
}*/
