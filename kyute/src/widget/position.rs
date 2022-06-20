//! Relative positioning
use crate::widget::prelude::*;

enum Anchor {
    Left(Length),
    Top(Length),
    Bottom(Length),
    Right(Length),
}

/// Positioning wrapper
pub struct RelativePositioning<W> {
    inner: W,
    anchor: Anchor,
}

impl<W> RelativePositioning<W> {
    pub fn left(length: Length, inner: W) -> RelativePositioning<W> {
        RelativePositioning {
            anchor: Anchor::Left(length),
            inner,
        }
    }
    pub fn top(length: Length, inner: W) -> RelativePositioning<W> {
        RelativePositioning {
            anchor: Anchor::Top(length),
            inner,
        }
    }
    pub fn bottom(length: Length, inner: W) -> RelativePositioning<W> {
        RelativePositioning {
            anchor: Anchor::Bottom(length),
            inner,
        }
    }
    pub fn right(length: Length, inner: W) -> RelativePositioning<W> {
        RelativePositioning {
            anchor: Anchor::Right(length),
            inner,
        }
    }
}

impl<W: Widget + 'static> WidgetWrapper for RelativePositioning<W> {
    type Inner = W;

    fn inner(&self) -> &Self::Inner {
        &self.inner
    }

    fn inner_mut(&mut self) -> &mut Self::Inner {
        &mut self.inner
    }

    fn layout(&self, ctx: &mut LayoutCtx, constraints: &LayoutConstraints, env: &Environment) -> Layout {
        let sublayout = self.inner.layout(ctx, constraints, env);
        let mut layout = sublayout;
        match self.anchor {
            Anchor::Left(left) => {
                layout.left = Some(constraints.resolve_width(left));
            }
            Anchor::Top(top) => {
                layout.top = Some(constraints.resolve_height(top));
            }
            Anchor::Bottom(bottom) => {
                layout.bottom = Some(constraints.resolve_height(bottom));
            }
            Anchor::Right(right) => {
                layout.right = Some(constraints.resolve_width(right));
            }
        }
        layout
    }
}
