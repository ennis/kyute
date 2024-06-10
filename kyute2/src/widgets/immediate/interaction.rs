use crate::{
    widgets::immediate::{
        api::{Circle, Point, Rect, VarId},
        IMCTX,
    },
    Ctx, Event,
};

pub trait Shape {
    fn to_region(&self) -> InteractRegion;
}

impl Shape for Rect {
    fn to_region(&self) -> InteractRegion {
        InteractRegion::Rect { rect: *self }
    }
}

impl Shape for Circle {
    fn to_region(&self) -> InteractRegion {
        InteractRegion::Circle {
            center: self.center,
            radius: self.radius,
        }
    }
}

pub fn interact(shape: impl Shape, f: impl FnMut(&mut Ctx, &Event)) {
    IMCTX.with(|imctx| imctx.add_interaction(shape.to_region(), f));
}

pub enum InteractRegion {
    Rect { rect: Rect },
    Circle { center: Point, radius: VarId },
}

impl InteractRegion {
    pub fn resolve(&self) -> ResolvedInteractRegion {
        match self {
            InteractRegion::Rect { rect } => ResolvedInteractRegion::Rect { rect: rect.resolve() },
            InteractRegion::Circle { center, radius } => ResolvedInteractRegion::Circle {
                center: center.resolve(),
                radius: radius.resolve(),
            },
        }
    }
}

pub enum ResolvedInteractRegion {
    Rect { rect: kurbo::Rect },
    Circle { center: kurbo::Point, radius: f64 },
}

impl ResolvedInteractRegion {
    pub fn hit_test(&self, position: kurbo::Point) -> bool {
        match self {
            ResolvedInteractRegion::Rect { rect } => rect.contains(position),
            ResolvedInteractRegion::Circle { center, radius } => {
                let distance = (*center - position).hypot();
                distance <= *radius
            }
        }
    }
}

pub struct Interaction {
    pub region: InteractRegion,
    pub handler: Box<dyn FnMut(&mut Ctx, &Event)>,
}

impl Interaction {
    pub fn resolve(self) -> ResolvedInteraction {
        ResolvedInteraction {
            region: self.region.resolve(),
            handler: self.handler,
        }
    }
}

pub struct ResolvedInteraction {
    pub region: ResolvedInteractRegion,
    pub handler: Box<dyn FnMut(&mut Ctx, &Event)>,
}
