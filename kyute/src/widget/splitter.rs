use crate::{align_boxes, composable, core2::WindowPaintCtx, layout::BoxConstraints, widget::{Axis, LayoutWrapper}, Alignment, Environment, Event, EventCtx, GpuFrameCtx, LayoutCtx, Measurements, Offset, PaintCtx, Rect, Widget, WidgetPod, Size, Orientation, Key, cache};
use kyute_shell::drawing::ToSkia;
use std::cell::Cell;

/// Splits a region vertically or horizontally into two sub-regions of adjustable sizes.
#[derive(Clone)]
pub struct SplitPane {
    orientation: Orientation,
    split_points: Vec<f64>,
    new_split_points: Key<Option<Vec<f64>>>,
    nodes: Vec<WidgetPod>,
}

impl SplitPane
{
    #[composable(uncached)]
    pub fn new(orientation: Orientation) -> SplitPane {
        let new_split_points = cache::state(|| None);
        SplitPane {
            orientation,
            split_points: vec![],
            new_split_points,
            nodes: vec![]
        }
    }

    /// Adds a new child widget.
    ///
    /// Note: this resets the split positions previously set with `split_points`.
    #[composable(uncached)]
    pub fn push(&mut self, node: impl Widget + 'static) {
        self.nodes.push(WidgetPod::new(node));
    }

    /// Sets the position of the splits. `split_points` must contain be `N-1` sorted values between 0.0 and 1.0,
    /// where `N` is the number of child widgets added to the SplitPane.
    pub fn split_points(mut self, split_points: impl Into<Vec<f64>>) -> SplitPane {
        let split_points = split_points.into();
        assert!((self.nodes.len() == 0 && split_points.len() == 0) || (self.nodes.len() > 0 && (split_points.len() == self.nodes.len() - 1)));
        self.split_points = split_points.into();
        self
    }

    /// If the split positions have changed, returns the new splits.
    pub fn new_split_points(&self) -> Option<Vec<f64>> {
        self.new_split_points.update(None)
    }
}

impl Widget for SplitPane
{
    fn event(&self, ctx: &mut EventCtx, event: &mut Event, env: &Environment) {
        self.left.event(ctx, event, env);
        self.right.event(ctx, event, env);
    }

    fn layout(
        &self,
        ctx: &mut LayoutCtx,
        constraints: BoxConstraints,
        env: &Environment,
    ) -> Measurements {

        // len:
        let (len, cross_len) = match self.axis {
            Axis::Horizontal => {
                (constraints.max_height(), constraints.max_width())
            }
            Axis::Vertical => {
                (constraints.max_width(), constraints.max_height())
            }
        };

        let (left_len, right_len) = if w.is_infinite() {
            tracing::warn!("Splitter::layout: no width or height constraint along split");
            (1000.0 * self.position, w)
        } else {
            (w * self.position, (1.0-self.position) * w)
        };

        let (m_left, m_right) = match self.axis {
            Axis::Horizontal => {
                todo!()
            }
            Axis::Vertical => {
                let left_constraints = BoxConstraints {
                    max: Size(left_len, cross_len),
                    .. constraints
                };
                let right_constraints = BoxConstraints {
                    max: Size(right_len, cross_len),
                    .. constraints
                };
                let m_left = self.left.layout(ctx, left_constraints, env);
                let m_right = self.right.layout(ctx, right_constraints, env);
                (m_left, m_right)
            }
        };

        let child_measurements = self.inner.layout(ctx, constraints.loosen(), env);
        let mut m = Measurements::new(constraints.constrain(child_measurements.size()).into());
        let offset = align_boxes(self.alignment, &mut m, child_measurements);
        self.inner.set_child_offset(offset);

        Measurements::new(constraints.)
        m
    }

    fn paint(&self, ctx: &mut PaintCtx, bounds: Rect, env: &Environment) {
        self.inner.paint(ctx, bounds, env)
    }
}
