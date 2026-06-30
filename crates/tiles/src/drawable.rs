use crate::cell::Cell;
use crate::color::Color;

pub struct DrawableWrapper<F>
where
    F: Fn(&mut dyn FnMut(Cell)),
{
    emit_cells: F,
}

pub trait Drawable {
    fn emit_cells(&self, f: &mut dyn FnMut(Cell));

    fn to_cells(&self) -> Vec<Cell> {
        let mut cells = vec![];
        self.emit_cells(&mut |cell| cells.push(cell));
        cells
    }

    fn color(self, c: Color) -> DrawableWrapper<impl Fn(&mut dyn FnMut(Cell))>
    where
        Self: Sized,
    {
        self.map_cell(move |mut cell| {
            cell.color = c.to_array();
            cell
        })
    }

    fn map_cell(self, map: impl Fn(Cell) -> Cell) -> DrawableWrapper<impl Fn(&mut dyn FnMut(Cell))>
    where
        Self: Sized,
    {
        DrawableWrapper {
            emit_cells: move |out: &mut dyn FnMut(Cell)| {
                self.emit_cells(&mut |cell| {
                    out(map(cell));
                });
            },
        }
    }
}

impl<F: Fn(&mut dyn FnMut(Cell))> Drawable for DrawableWrapper<F> {
    fn emit_cells(&self, f: &mut dyn FnMut(Cell)) {
        (self.emit_cells)(f);
    }
}

impl Drawable for Cell {
    fn emit_cells(&self, f: &mut dyn FnMut(Cell)) {
        f(*self);
    }
}

impl<A: Drawable, B: Drawable> Drawable for (A, B) {
    fn emit_cells(&self, f: &mut dyn FnMut(Cell)) {
        self.0.emit_cells(f);
        self.1.emit_cells(f);
    }
}

impl<A: Drawable, B: Drawable, C: Drawable> Drawable for (A, B, C) {
    fn emit_cells(&self, f: &mut dyn FnMut(Cell)) {
        self.0.emit_cells(f);
        self.1.emit_cells(f);
        self.2.emit_cells(f);
    }
}

impl<D: Drawable> Drawable for Vec<D> {
    fn emit_cells(&self, f: &mut dyn FnMut(Cell)) {
        for d in self {
            d.emit_cells(f);
        }
    }
}

impl<D: Drawable, const N: usize> Drawable for [D; N] {
    fn emit_cells(&self, f: &mut dyn FnMut(Cell)) {
        for d in self {
            d.emit_cells(f);
        }
    }
}

pub struct Transformed<T: Transformable + Sized> {
    flip_x: bool,
    flip_y: bool,
    rotate_90: bool,
    rotate_180: bool,
    translate: (f32, f32),
    inner: T,
}

pub trait Transformable {
    fn original_position(&self) -> (f32, f32);
    fn emit_local_cells(&self, f: &mut dyn FnMut(Cell));

    fn transformed(self) -> Transformed<Self>
    where
        Self: Sized,
    {
        Transformed {
            flip_x: false,
            flip_y: false,
            rotate_90: false,
            rotate_180: false,
            translate: (0.0, 0.0),
            inner: self,
        }
    }

    fn flip_y(self) -> Transformed<Self>
    where
        Self: Sized,
    {
        self.transformed().flip_y()
    }

    fn flip_x(self) -> Transformed<Self>
    where
        Self: Sized,
    {
        self.transformed().flip_x()
    }

    fn rotate_90_cw(self) -> Transformed<Self>
    where
        Self: Sized,
    {
        self.transformed().rotate_90_cw()
    }

    fn rotate_90_ccw(self) -> Transformed<Self>
    where
        Self: Sized,
    {
        self.transformed().rotate_90_ccw()
    }

    fn rotate_180(self) -> Transformed<Self>
    where
        Self: Sized,
    {
        self.transformed().rotate_180()
    }

    fn translate(self, dx: f32, dy: f32) -> Transformed<Self>
    where
        Self: Sized,
    {
        self.transformed().translate(dx, dy)
    }

