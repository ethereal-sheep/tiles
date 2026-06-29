use crate::cell::Cell;
use crate::color::Color;

pub trait Drawable {
    fn emit_cells(&self, f: &mut impl FnMut(Cell));

    fn color(self, c: Color) -> Colored<Self>
    where
        Self: Sized,
    {
        Colored {
            inner: self,
            color: c,
        }
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

impl<A: Drawable, B: Drawable> Drawable for (A, B) {
    fn emit_cells(&self, f: &mut impl FnMut(Cell)) {
        self.0.emit_cells(f);
        self.1.emit_cells(f);
    }
}

impl<A: Drawable, B: Drawable, C: Drawable> Drawable for (A, B, C) {
    fn emit_cells(&self, f: &mut impl FnMut(Cell)) {
        self.0.emit_cells(f);
        self.1.emit_cells(f);
        self.2.emit_cells(f);
    }
}

impl<D: Drawable> Drawable for Vec<D> {
    fn emit_cells(&self, f: &mut impl FnMut(Cell)) {
        for d in self {
            d.emit_cells(f);
        }
    }
}

impl<D: Drawable, const N: usize> Drawable for [D; N] {
    fn emit_cells(&self, f: &mut impl FnMut(Cell)) {
        for d in self {
            d.emit_cells(f);
        }
    }
}
