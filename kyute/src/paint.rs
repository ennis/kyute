use piet_direct2d::{
    D2DFont, D2DFontBuilder, D2DRenderContext, D2DText, D2DTextLayout, D2DTextLayoutBuilder,
    GenericBrush,
};

pub type RenderContext<'a> = D2DRenderContext<'a>;
pub type Brush = GenericBrush;
pub type Text<'a> = D2DText<'a>;
pub type Font = D2DFont;
pub type FontBuilder<'a> = D2DFontBuilder<'a>;
pub type TextLayout = D2DTextLayout;
pub type TextLayoutBuilder<'a> = D2DTextLayoutBuilder<'a>;
pub type Image<'a> = <D2DRenderContext<'a> as piet::RenderContext>::Image;
