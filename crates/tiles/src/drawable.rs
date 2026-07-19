use crate::cell::Cell;
use crate::color::Color;

pub struct Mapped<T: Drawable, F: Fn(Cell) -> Cell> {
    inner: T,
    map: F,
}

pub trait Drawable {
    fn origin(&self) -> Option<(f32, f32)> {
        None
    }

    fn emit_local_cells(&self, f: &mut dyn FnMut(Cell));

    fn emit_cells(&self, f: &mut dyn FnMut(Cell)) {
        if let Some((ox, oy)) = self.origin() {
            self.emit_local_cells(&mut |mut cell| {
                cell.position.x += ox;
                cell.position.y += oy;
                f(cell);
            });
        } else {
            self.emit_local_cells(f);
        }
    }

    fn to_local_cells(&self) -> Vec<Cell> {
        let mut cells = vec![];
        self.emit_local_cells(&mut |cell| cells.push(cell));
        cells
    }

    fn to_cells(&self) -> Vec<Cell> {
        let mut cells = vec![];
        self.emit_cells(&mut |cell| cells.push(cell));
        cells
    }

    fn map_cell(self, map: impl Fn(Cell) -> Cell) -> Mapped<Self, impl Fn(Cell) -> Cell>
    where
        Self: Sized,
    {
        Mapped { inner: self, map }
    }

    fn color(self, c: Color) -> Mapped<Self, impl Fn(Cell) -> Cell>
    where
        Self: Sized,
    {
        self.map_cell(move |mut cell| {
            cell.color = c.to_array();
            cell
        })
    }

    fn flip_y(self) -> Mapped<Self, impl Fn(Cell) -> Cell>
    where
        Self: Sized,
    {
        self.map_cell(|mut cell| {
            cell.position.y = -cell.position.y;
            cell
        })
    }

    fn flip_x(self) -> Mapped<Self, impl Fn(Cell) -> Cell>
    where
        Self: Sized,
    {
        self.map_cell(|mut cell| {
            cell.position.x = -cell.position.x;
            cell
        })
    }

    fn rotate_90_cw(self) -> Mapped<Self, impl Fn(Cell) -> Cell>
    where
        Self: Sized,
    {
        self.map_cell(|mut cell| {
            let (x, y) = (cell.position.x, cell.position.y);
            cell.position.x = -y;
            cell.position.y = x;
            cell
        })
    }

    fn rotate_90_ccw(self) -> Mapped<Self, impl Fn(Cell) -> Cell>
    where
        Self: Sized,
    {
        self.map_cell(|mut cell| {
            let (x, y) = (cell.position.x, cell.position.y);
            cell.position.x = y;
            cell.position.y = -x;
            cell
        })
    }

    fn rotate_180(self) -> Mapped<Self, impl Fn(Cell) -> Cell>
    where
        Self: Sized,
    {
        self.map_cell(|mut cell| {
            cell.position.x = -cell.position.x;
            cell.position.y = -cell.position.y;
            cell
        })
    }

    fn translate(self, dx: f32, dy: f32) -> Mapped<Self, impl Fn(Cell) -> Cell>
    where
        Self: Sized,
    {
        self.map_cell(move |mut cell| {
            cell.position.x += dx;
            cell.position.y += dy;
            cell
        })
    }

    fn translate_x(self, dx: f32) -> Mapped<Self, impl Fn(Cell) -> Cell>
    where
        Self: Sized,
    {
        self.translate(dx, 0.0)
    }

    fn translate_y(self, dy: f32) -> Mapped<Self, impl Fn(Cell) -> Cell>
    where
        Self: Sized,
    {
        self.translate(0.0, dy)
    }
}

impl<T: Drawable, F: Fn(Cell) -> Cell> Drawable for Mapped<T, F> {
    fn origin(&self) -> Option<(f32, f32)> {
        self.inner.origin()
    }

    fn emit_local_cells(&self, f: &mut dyn FnMut(Cell)) {
        self.inner.emit_local_cells(&mut |cell| {
            f((self.map)(cell));
        });
    }
}

impl Drawable for Cell {
    fn emit_local_cells(&self, f: &mut dyn FnMut(Cell)) {
        f(*self);
    }
}

impl<A: Drawable, B: Drawable> Drawable for (A, B) {
    fn emit_local_cells(&self, f: &mut dyn FnMut(Cell)) {
        self.0.emit_cells(f);
        self.1.emit_cells(f);
    }
}

impl<A: Drawable, B: Drawable, C: Drawable> Drawable for (A, B, C) {
    fn emit_local_cells(&self, f: &mut dyn FnMut(Cell)) {
        self.0.emit_cells(f);
        self.1.emit_cells(f);
        self.2.emit_cells(f);
    }
}

impl<D: Drawable> Drawable for Vec<D> {
    fn emit_local_cells(&self, f: &mut dyn FnMut(Cell)) {
        for d in self {
            d.emit_cells(f);
        }
    }
}

impl<D: Drawable, const N: usize> Drawable for [D; N] {
    fn emit_local_cells(&self, f: &mut dyn FnMut(Cell)) {
        for d in self {
            d.emit_cells(f);
        }
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

    impl Drawable for TestShape {
        fn origin(&self) -> Option<(f32, f32)> {
            Some(self.origin)
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

    // --- Origin ---

    #[test]
    fn drawable_adds_origin() {
        let s = TestShape::at((10.0, 20.0), vec![(1.0, 2.0)]);
        let pos = positions(&s);
        assert!(approx_eq(pos[0], (11.0, 22.0)));
    }

    #[test]
    fn drawable_zero_origin() {
        let s = TestShape::unit();
        let pos = positions(&s);
        assert!(approx_eq(pos[0], (1.0, 0.0)));
    }

    // --- Translate ---

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

    // --- Flip ---

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

    // --- Rotate ---

    #[test]
    fn rotate_90_cw() {
        let s = TestShape::unit().rotate_90_cw();
        let pos = positions(&s);
        assert!(approx_eq(pos[0], (0.0, 1.0)));
    }

    #[test]
    fn rotate_90_ccw() {
        let s = TestShape::unit().rotate_90_ccw();
        let pos = positions(&s);
        assert!(approx_eq(pos[0], (0.0, -1.0)));
    }

    #[test]
    fn rotate_180() {
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

    // --- Combined ---

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
        assert!(approx_eq(pos[0], (4.0, 5.0)));
    }

    // --- Chaining color + transforms ---

    #[test]
    fn color_then_flip() {
        let s = TestShape::at((0.0, 0.0), vec![(1.0, 2.0)])
            .color(Color::linear(1.0, 0.0, 0.0, 1.0))
            .flip_y();
        let cells = s.to_cells();
        assert_eq!(cells[0].color, [1.0, 0.0, 0.0, 1.0]);
        assert!(approx_eq(
            (cells[0].position.x, cells[0].position.y),
            (1.0, -2.0)
        ));
    }

    #[test]
    fn flip_then_color() {
        let s = TestShape::at((0.0, 0.0), vec![(1.0, 2.0)])
            .flip_y()
            .color(Color::linear(0.0, 1.0, 0.0, 1.0));
        let cells = s.to_cells();
        assert_eq!(cells[0].color, [0.0, 1.0, 0.0, 1.0]);
        assert!(approx_eq(
            (cells[0].position.x, cells[0].position.y),
            (1.0, -2.0)
        ));
    }
}