    fn translate_x(self, dx: f32) -> Transformed<Self>
    where
        Self: Sized,
    {
        self.transformed().translate_x(dx)
    }

    fn translate_y(self, dy: f32) -> Transformed<Self>
    where
        Self: Sized,
    {
        self.transformed().translate_y(dy)
    }
}

impl<T: Transformable> Transformed<T> {
    fn flip_y(mut self) -> Transformed<T> {
        self.flip_y = !self.flip_y;
        self
    }

    fn flip_x(mut self) -> Transformed<T> {
        self.flip_x = !self.flip_x;
        self
    }

    fn rotate_90_cw(mut self) -> Transformed<T> {
        match (self.rotate_180, self.rotate_90) {
            (false, false) => {
                self.rotate_90 = true;
            } // 0 -> 90
            (false, true) => {
                self.rotate_180 = true;
                self.rotate_90 = false;
            } // 90 -> 180
            (true, false) => {
                self.rotate_180 = true;
                self.rotate_90 = true;
            } // 180 -> 270
            (true, true) => {
                self.rotate_180 = false;
                self.rotate_90 = false;
            } // 270 -> 0
        }
        self
    }

    fn rotate_90_ccw(mut self) -> Transformed<T> {
        match (self.rotate_180, self.rotate_90) {
            (false, false) => {
                self.rotate_180 = true;
                self.rotate_90 = true;
            } // 0 -> 270
            (false, true) => {
                self.rotate_90 = false;
            } // 90 -> 0
            (true, false) => {
                self.rotate_90 = true;
            } // 180 -> 90
            (true, true) => {
                self.rotate_180 = false;
            } // 270 -> 180
        }
        self
    }

    fn rotate_180(mut self) -> Transformed<T> {
        self.rotate_180 = !self.rotate_180;
        self
    }

    fn translate(self, dx: f32, dy: f32) -> Transformed<T> {
        self.translate_x(dx).translate_y(dy)
    }

    fn translate_x(mut self, dx: f32) -> Transformed<T> {
        self.translate.0 += dx;
        self
    }

    fn translate_y(mut self, dy: f32) -> Transformed<T> {
        self.translate.1 += dy;
        self
    }
}

impl<T: Transformable> Drawable for T {
    fn emit_cells(&self, f: &mut dyn FnMut(Cell)) {
        let position = self.original_position();
        self.emit_local_cells(&mut |mut cell| {
            cell.position.x += position.0;
            cell.position.y += position.1;
            f(cell);
        });
    }
}

