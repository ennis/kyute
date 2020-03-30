/*use crate::ui::kyute::layout::{Alignment, Bounds, BoxConstraints, Layout, Offset, Point, Size};
use crate::ui::kyute::renderer::Painter;
use crate::ui::kyute::Widget;*/

/*/// Expands the child widget to fill all its available space.
pub struct Align<Inner> {
    inner: Inner,
    alignment: Alignment,
}

impl<Inner> Align<Inner> {
    pub fn new(alignment: Alignment, inner: Inner) -> Align<Inner> {
        Align { inner, alignment }
    }
}

impl<A, Inner> Widget<A> for Align<Inner>
where
    Inner: Widget<A>,
{
    fn layout(&mut self, painter: &Painter, constraints: &BoxConstraints) -> LayoutNode {
        let child = self.inner.layout(painter, constraints);

        let parent_size = constraints.biggest();
        let child_size = child.size();

        let parent_pos = Point::new(
            0.5 * parent_size.width * (1.0 + self.alignment.x),
            0.5 * parent_size.height * (1.0 + self.alignment.y),
        );
        let child_pos = Point::new(
            0.5 * child_size.width * (1.0 + self.alignment.x),
            0.5 * child_size.height * (1.0 + self.alignment.y),
        );
        let offset = parent_pos - child_pos;
        child.with_offset(offset)
    }

    fn paint(&mut self, painter: &mut Painter, layout: Layout) {
        self.inner.paint(painter, layout);
    }
}
*/
