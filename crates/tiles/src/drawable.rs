use crate::cell::Cell;
use crate::color::Color;

pub trait Drawable {
    fn emit_cells(&self, f: &mut impl FnMut(Cell));

    fn color(self, c: Color) -> Colored<Self>
    where
        Self: Sized,
    {
        Colored { inner: self, color: c }
    }
}

pub struct Colored<D> {
    inner: D,
    color: Color,
}

impl<D: Drawable> Drawable for Colored<D> {
    fn emit_cells(&self, f: &mut impl FnMut(Cell)) {
        let color = self.color;
        self.inner.emit_cells(&mut |mut cell| {
            cell.color = color.to_array();
            f(cell);
        });
    }
}

impl Drawable for Cell {
    fn emit_cells(&self, f: &mut impl FnMut(Cell)) {
        f(*self);
    }
}