impl<T: Transformable> Drawable for Transformed<T> {
    fn emit_cells(&self, f: &mut dyn FnMut(Cell)) {
        let position = self.inner.original_position();
        self.inner.emit_local_cells(&mut |mut cell| {
            if self.flip_x {
                cell.position.x = -cell.position.x;
            }
            if self.flip_y {
                cell.position.y = -cell.position.y;
            }
            if self.rotate_90 {
                let (nx, ny) = (-cell.position.y, cell.position.x);
                cell.position.x = nx;
                cell.position.y = ny;
            }
            if self.rotate_180 {
                let (nx, ny) = (-cell.position.x, -cell.position.y);
                cell.position.x = nx;
                cell.position.y = ny;
            }

            cell.position.x += self.translate.0;
            cell.position.y += self.translate.1;

            cell.position.x += position.0;
            cell.position.y += position.1;
            f(cell);
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::Vec3;

    struct TestShape {
        origin: (f32, f32),
        cells: Vec<(f32, f32)>,
    }

    impl TestShape {
        fn unit() -> Self {
            Self {
                origin: (0.0, 0.0),
                cells: vec![(1.0, 0.0)],
            }
        }

        fn at(origin: (f32, f32), cells: Vec<(f32, f32)>) -> Self {
            Self { origin, cells }
        }
    }

    impl Transformable for TestShape {
        fn original_position(&self) -> (f32, f32) {
            self.origin
        }

        fn emit_local_cells(&self, f: &mut dyn FnMut(Cell)) {
            for &(x, y) in &self.cells {
                f(Cell::new(x, y));
            }
        }
    }

    fn positions(d: &impl Drawable) -> Vec<(f32, f32)> {
        let cells = d.to_cells();
        cells.iter().map(|c| (c.position.x, c.position.y)).collect()
    }

    fn approx_eq(a: (f32, f32), b: (f32, f32)) -> bool {
        (a.0 - b.0).abs() < 1e-5 && (a.1 - b.1).abs() < 1e-5
    }

    // --- Drawable trait ---

    #[test]
    fn cell_emits_itself() {
        let c = Cell::new(3.0, 7.0);
        let cells = c.to_cells();
        assert_eq!(cells.len(), 1);
        assert_eq!(cells[0].position, Vec3::new(3.0, 7.0, 0.0));
    }

    #[test]
    fn tuple_2_emits_both() {
        let a = Cell::new(1.0, 2.0);
        let b = Cell::new(3.0, 4.0);
        let cells = (a, b).to_cells();
        assert_eq!(cells.len(), 2);
        assert_eq!(cells[0].position, Vec3::new(1.0, 2.0, 0.0));
        assert_eq!(cells[1].position, Vec3::new(3.0, 4.0, 0.0));
    }

    #[test]
    fn tuple_3_emits_all() {
        let a = Cell::new(1.0, 0.0);
        let b = Cell::new(2.0, 0.0);
        let c = Cell::new(3.0, 0.0);
        let cells = (a, b, c).to_cells();
        assert_eq!(cells.len(), 3);
    }

    #[test]
    fn vec_emits_all() {
        let v = vec![
            Cell::new(0.0, 0.0),
            Cell::new(1.0, 1.0),
            Cell::new(2.0, 2.0),
        ];
        let cells = v.to_cells();
        assert_eq!(cells.len(), 3);
    }

    #[test]
    fn array_emits_all() {
        let arr = [Cell::new(0.0, 0.0), Cell::new(5.0, 5.0)];
        let cells = arr.to_cells();
        assert_eq!(cells.len(), 2);
        assert_eq!(cells[1].position, Vec3::new(5.0, 5.0, 0.0));
    }

    #[test]
    fn colored_overrides_color() {
        let c = Cell::new(0.0, 0.0);
        let red = Color::linear(1.0, 0.0, 0.0, 1.0);
        let colored = c.color(red);
        let cells = colored.to_cells();
        assert_eq!(cells[0].color, [1.0, 0.0, 0.0, 1.0]);
    }

    #[test]
    fn colored_preserves_position() {
        let c = Cell::new(4.0, 5.0);
        let colored = c.color(Color::linear(0.0, 1.0, 0.0, 1.0));
        let cells = colored.to_cells();
        assert_eq!(cells[0].position, Vec3::new(4.0, 5.0, 0.0));
    }

    // --- Transformable: base emit ---

    #[test]
    fn transformable_adds_origin() {
        let s = TestShape::at((10.0, 20.0), vec![(1.0, 2.0)]);
        let pos = positions(&s);
        assert!(approx_eq(pos[0], (11.0, 22.0)));
    }

    #[test]
    fn transformable_zero_origin() {
        let s = TestShape::unit();
        let pos = positions(&s);
        assert!(approx_eq(pos[0], (1.0, 0.0)));
    }

    // --- Transformed: translate ---

    #[test]
    fn translate_x() {
        let s = TestShape::unit().translate_x(5.0);
        let pos = positions(&s);
        assert!(approx_eq(pos[0], (6.0, 0.0)));
    }

    #[test]
    fn translate_y() {
        let s = TestShape::unit().translate_y(3.0);
        let pos = positions(&s);
        assert!(approx_eq(pos[0], (1.0, 3.0)));
    }

    #[test]
    fn translate_xy() {
        let s = TestShape::unit().translate(2.0, 4.0);
        let pos = positions(&s);
        assert!(approx_eq(pos[0], (3.0, 4.0)));
    }

    #[test]
    fn translate_with_origin() {
        let s = TestShape::at((10.0, 10.0), vec![(1.0, 1.0)]).translate(5.0, 5.0);
        let pos = positions(&s);
        assert!(approx_eq(pos[0], (16.0, 16.0)));
    }

    // --- Transformed: flip ---

    #[test]
    fn flip_x() {
        let s = TestShape::unit().flip_x();
        let pos = positions(&s);
        assert!(approx_eq(pos[0], (-1.0, 0.0)));
    }

    #[test]
    fn flip_y() {
        let s = TestShape::at((0.0, 0.0), vec![(0.0, 3.0)]).flip_y();
        let pos = positions(&s);
        assert!(approx_eq(pos[0], (0.0, -3.0)));
    }

    #[test]
    fn flip_x_twice_is_identity() {
        let s = TestShape::unit().flip_x().flip_x();
        let pos = positions(&s);
        assert!(approx_eq(pos[0], (1.0, 0.0)));
    }

    #[test]
    fn flip_y_twice_is_identity() {
        let s = TestShape::at((0.0, 0.0), vec![(0.0, 5.0)])
            .flip_y()
            .flip_y();
        let pos = positions(&s);
        assert!(approx_eq(pos[0], (0.0, 5.0)));
    }

    // --- Transformed: rotate ---

    #[test]
    fn rotate_90_cw() {
        // (1, 0) -> rotate_90 flag: (-y, x) = (0, 1)
        let s = TestShape::unit().rotate_90_cw();
        let pos = positions(&s);
        assert!(approx_eq(pos[0], (0.0, 1.0)));
    }

    #[test]
    fn rotate_90_ccw() {
        // (1, 0) -> 270 = rotate_180 + rotate_90
        // rotate_90: (-y, x) = (0, 1), then rotate_180: (0, -1)
        let s = TestShape::unit().rotate_90_ccw();
        let pos = positions(&s);
        assert!(approx_eq(pos[0], (0.0, -1.0)));
    }

    #[test]
    fn rotate_180() {
        // (1, 0) -> (-1, 0)
        let s = TestShape::unit().rotate_180();
        let pos = positions(&s);
        assert!(approx_eq(pos[0], (-1.0, 0.0)));
    }

    #[test]
    fn rotate_90_cw_four_times_is_identity() {
        let s = TestShape::unit()
            .rotate_90_cw()
            .rotate_90_cw()
            .rotate_90_cw()
            .rotate_90_cw();
        let pos = positions(&s);
        assert!(approx_eq(pos[0], (1.0, 0.0)));
    }

    #[test]
    fn rotate_180_twice_is_identity() {
        let s = TestShape::unit().rotate_180().rotate_180();
        let pos = positions(&s);
        assert!(approx_eq(pos[0], (1.0, 0.0)));
    }

    #[test]
    fn rotate_90_cw_then_ccw_is_identity() {
        let s = TestShape::unit().rotate_90_cw().rotate_90_ccw();
        let pos = positions(&s);
        assert!(approx_eq(pos[0], (1.0, 0.0)));
    }

    // --- Transformed: combined ---

    #[test]
    fn flip_x_then_translate() {
        let s = TestShape::unit().flip_x().translate(10.0, 0.0);
        let pos = positions(&s);
        assert!(approx_eq(pos[0], (9.0, 0.0)));
    }

    #[test]
    fn rotate_then_translate() {
        let s = TestShape::unit().rotate_90_cw().translate(5.0, 5.0);
        let pos = positions(&s);
        assert!(approx_eq(pos[0], (5.0, 6.0)));
    }

    #[test]
    fn multiple_cells_transform() {
        let s = TestShape::at((0.0, 0.0), vec![(1.0, 0.0), (0.0, 1.0), (-1.0, 0.0)]).rotate_90_cw();
        let pos = positions(&s);
        assert!(approx_eq(pos[0], (0.0, 1.0)));
        assert!(approx_eq(pos[1], (-1.0, 0.0)));
        assert!(approx_eq(pos[2], (0.0, -1.0)));
    }

    #[test]
    fn transform_with_origin_offset() {
        let s = TestShape::at((5.0, 5.0), vec![(1.0, 0.0)]).flip_x();
        let pos = positions(&s);
        // flip_x negates local x: -1, then adds origin: (-1+5, 0+5) = (4, 5)
        assert!(approx_eq(pos[0], (4.0, 5.0)));
    }
}
