use std::any::Any;

use kurbo::Point;

use crate::{
    element::TransformNode, AnyWidget, BoxConstraints, ChangeFlags, Element, ElementId, Event, EventCtx, Geometry,
    HitTestResult, LayoutCtx, LengthOrPercentage, PaintCtx, Size, TreeCtx, Widget,
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
    pub items: Vec<FlexItem>,
    pub orientation: Axis,
    pub main_axis_alignment: MainAxisAlignment,
    pub cross_axis_alignment: CrossAxisAlignment,
}

pub struct FlexItem {
    pub flex: f64,
    pub alignment: Option<CrossAxisAlignment>,
    pub content: Box<dyn AnyWidget>,
}

impl Flex {}

impl Widget for Flex {
    type Element = FlexElement;

    fn build(self, cx: &mut TreeCtx, id: ElementId) -> Self::Element {
        let items: Vec<_> = self
            .items
            .into_iter()
            .enumerate()
            .map(|(i, item)|
                // FIXME: ID shouldn't be derived from index
                FlexElementItem {
                    alignment: item.alignment,
                    flex: item.flex,
                    content:TransformNode::new(cx.build_with_id(&i, item.content))
                })
            .collect();

        FlexElement {
            id,
            items,
            axis: self.orientation,
            main_axis_alignment: self.main_axis_alignment,
            cross_axis_alignment: self.cross_axis_alignment,
        }
    }

