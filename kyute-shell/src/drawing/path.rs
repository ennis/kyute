use crate::{
    bindings::Windows::Win32::Direct2D::{
        ID2D1PathGeometry1, D2D1_FIGURE_BEGIN, D2D1_FIGURE_END, D2D_POINT_2F,
    },
    platform::Platform,
};
pub use svgtypes::{Path, PathParser, PathSegment};
use thiserror::Error;
use windows::Interface;

pub struct PathGeometry(pub(crate) ID2D1PathGeometry1);

#[derive(Debug, Error)]
pub enum PathError {
    #[error("invalid path syntax")]
    SyntaxError(#[from] svgtypes::Error),
    #[error("could not create path")]
    Other(#[from] crate::error::Error),
}

impl PathGeometry {
    pub fn try_from_svg_path(path_str: &str) -> Result<PathGeometry, PathError> {
        let platform = Platform::instance();

        // parse the path string
        let mut path: Path = path_str.parse().map_err(|e| PathError::SyntaxError(e))?;
        path.conv_to_absolute();

        // build geometry
        unsafe {
            let factory = &platform.0.d2d_factory;
            let mut path_geometry = None;
            let path_geometry = factory
                .CreatePathGeometry(&mut path_geometry)
                .and_some(path_geometry)
                .unwrap()
                .cast::<ID2D1PathGeometry1>()
                .unwrap();
            let mut geometry_sink = None;
            let geometry_sink = path_geometry
                .Open(&mut geometry_sink)
                .and_some(geometry_sink)
                .unwrap();

            let mut in_figure = false;
            let (mut init_x, mut init_y) = (0.0, 0.0);

            for seg in path.0 {
                // begin figure
                if !in_figure {
                    match &seg {
                        PathSegment::MoveTo { x, y, .. } => {
                            init_x = *x;
                            init_y = *y;
                            continue;
                        }
                        PathSegment::ClosePath { .. } => {}
                        PathSegment::LineTo { .. }
                        | PathSegment::HorizontalLineTo { .. }
                        | PathSegment::VerticalLineTo { .. }
                        | PathSegment::CurveTo { .. }
                        | PathSegment::SmoothCurveTo { .. }
                        | PathSegment::Quadratic { .. }
                        | PathSegment::SmoothQuadratic { .. }
                        | PathSegment::EllipticalArc { .. } => {
                            geometry_sink.BeginFigure(
                                D2D_POINT_2F {
                                    x: init_x as f32,
                                    y: init_y as f32,
                                },
                                D2D1_FIGURE_BEGIN::D2D1_FIGURE_BEGIN_FILLED,
                            );
                            in_figure = true;
                        }
                    }
                }

                match seg {
                    PathSegment::MoveTo { x, y, .. } => {
                        geometry_sink.EndFigure(D2D1_FIGURE_END::D2D1_FIGURE_END_OPEN);
                        init_x = x;
                        init_y = y;
                    }
                    PathSegment::LineTo { x, y, .. } => geometry_sink.AddLine(D2D_POINT_2F {
                        x: x as f32,
                        y: y as f32,
                    }),
                    PathSegment::HorizontalLineTo { .. } => unimplemented!(),
                    PathSegment::VerticalLineTo { .. } => unimplemented!(),
                    PathSegment::CurveTo { .. } => unimplemented!(),
                    PathSegment::SmoothCurveTo { .. } => unimplemented!(),
                    PathSegment::Quadratic { .. } => unimplemented!(),
                    PathSegment::SmoothQuadratic { .. } => unimplemented!(),
                    PathSegment::EllipticalArc { .. } => unimplemented!(),
                    PathSegment::ClosePath { .. } => {
                        assert!(in_figure);
                        geometry_sink.EndFigure(D2D1_FIGURE_END::D2D1_FIGURE_END_CLOSED);
                        in_figure = false;
                    }
                }
            }

            geometry_sink.Close().ok().unwrap();
            Ok(PathGeometry(path_geometry))
        }
    }
}
