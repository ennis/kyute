use crate::{drawing::ToSkia, Point};
use skia_safe as sk;
use skia_safe::{path::ArcSize, scalar, PathDirection};
use svgtypes::{PathParser, PathSegment};

/// List of all path commands.
#[derive(Clone, Copy, PartialEq, Debug)]
#[allow(missing_docs)]
pub enum PathCommand {
    MoveTo,
    LineTo,
    HorizontalLineTo,
    VerticalLineTo,
    CurveTo,
    SmoothCurveTo,
    Quadratic,
    SmoothQuadratic,
    EllipticalArc,
    //ClosePath,
}

pub(crate) fn svg_path_to_skia(svg_path: &str) -> Result<sk::Path, svgtypes::Error> {
    let mut sk_path = sk::Path::new();
    let mut last_cp = Point::origin();
    let mut last_p = Point::origin();
    let mut last_verb = PathCommand::MoveTo;

    fn absx(abs: bool, last_p: Point, x: f64) -> f64 {
        if abs {
            x
        } else {
            x + last_p.x
        }
    }

    fn absy(abs: bool, last_p: Point, y: f64) -> f64 {
        if abs {
            y
        } else {
            y + last_p.y
        }
    }

    fn abs2(abs: bool, last_p: Point, x: f64, y: f64) -> (f64, f64) {
        if abs {
            (x, y)
        } else {
            (x + last_p.x, y + last_p.y)
        }
    }

    let parser = PathParser::from(svg_path);

    for segment in parser {
        match segment? {
            PathSegment::MoveTo { abs, x, y } => {
                let (x, y) = abs2(abs, last_p, x, y);
                sk_path.move_to((x as scalar, y as scalar));
                last_p = (x, y).into();
                last_verb = PathCommand::MoveTo;
            }
            PathSegment::LineTo { abs, x, y } => {
                let (x, y) = abs2(abs, last_p, x, y);
                sk_path.line_to((x as scalar, y as scalar));
                last_p = (x, y).into();
                last_verb = PathCommand::LineTo;
            }
            PathSegment::HorizontalLineTo { abs, x } => {
                let x = absx(abs, last_p, x);
                sk_path.line_to((x as scalar, last_p.y as scalar));
                last_p.x = x;
                last_verb = PathCommand::HorizontalLineTo;
            }
            PathSegment::VerticalLineTo { abs, y } => {
                let y = absy(abs, last_p, y);
                sk_path.line_to((last_p.x as scalar, y as scalar));
                last_p.y = y;
                last_verb = PathCommand::VerticalLineTo;
            }
            PathSegment::CurveTo {
                abs,
                x1,
                y1,
                x2,
                y2,
                x,
                y,
            } => {
                let (x1, y1) = abs2(abs, last_p, x1, y1);
                let (x2, y2) = abs2(abs, last_p, x2, y2);
                let (x, y) = abs2(abs, last_p, x, y);
                sk_path.cubic_to(
                    (x1 as scalar, y1 as scalar),
                    (x2 as scalar, y2 as scalar),
                    (x as scalar, y as scalar),
                );
                last_p = (x, y).into();
                last_cp = (x2, y2).into();
                last_verb = PathCommand::CurveTo;
            }
            PathSegment::SmoothCurveTo { abs, x2, y2, x, y } => {
                let (x2, y2) = abs2(abs, last_p, x2, y2);
                let (x, y) = abs2(abs, last_p, x, y);
                let cp1 = match last_verb {
                    PathCommand::CurveTo | PathCommand::SmoothCurveTo => last_p - (last_cp - last_p),
                    _ => last_p,
                };
                sk_path.cubic_to(cp1.to_skia(), (x2 as scalar, y2 as scalar), (x as scalar, y as scalar));
                last_cp = (x2, y2).into();
                last_p = (x, y).into();
                last_verb = PathCommand::SmoothCurveTo;
            }
            PathSegment::Quadratic { abs, x1, y1, x, y } => {
                let (x1, y1) = abs2(abs, last_p, x1, y1);
                let (x, y) = abs2(abs, last_p, x, y);
                sk_path.quad_to((x1 as scalar, y1 as scalar), (x as scalar, y as scalar));
                last_p = (x, y).into();
                last_cp = (x1, y1).into();
                last_verb = PathCommand::Quadratic;
            }
            PathSegment::SmoothQuadratic { abs, x, y } => {
                let (x, y) = abs2(abs, last_p, x, y);
                let cp = match last_verb {
                    PathCommand::Quadratic | PathCommand::SmoothQuadratic => last_p - (last_cp - last_p),
                    _ => last_p,
                };
                sk_path.quad_to(cp.to_skia(), (x as scalar, y as scalar));
                last_cp = cp.into();
                last_p = (x, y).into();
                last_verb = PathCommand::SmoothQuadratic;
            }
            PathSegment::EllipticalArc {
                abs,
                rx,
                ry,
                x_axis_rotation,
                large_arc,
                sweep,
                x,
                y,
            } => {
                let (x, y) = abs2(abs, last_p, x, y);
                let large_arc = if large_arc { ArcSize::Large } else { ArcSize::Small };
                let direction = if sweep { PathDirection::CCW } else { PathDirection::CW };
                sk_path.arc_to_rotated(
                    (rx as scalar, ry as scalar),
                    x_axis_rotation as scalar,
                    large_arc,
                    direction,
                    (x as scalar, y as scalar),
                );
                last_p = (x, y).into();
                last_verb = PathCommand::EllipticalArc;
            }
            PathSegment::ClosePath { abs: _ } => {
                sk_path.close();
            }
        }
    }

    Ok(sk_path)
}