    fn update(self, cx: &mut TreeCtx, element: &mut Self::Element) -> ChangeFlags {
        let mut f = ChangeFlags::empty();

        fn update<T: PartialEq>(orig: &mut T, new: T, flags: &mut ChangeFlags, change: ChangeFlags) {
            if *orig != new {
                *orig = new;
                *flags |= change;
            }
        }
        update(&mut element.axis, self.orientation, &mut f, ChangeFlags::GEOMETRY);
        update(
            &mut element.main_axis_alignment,
            self.main_axis_alignment,
            &mut f,
            ChangeFlags::GEOMETRY,
        );
        update(
            &mut element.cross_axis_alignment,
            self.cross_axis_alignment,
            &mut f,
            ChangeFlags::GEOMETRY,
        );
        //update(&mut element.options, self.options, &mut f, ChangeFlags::GEOMETRY);
        //update(&mut element.style, self.style, &mut f, ChangeFlags::PAINT);

        let num_items = self.items.len();
        let num_items_in_element = element.items.len();
        if num_items != num_items_in_element {
            f |= ChangeFlags::GEOMETRY;
        }

        for (i, item) in self.items.into_iter().enumerate() {
            // TODO: match by item identity
            if i < num_items_in_element {
                //eprintln!("update grid item");
                update(&mut element.items[i].flex, item.flex, &mut f, ChangeFlags::GEOMETRY);
                update(
                    &mut element.items[i].alignment,
                    item.alignment,
                    &mut f,
                    ChangeFlags::GEOMETRY,
                );
                f |= cx.update_with_id(&i, item, &mut element.items[i].content);
            } else {
                element.items.push(FlexElementItem {
                    flex: item.flex,
                    alignment: item.alignment,
                    content: TransformNode::new(cx.build_with_id(&i, item.content)),
                });
            }
        }

        element.items.truncate(num_items);
        f
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Axis {
    Horizontal,
    Vertical,
}

impl Axis {
    fn main_size(&self, size: Size) -> f64 {
        match self {
            Axis::Horizontal => size.width,
            Axis::Vertical => size.height,
        }
    }
    fn cross_size(&self, size: Size) -> f64 {
        match self {
            Axis::Horizontal => size.height,
            Axis::Vertical => size.width,
        }
    }

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

    fn main_axis_constraints(&self, box_constraints: &BoxConstraints) -> (f64, f64) {
        match self {
            Axis::Horizontal => (box_constraints.min.width, box_constraints.max.width),
            Axis::Vertical => (box_constraints.min.height, box_constraints.max.height),
        }
    }

    fn cross_axis_constraints(&self, box_constraints: &BoxConstraints) -> (f64, f64) {
        match self {
            Axis::Horizontal => (box_constraints.min.height, box_constraints.max.height),
            Axis::Vertical => (box_constraints.min.width, box_constraints.max.width),
        }
    }
}

pub struct FlexElementItem {
    flex: f64,
    alignment: Option<CrossAxisAlignment>,
    content: TransformNode<Box<dyn Element>>,
}

pub struct FlexElement {
    id: ElementId,
    axis: Axis,
    main_axis_alignment: MainAxisAlignment,
    cross_axis_alignment: CrossAxisAlignment,
    items: Vec<FlexElementItem>,
}

fn cross_axis_natural_size(
    orientation: Axis,
    alignment: CrossAxisAlignment,
    item: FlexElementItem,
    max_size: Size,
) -> f64 {
    match orientation {
        Axis::Horizontal => e.natural_height(params.max.width),
        Axis::Vertical => e.natural_width(params.max.height),
    }
}

fn zero_flex_child_constraints(
    main_axis: Axis,
    cross_axis_alignment: CrossAxisAlignment,
    constraints: &BoxConstraints,
) -> BoxConstraints {
    let (mut cross_min, cross_max) = main_axis.cross_axis_constraints(constraints);
    if cross_axis_alignment == CrossAxisAlignment::Stretch {
        cross_min = cross_max;
    }
    main_axis.constraints(0.0, f64::INFINITY, cross_min, cross_max)
}

fn non_zero_flex_child_constraints(
    main_axis: Axis,
    flex: f64,
    cross_axis_alignment: CrossAxisAlignment,
    constraints: &BoxConstraints,
) -> BoxConstraints {
    let (mut cross_min, cross_max) = main_axis.cross_axis_constraints(constraints);
    if cross_axis_alignment == CrossAxisAlignment::Stretch {
        cross_min = cross_max;
    }
    main_axis.constraints(0.0, f64::INFINITY, cross_min, cross_max)
}

impl Element for FlexElement {
    fn id(&self) -> ElementId {
        self.id
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, constraints: &BoxConstraints) -> Geometry {
        // Quoting the flutter docs:
        //
        //      Layout for a Flex proceeds in six steps:
        //
        //      1.Layout each child with a null or zero flex factor (e.g., those that are not Expanded) with unbounded main axis constraints and the incoming cross axis constraints.
        // If the crossAxisAlignment is CrossAxisAlignment.stretch, instead use tight cross axis constraints that match the incoming max extent in the cross axis.
        //      Divide the remaining main axis space among the children with non-zero flex factors (e.g., those that are Expanded) according to their flex factor.
        // For example, a child with a flex factor of 2.0 will receive twice the amount of main axis space as a child with a flex factor of 1.0.
        //      Layout each of the remaining children with the same cross axis constraints as in step 1, but instead of using unbounded main axis constraints,
        // use max axis constraints based on the amount of space allocated in step 2. Children with Flexible.fit properties that are FlexFit.tight are given tight constraints
        // (i.e., forced to fill the allocated space), and children with Flexible.fit properties that are FlexFit.loose are given loose constraints (i.e., not forced to fill the allocated space).
        //      The cross axis extent of the Flex is the maximum cross axis extent of the children (which will always satisfy the incoming constraints).
        //      The main axis extent of the Flex is determined by the mainAxisSize property. If the mainAxisSize property is MainAxisSize.max, then the main axis extent of the Flex is the max extent of the incoming main axis constraints. If the mainAxisSize property is MainAxisSize.min, then the main axis extent of the Flex is the sum of the main axis extents of the children (subject to the incoming constraints).
        //      Determine the position for each child according to the mainAxisAlignment and the crossAxisAlignment. For example, if the mainAxisAlignment is MainAxisAlignment.spaceBetween, any main axis space that has not been allocated to children is divided evenly and placed between the children.

        let main_axis_natural_size = move |e: &mut dyn Element| match self.axis {
            Axis::Horizontal => e.natural_height(params.max.width),
            Axis::Vertical => e.natural_width(params.max.height),
        };

        let (main_axis_min, main_axis_max) = self.axis.main_axis_constraints(constraints);

        let mut flex_sum = 0.0;
        let mut non_flex_main_total = 0.0;

        let mut child_geoms = vec![Geometry::ZERO; self.items.len()];

        for (i, c) in self.items.iter_mut().enumerate() {
            if c.flex == 0.0 {
                let child_cstr = zero_flex_child_constraints(
                    self.axis,
                    c.alignment.unwrap_or(self.cross_axis_alignment),
                    constraints,
                );
                let child_geom = ctx.layout(&mut c.content, &child_cstr);
                non_flex_main_total += child_geom.size.width;
                child_geoms[i] = child_geom;
            } else {
                flex_sum += c.flex;
            }
        }

        // Divide the remaining main axis space among the children with non-zero flex factors
        let remaining_main = main_axis_max - non_flex_main_total;
        for (i, c) in self.items.iter_mut().enumerate() {
            if c.flex != 0.0 {}
        }

        Geometry::ZERO
    }

    fn event(&mut self, ctx: &mut EventCtx, event: &mut Event) -> ChangeFlags {
        if let Some(next_target) = event.next_target() {
            let child = self
                .items
                .iter_mut()
                .find(|e| e.id() == next_target)
                .expect("invalid child specified");
            child.event(ctx, event)
        } else {
            // Nothing
            ChangeFlags::NONE
        }
    }

    fn natural_width(&mut self, _height: f64) -> f64 {
        todo!()
    }

    fn natural_height(&mut self, _width: f64) -> f64 {
        todo!()
    }

    fn natural_baseline(&mut self, _params: &BoxConstraints) -> f64 {
        todo!()
    }

    fn hit_test(&self, _ctx: &mut HitTestResult, _position: Point) -> bool {
        todo!()
    }

    fn paint(&mut self, _ctx: &mut PaintCtx) {
        todo!()
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
