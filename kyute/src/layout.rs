

#[derive(Copy,Clone,Debug)]
pub struct Size {
    pub w: f64,
    pub h: f64
}

#[derive(Copy,Clone,Debug)]
pub struct BoxConstraints {
    pub min: Size,
    pub max: Size,
}

impl BoxConstraints {
    pub fn new(min_w: f64, min_h: f64, max_w: f64, max_h: f64) -> BoxConstraints {
        BoxConstraints {
            min: Size { w: min_w, h: min_h },
            max: Size { w: max_w, h: max_h }
        }
    }

    pub fn tight(size: Size) -> BoxConstraints {
        BoxConstraints {
            min: size,
            max: size,
        }
    }
}