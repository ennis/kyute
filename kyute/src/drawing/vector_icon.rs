use crate::{
    drawing::{svg_path_to_skia, ToSkia},
    Color, PaintCtx, Rect, Size, Transform,
};
use anyhow::{anyhow, bail};
use skia_safe as sk;
use std::str::FromStr;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct DrawOptions<'a> {
    /// Draw only the specified groups.
    groups: Option<&'a [&'a str]>,
    /// Override transform.
    transform: Option<Transform>,
}

impl<'a> Default for DrawOptions<'a> {
    fn default() -> Self {
        DrawOptions {
            groups: None,
            transform: None,
        }
    }
}

#[derive(Debug)]
struct Group {
    id: String,
    transform: Transform,
    items: Vec<DrawItem>,
}

impl Group {
    fn from_svg(node: roxmltree::Node) -> anyhow::Result<Group> {
        let mut id = String::new();
        let mut transform = Default::default();
        let mut items = vec![];

        for attr in node.attributes() {
            match attr.name() {
                "id" => {
                    id = attr.value().to_string();
                }
                "transform" => {
                    let t = svgtypes::Transform::from_str(attr.value())?;
                    transform = Transform::new(t.a, t.b, t.c, t.d, t.e, t.f);
                }
                _ => {
                    warn!("unsupported path attribute: {}", attr.name());
                }
            }
        }

        for child in node.children() {
            if !child.is_element() {
                continue;
            }
            items.push(DrawItem::from_svg(child)?);
        }

        Ok(Group { id, transform, items })
    }

    fn draw(&self, ctx: &mut PaintCtx, options: &DrawOptions) {
        ctx.surface.canvas().save();
        ctx.surface.canvas().concat(&self.transform.to_skia());
        for item in self.items.iter() {
            item.draw(ctx, options)
        }
        ctx.surface.canvas().restore();
    }
}

#[derive(Debug)]
struct PathElem {
    path: sk::Path,
    fill: Option<Color>,
    stroke: Option<Color>,
    stroke_width: f64,
}

impl PathElem {
    fn from_svg(node: roxmltree::Node) -> anyhow::Result<PathElem> {
        let mut path = None;
        let mut id = String::new();
        let mut fill = None;
        let mut stroke = None;
        let mut stroke_width = 1.0;

        for attr in node.attributes() {
            match attr.name() {
                "d" => {
                    path = Some(svg_path_to_skia(attr.value())?);
                }
                "id" => {
                    id = attr.value().to_string();
                }
                "fill" => {
                    fill = if attr.value() == "none" {
                        None
                    } else {
                        let color = svgtypes::Color::from_str(attr.value())?;
                        Some(Color::from_rgba_u8(color.red, color.green, color.blue, color.alpha))
                    };
                }
                "stroke" => {
                    stroke = if attr.value() == "none" {
                        None
                    } else {
                        let color = svgtypes::Color::from_str(attr.value())?;
                        Some(Color::from_rgba_u8(color.red, color.green, color.blue, color.alpha))
                    };
                }
                "stroke-width" => {
                    stroke_width = svgtypes::Number::from_str(attr.value())?.0;
                }
                _ => {
                    warn!("unsupported path attribute: {}", attr.name());
                }
            }
        }

        if path.is_none() {
            bail!("<path> element without path data");
        }

        Ok(PathElem {
            path: path.unwrap(),
            fill,
            stroke,
            stroke_width,
        })
    }

    fn draw(&self, ctx: &mut PaintCtx, options: &DrawOptions) {
        let mut paint = sk::Paint::new(self.fill.unwrap_or_default().to_skia(), None);
        paint.set_anti_alias(true);
        if let Some(fill) = self.fill {
            paint.set_style(sk::PaintStyle::Fill);
            ctx.surface.canvas().draw_path(&self.path, &paint);
        }
        if let Some(stroke) = self.stroke {
            paint.set_style(sk::PaintStyle::Stroke);
            paint.set_stroke_width(self.stroke_width as f32);
            paint.set_color4f(stroke.to_skia(), None);
            ctx.surface.canvas().draw_path(&self.path, &paint);
        }
    }
}

#[derive(Debug)]
enum DrawItem {
    Group(Group),
    Path(PathElem),
}

impl DrawItem {
    fn from_svg(node: roxmltree::Node) -> anyhow::Result<DrawItem> {
        match node.tag_name().name() {
            "path" => Ok(DrawItem::Path(PathElem::from_svg(node)?)),
            "g" => Ok(DrawItem::Group(Group::from_svg(node)?)),
            other => {
                bail!("unsupported element: {}, data={:?}", other, node.text());
            }
        }
    }

    fn draw(&self, ctx: &mut PaintCtx, options: &DrawOptions) {
        match self {
            DrawItem::Group(group) => group.draw(ctx, options),
            DrawItem::Path(path) => path.draw(ctx, options),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// VectorIcon
////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct VectorIcon {
    items: Vec<DrawItem>,
    size: Size,
    view_box: Rect,
}

impl VectorIcon {
    pub fn load(svg: &str) -> anyhow::Result<VectorIcon> {
        let xml = roxmltree::Document::parse(svg)?;

        let svg = xml.root().first_child().ok_or_else(|| anyhow!("invalid µSVG"))?;
        if svg.tag_name().name() != "svg" {
            bail!("invalid µSVG")
        }

        let mut width = 0.0;
        let mut height = 0.0;
        let mut view_box = Rect::default();
        for attr in svg.attributes() {
            match attr.name() {
                "width" => {
                    width = svgtypes::Number::from_str(attr.value())?.0;
                }
                "height" => {
                    height = svgtypes::Number::from_str(attr.value())?.0;
                }
                "viewBox" => {
                    let vb = svgtypes::ViewBox::from_str(attr.value())?;
                    view_box.origin.x = vb.x;
                    view_box.origin.y = vb.y;
                    view_box.size.width = vb.w;
                    view_box.size.height = vb.h;
                }
                other => {
                    warn!("unsupported <svg> attribute: {}", other)
                }
            }
        }

        let mut items = vec![];

        for child in svg.children() {
            if !child.is_element() {
                continue;
            }
            match child.tag_name().name() {
                "defs" => {
                    // TODO
                }
                _ => {
                    items.push(DrawItem::from_svg(child)?);
                }
            }
        }

        Ok(VectorIcon {
            items,
            size: Size::new(width, height),
            view_box,
        })
    }

    pub fn draw(&self, ctx: &mut PaintCtx, options: &DrawOptions) {
        for item in self.items.iter() {
            item.draw(ctx, options)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_usvg() {
        let usvg = r##"
<svg viewBox="0 0 20 20" xmlns="http://www.w3.org/2000/svg" xml:space="preserve" fill-rule="evenodd" clip-rule="evenodd" stroke-miterlimit="1.5">
  <g id="darkmode" transform="matrix(1,0,0,1,-0.25,-2)">
    <path d="M2.5 13 9 18l9-14" fill="none" stroke="#000" stroke-width="4.07"/>
  </g>
</svg>
"##;
        let v = VectorIcon::load(usvg).unwrap();
        eprintln!("{:?}", v);
    }
}
