use crate::drawing::{mk_point_f, Point};

use crate::platform::Platform;
use std::ptr;
pub use svgtypes::Path;
pub use svgtypes::PathParser;
pub use svgtypes::PathSegment;
use thiserror::Error;
use winapi::shared::winerror::SUCCEEDED;
use winapi::um::d2d1::*;
use winapi::um::d2d1_1::*;
use wio::com::ComPtr;

pub struct PathGeometry(pub(crate) ComPtr<ID2D1PathGeometry1>);

#[derive(Debug, Error)]
pub enum PathError {
    #[error("invalid path syntax")]
    SyntaxError(#[from] svgtypes::Error),
    #[error("could not create path")]
    Other(#[from] crate::error::Error),
}

impl PathGeometry {
    pub fn try_from_svg_path(platform: &Platform, path_str: &str) -> Result<PathGeometry, PathError> {
        // parse the path string
        let mut path: Path = path_str.parse().map_err(|e| PathError::SyntaxError(e))?;
        path.conv_to_absolute();

        // build geometry
        unsafe {
            let factory = &platform.0.d2d_factory;
            let mut path_geometry: *mut ID2D1PathGeometry1 = ptr::null_mut();
            let hr = factory.CreatePathGeometry(&mut path_geometry);
            assert!(SUCCEEDED(hr)); // TODO
            let path_geometry = ComPtr::from_raw(path_geometry);

            let mut geometry_sink: *mut ID2D1GeometrySink = ptr::null_mut();
            path_geometry.Open(&mut geometry_sink);
            let geometry_sink = ComPtr::from_raw(geometry_sink);
            assert!(SUCCEEDED(hr)); // TODO

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
                                D2D1_POINT_2F {
                                    x: init_x as f32,
                                    y: init_y as f32,
                                },
                                D2D1_FIGURE_BEGIN_FILLED,
                            );
                            in_figure = true;
                        }
                    }
                }

                match seg {
                    PathSegment::MoveTo { x, y, .. } => {
                        geometry_sink.EndFigure(D2D1_FIGURE_END_OPEN);
                        init_x = x;
                        init_y = y;
                    }
                    PathSegment::LineTo { x, y, .. } => geometry_sink.AddLine(D2D1_POINT_2F {
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
                        geometry_sink.EndFigure(D2D1_FIGURE_END_CLOSED);
                        in_figure = false;
                    }
                }
            }

            let hr = geometry_sink.Close();
            assert!(SUCCEEDED(hr)); // TODO
            Ok(PathGeometry(path_geometry))
        }
    }
}
