use kurbo::{Point, Rect, Vec2};

use crate::{
    BoxConstraints, Event, Geometry, HitTestResult, IntoWidget, LayoutCtx, PaintCtx, Size, Widget, WidgetCtx,
    WidgetPod, WidgetPtr,
};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub enum MainAxisAlignment {
    Start,
    End,
    Center,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub enum CrossAxisAlignment {
    Start,
    End,
    Center,
    Stretch,
    Baseline,
}

////////////////////////////////////////////////////////////////////////////////////////////////////
pub struct Flex {
    pub axis: Axis,
    pub main_axis_alignment: MainAxisAlignment,
    pub cross_axis_alignment: CrossAxisAlignment,
    items: Vec<FlexItem>,
}

impl Flex {
    pub fn new(axis: Axis) -> Flex {
        Flex {
            axis,
            main_axis_alignment: MainAxisAlignment::Start,
            cross_axis_alignment: CrossAxisAlignment::Start,
            items: Vec::new(),
        }
    }

    pub fn row() -> Flex {
        Flex::new(Axis::Horizontal)
    }

    pub fn column() -> Flex {
        Flex::new(Axis::Vertical)
    }

    pub fn push(&mut self, item: impl IntoWidget) {
        self.items.push(FlexItem {
            flex: 0.0,
            alignment: None,
            offset: Vec2::ZERO,
            size: Size::ZERO,
            content: item.into_widget_pod(),
        });
    }

    pub fn push_flex(&mut self, item: impl IntoWidget, flex: f64) {
        self.items.push(FlexItem {
            flex,
            alignment: None,
            offset: Vec2::ZERO,
            size: Size::ZERO,
            content: item.into_widget_pod(),
        });
    }
}

pub struct FlexItem {
    flex: f64,
    alignment: Option<CrossAxisAlignment>,
    offset: Vec2,
    size: Size,
    content: WidgetPtr,
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Axis {
    Horizontal,
    Vertical,
}

impl Axis {
    fn constraints(
        &self,
        main_axis_min: f64,
        main_axis_max: f64,
        cross_axis_min: f64,
        cross_axis_max: f64,
    ) -> BoxConstraints {
        match self {
            Axis::Horizontal => BoxConstraints {
                min: Size {
                    width: main_axis_min,
                    height: cross_axis_min,
                },
                max: Size {
                    width: main_axis_max,
                    height: cross_axis_max,
                },
            },
            Axis::Vertical => BoxConstraints {
                min: Size {
                    width: cross_axis_min,
                    height: main_axis_min,
                },
                max: Size {
                    width: cross_axis_max,
                    height: main_axis_max,
                },
            },
        }
    }
}

/// Helper trait for main axis/cross axis sizes
trait AxisSizeHelper {
    fn main_length(&self, main_axis: Axis) -> f64;
    fn cross_length(&self, main_axis: Axis) -> f64;

    fn from_main_cross(main_axis: Axis, main: f64, cross: f64) -> Self;
}

impl AxisSizeHelper for Size {
    fn main_length(&self, main_axis: Axis) -> f64 {
        match main_axis {
            Axis::Horizontal => self.width,
            Axis::Vertical => self.height,
        }
    }

    fn cross_length(&self, main_axis: Axis) -> f64 {
        match main_axis {
            Axis::Horizontal => self.height,
            Axis::Vertical => self.width,
        }
    }

    fn from_main_cross(main_axis: Axis, main: f64, cross: f64) -> Self {
        match main_axis {
            Axis::Horizontal => Size {
                width: main,
                height: cross,
            },
            Axis::Vertical => Size {
                width: cross,
                height: main,
            },
        }
    }
}

trait AxisOffsetHelper {
    fn set_main_axis_offset(&mut self, main_axis: Axis, offset: f64);
    fn set_cross_axis_offset(&mut self, main_axis: Axis, offset: f64);
}

impl AxisOffsetHelper for Vec2 {
    fn set_main_axis_offset(&mut self, main_axis: Axis, offset: f64) {
        match main_axis {
            Axis::Horizontal => self.x = offset,
            Axis::Vertical => self.y = offset,
        }
    }

    fn set_cross_axis_offset(&mut self, main_axis: Axis, offset: f64) {
        match main_axis {
            Axis::Horizontal => self.y = offset,
            Axis::Vertical => self.x = offset,
        }
    }
}

fn main_cross_constraints(axis: Axis, min_main: f64, max_main: f64, min_cross: f64, max_cross: f64) -> BoxConstraints {
    match axis {
        Axis::Horizontal => BoxConstraints {
            min: Size {
                width: min_main,
                height: min_cross,
            },
            max: Size {
                width: max_main,
                height: max_cross,
            },
        },
        Axis::Vertical => BoxConstraints {
            min: Size {
                width: min_cross,
                height: min_main,
            },
            max: Size {
                width: max_cross,
                height: max_main,
            },
        },
    }
}

impl Widget for Flex {
    fn mount(&mut self, cx: &mut WidgetCtx) {
        for item in &mut self.items {
            item.content.mount(cx);
        }
    }

    fn update(&mut self, cx: &mut WidgetCtx) {
        for item in &mut self.items {
            item.content.update(cx);
        }
    }

    fn event(&mut self, _cx: &mut WidgetCtx, _event: &mut Event) {}

    fn hit_test(&mut self, result: &mut HitTestResult, position: Point) -> bool {
        for item in &mut self.items {
            if result.test_with_offset(item.offset, position, |result, position| {
                item.content.hit_test(result, position)
            }) {
                return true;
            }
        }
        // we don't hit-test the blank space
        false
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, constraints: &BoxConstraints) -> Geometry {
        let axis = self.axis;
        let (main_axis_min, main_axis_max, mut cross_axis_min, cross_axis_max) = if axis == Axis::Horizontal {
            (
                constraints.min.width,
                constraints.max.width,
                constraints.min.height,
                constraints.max.height,
            )
        } else {
            (
                constraints.min.height,
                constraints.max.height,
                constraints.min.width,
                constraints.max.width,
            )
        };

        // stretch constraints
        if self.cross_axis_alignment == CrossAxisAlignment::Stretch {
            cross_axis_min = cross_axis_max;
        }

        let flex_sum: f64 = self.items.iter().map(|e| e.flex).sum(); // sum of flex factors
        let mut non_flex_main_total = 0.0; // total size of inflexible children
        let mut child_geoms = vec![Geometry::ZERO; self.items.len()];
        // Layout each child with a zero flex factor (i.e. they don't expand along the main axis, they get their natural size instead)
        for (i, c) in self.items.iter_mut().enumerate() {
            if c.flex == 0.0 {
                // layout child with unbounded main axis constraints and the incoming cross axis constraints
                let child_constraints =
                    main_cross_constraints(axis, 0.0, f64::INFINITY, cross_axis_min, cross_axis_max);
                child_geoms[i] = c.content.layout(ctx, &child_constraints);
                non_flex_main_total += child_geoms[i].size.main_length(axis);
            }
        }

        // Divide the remaining main axis space among the children with non-zero flex factors
        let remaining_main = main_axis_max - non_flex_main_total;
        for (i, c) in self.items.iter_mut().enumerate() {
            if c.flex != 0.0 {
                let main_size = remaining_main * c.flex / flex_sum;
                // pass loose constraints along the main axis; it's the child's job to decide whether to fill the space or not
                let child_constraints = main_cross_constraints(axis, 0.0, main_size, cross_axis_min, cross_axis_max);
                child_geoms[i] = c.content.layout(ctx, &child_constraints);
            }
        }

        // Determine the main-axis extent.
        // This is the sum of main-axis sizes of children, subject to the incoming constraints.
        // If you want the flex to take all the available space along the main axis, pass tight constraints as input.
        let main_axis_size: f64 = child_geoms.iter().map(|g| g.size.main_length(axis)).sum();
        let main_axis_size_constrained = main_axis_size.max(main_axis_min).min(main_axis_max);
        let blank_space = main_axis_size_constrained - main_axis_size;

        // Position the children, depending on main axis alignment
        let space = match self.main_axis_alignment {
            MainAxisAlignment::SpaceBetween => blank_space / (self.items.len() - 1) as f64,
            MainAxisAlignment::SpaceAround => blank_space / self.items.len() as f64,
            MainAxisAlignment::SpaceEvenly => blank_space / (self.items.len() + 1) as f64,
            MainAxisAlignment::Center | MainAxisAlignment::Start | MainAxisAlignment::End => 0.0,
        };
        let mut offset = match self.main_axis_alignment {
            MainAxisAlignment::SpaceBetween => 0.0,
            MainAxisAlignment::SpaceAround => space / 2.0,
            MainAxisAlignment::SpaceEvenly => space,
            MainAxisAlignment::Center => blank_space / 2.0,
            MainAxisAlignment::Start => 0.0,
            MainAxisAlignment::End => blank_space,
        };

        for (i, c) in self.items.iter_mut().enumerate() {
            c.offset.set_main_axis_offset(axis, offset);
            offset += child_geoms[i].size.main_length(axis) + space;
        }

        // Determine the cross-axis extent (maximum of cross-axis sizes of children)
        let max_cross_axis_size = child_geoms
            .iter()
            .map(|g| g.size.cross_length(axis))
            .reduce(f64::max)
            .unwrap();

        let mut max_baseline: f64 = 0.0;
        for c in child_geoms.iter() {
            let cb = c.baseline.unwrap_or(c.size.cross_length(axis));
            max_baseline = max_baseline.max(cb);
        }

        let max_cross_axis_size_baseline_aligned = child_geoms
            .iter()
            .map(|g| {
                let size = g.size.cross_length(axis);
                size + (max_baseline - g.baseline.unwrap_or(size))
            })
            .reduce(f64::max)
            .unwrap();

        let cross_axis_size = match self.cross_axis_alignment {
            CrossAxisAlignment::Baseline => max_cross_axis_size_baseline_aligned,
            _ => max_cross_axis_size,
        };
        let cross_axis_size = cross_axis_size.max(cross_axis_min).min(cross_axis_max);

        // Position the children on the cross axis
        for (i, c) in self.items.iter_mut().enumerate() {
            let size = c.size.cross_length(axis);
            let offset = match self.cross_axis_alignment {
                CrossAxisAlignment::Start => 0.0,
                CrossAxisAlignment::End => cross_axis_size - size,
                CrossAxisAlignment::Center => (cross_axis_size - size) / 2.0,
                CrossAxisAlignment::Stretch => 0.0,
                CrossAxisAlignment::Baseline => {
                    let baseline = child_geoms[i].baseline.unwrap_or(size);
                    max_baseline - baseline
                }
            };
            c.offset.set_cross_axis_offset(axis, offset);
        }

        let size = Size::from_main_cross(axis, main_axis_size_constrained, cross_axis_size);
        Geometry {
            size,
            baseline: Some(max_baseline),
            bounding_rect: Rect::from_origin_size(Point::ORIGIN, size),
            paint_bounding_rect: Rect::from_origin_size(Point::ORIGIN, size),
        }
    }

    fn paint(&mut self, cx: &mut PaintCtx) {
        for item in self.items.iter_mut() {
            cx.with_offset(item.offset, |cx| item.content.paint(cx));
        }
    }
}
