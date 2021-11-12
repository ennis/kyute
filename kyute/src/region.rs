use crate::Rect;

#[derive(Clone,Debug)]
pub struct Region {
    rects: Vec<Rect>,
}

impl Region {
    /// Creates an empty region
    pub fn new() -> Region {
        Region {
            rects: vec![]
        }
    }

    /// Adds a rectangle to this region.
    pub fn add_rect(&mut self, rect: Rect) {
        if rect.area() > 0.0 {
            self.rects.push(rect)
        }
    }

    /// Returns `true` if this region intersects the given rect.
    pub fn intersects(&self, rect: Rect) -> bool {
        self.rects.iter().any(|r| r.intersects(&rect))
    }

    /// Returns `true` if this region is empty.
    pub fn is_empty(&self) -> bool {
        self.rects.is_empty()
    }
}

impl Default for Region {
    fn default() -> Self {
        Region::new()
    }
}