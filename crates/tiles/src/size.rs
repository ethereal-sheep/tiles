#[derive(Debug, Default, Clone, Copy)]
pub struct Size {
    pub width: u32,
    pub height: u32,
}

impl Size {
    pub fn new(w: u32, h: u32) -> Self {
        Size {
            width: w,
            height: h,
        }
    }
}
