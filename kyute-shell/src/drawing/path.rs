use crate::drawing::{Point, ToSkia};
//use skia_safe as sk;
use skia_safe::{path::ArcSize, scalar, PathDirection};
use svgtypes::PathCommand;

pub type Path = svgtypes::Path;
pub type PathSegment = svgtypes::PathSegment;

impl ToSkia for Path {
    type Target = skia_safe::Path;

    fn to_skia(&self) -> Self::Target {
        let mut sk_path = skia_safe::Path::new();

        //let mut points = Vec::with_capacity(3.0*self.0.len());
        //let mut verbs = Vec::with_capacity(self.0.len());

        let mut last_cp = Point::origin();
        let mut last_p = Point::origin();
        let mut last_verb = PathCommand::MoveTo;

        for &segment in self.0.iter() {
            match segment {
                PathSegment::MoveTo { abs, x, y } => {
                    if abs {
                        sk_path.move_to((x as scalar, y as scalar));
                    } else {
                        sk_path.r_move_to((x as scalar, y as scalar));
                    }
                    last_p = (x, y).into();
                    last_verb = PathCommand::MoveTo;
                }
                PathSegment::LineTo { abs, x, y } => {
                    if abs {
                        sk_path.line_to((x as scalar, y as scalar));
                    } else {
                        sk_path.r_line_to((x as scalar, y as scalar));
                    }
                    last_p = (x, y).into();
                    last_verb = PathCommand::LineTo;
                }
                PathSegment::HorizontalLineTo { abs, x } => {
                    unimplemented!()
                }
                PathSegment::VerticalLineTo { abs, y } => {
                    unimplemented!()
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
                    if abs {
                        sk_path.cubic_to(
                            (x1 as scalar, y1 as scalar),
                            (x2 as scalar, y2 as scalar),
                            (x as scalar, y as scalar),
                        );
                    } else {
                        sk_path.r_cubic_to(
                            (x1 as scalar, y1 as scalar),
                            (x2 as scalar, y2 as scalar),
                            (x as scalar, y as scalar),
                        );
                    }
                    last_cp = (x2, y2).into();
                    last_p = (x, y).into();
                    last_verb = PathCommand::CurveTo;
                }
                PathSegment::SmoothCurveTo { abs, x2, y2, x, y } => {
                    let cp1 = match last_verb {
                        PathCommand::CurveTo | PathCommand::SmoothCurveTo => {
                            if abs {
                                last_p - (last_cp - last_p)
                            } else {
                                Point::origin() - (last_cp - last_p)
                            }
                        }
                        _ => last_p,
                    };

                    if abs {
                        sk_path.cubic_to(
                            cp1.to_skia(),
                            (x2 as scalar, y2 as scalar),
                            (x as scalar, y as scalar),
                        );
                    } else {
                        sk_path.r_cubic_to(
                            cp1.to_skia(),
                            (x2 as scalar, y2 as scalar),
                            (x as scalar, y as scalar),
                        );
                    }
                    last_cp = (x2, y2).into();
                    last_p = (x, y).into();
                    last_verb = PathCommand::SmoothCurveTo;
                }
                PathSegment::Quadratic { abs, x1, y1, x, y } => {
                    if abs {
                        sk_path.quad_to((x1 as scalar, y1 as scalar), (x as scalar, y as scalar));
                    } else {
                        sk_path.r_quad_to((x1 as scalar, y1 as scalar), (x as scalar, y as scalar));
                    }
                    last_cp = (x1, y1).into();
                    last_p = (x, y).into();
                    last_verb = PathCommand::Quadratic;
                }
                PathSegment::SmoothQuadratic { abs, x, y } => {
                    let cp = match last_verb {
                        PathCommand::Quadratic | PathCommand::SmoothQuadratic => {
                            if abs {
                                last_p - (last_cp - last_p)
                            } else {
                                Point::origin() - (last_cp - last_p)
                            }
                        }
                        _ => last_p,
                    };

                    if abs {
                        sk_path.quad_to(cp.to_skia(), (x as scalar, y as scalar));
                    } else {
                        sk_path.r_quad_to(cp.to_skia(), (x as scalar, y as scalar));
                    }
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
                    let large_arc = if large_arc {
                        ArcSize::Large
                    } else {
                        ArcSize::Small
                    };
                    let direction = if sweep {
                        PathDirection::CCW
                    } else {
                        PathDirection::CW
                    };

                    if abs {
                        sk_path.arc_to_rotated(
                            (rx as scalar, ry as scalar),
                            x_axis_rotation as scalar,
                            large_arc,
                            direction,
                            (x as scalar, y as scalar),
                        );
                    } else {
                        sk_path.r_arc_to_rotated(
                            (rx as scalar, ry as scalar),
                            x_axis_rotation as scalar,
                            large_arc,
                            direction,
                            (x as scalar, y as scalar),
                        );
                    }

                    last_p = (x, y).into();
                    last_verb = PathCommand::EllipticalArc;
                }
                PathSegment::ClosePath { abs: _ } => {
                    sk_path.close();
                }
            }
        }

        sk_path
    }
}
