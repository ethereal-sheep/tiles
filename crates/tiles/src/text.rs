use crate::cell::Cell;
use crate::color::Color;
use crate::drawable::Drawable;
use crate::font::Font;
use crate::rect::Rect;

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub(crate) enum AnchorBox {
    #[default]
    Highlight,
    Tight,
}

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub(crate) enum AnchorCorner {
    #[default]
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
    TopCenter,
    BottomCenter,
    CenterLeft,
    CenterRight,
    Center,
}

pub struct Text {
    font: &'static Font,
    content: String,
    position: (f32, f32),
    anchor_box: AnchorBox,
    anchor_corner: AnchorCorner,
    gap_override: Option<usize>,
    color: Color,
    color_map: Option<Box<dyn Fn(usize, char) -> Color>>,
    position_map: Option<Box<dyn Fn(usize, char) -> (f32, f32)>>,
    // Content-derived, independent of position/anchor
    highlight_size: (u32, u32),
    tight_offset: (f32, f32), // from layout origin to tight box top-left
    tight_size: (u32, u32),
}

impl Text {
    pub fn new(font: &'static Font, content: impl Into<String>) -> Self {
        let content = content.into();
        let highlight_size = compute_highlight_box(font, &content, None);
        let (tight_offset, tight_size) = compute_tight_info(font, &content, None, None);

        Self {
            font,
            content,
            position: (0.0, 0.0),
            anchor_box: AnchorBox::default(),
            anchor_corner: AnchorCorner::default(),
            gap_override: None,
            color: Color::linear(1.0, 1.0, 1.0, 1.0),
            color_map: None,
            position_map: None,
            highlight_size,
            tight_offset,
            tight_size,
        }
    }

    pub fn position(mut self, x: f32, y: f32) -> Self {
        self.position = (x, y);
        self
    }

    pub fn tight(mut self) -> Self {
        self.anchor_box = AnchorBox::Tight;
        self
    }

    pub fn top_left(mut self) -> Self {
        self.anchor_corner = AnchorCorner::TopLeft;
        self
    }

    pub fn top_right(mut self) -> Self {
        self.anchor_corner = AnchorCorner::TopRight;
        self
    }

    pub fn bottom_left(mut self) -> Self {
        self.anchor_corner = AnchorCorner::BottomLeft;
        self
    }

    pub fn bottom_right(mut self) -> Self {
        self.anchor_corner = AnchorCorner::BottomRight;
        self
    }

    pub fn top_center(mut self) -> Self {
        self.anchor_corner = AnchorCorner::TopCenter;
        self
    }

    pub fn bottom_center(mut self) -> Self {
        self.anchor_corner = AnchorCorner::BottomCenter;
        self
    }

    pub fn center_left(mut self) -> Self {
        self.anchor_corner = AnchorCorner::CenterLeft;
        self
    }

    pub fn center_right(mut self) -> Self {
        self.anchor_corner = AnchorCorner::CenterRight;
        self
    }

    pub fn center(mut self) -> Self {
        self.anchor_corner = AnchorCorner::Center;
        self
    }


    pub fn gap(mut self, gap: usize) -> Self {
        self.gap_override = Some(gap);
        self.highlight_size = compute_highlight_box(self.font, &self.content, self.gap_override);
        self.recompute_tight();
        self
    }

    pub fn map_color(mut self, f: impl Fn(usize, char) -> Color + 'static) -> Self {
        self.color_map = Some(Box::new(f));
        self
    }

    pub fn map_position(mut self, f: impl Fn(usize, char) -> (f32, f32) + 'static) -> Self {
        self.position_map = Some(Box::new(f));
        self.recompute_tight();
        self
    }

    pub fn color(mut self, c: Color) -> Self {
        self.color = c;
        self
    }

    pub fn width(&self) -> u32 {
        self.highlight_size.0
    }

    pub fn height(&self) -> u32 {
        self.highlight_size.1
    }

    pub fn bounds(&self) -> Rect {
        let (ox, oy) = self.layout_origin();
        Rect::from_top_left(
            ox + self.tight_offset.0,
            oy + self.tight_offset.1,
            self.tight_size.0,
            self.tight_size.1,
        )
    }

    pub fn rect(&self) -> Rect {
        let (ox, oy) = self.layout_origin();
        Rect::from_top_left(ox, oy, self.highlight_size.0, self.highlight_size.1)
    }

    pub fn font(&self) -> &'static Font {
        self.font
    }

    fn char_step(&self, ch: char) -> u32 {
        char_step(self.font, ch, self.gap_override)
    }

    fn recompute_tight(&mut self) {
        let (tight_offset, tight_size) = compute_tight_info(
            self.font,
            &self.content,
            self.gap_override,
            self.position_map.as_deref(),
        );
        self.tight_offset = tight_offset;
        self.tight_size = tight_size;
    }

    fn anchor_offset(&self) -> (f32, f32) {
        let (box_w, box_h, box_offset_x, box_offset_y) = match self.anchor_box {
            AnchorBox::Highlight => (
                self.highlight_size.0 as f32,
                self.highlight_size.1 as f32,
                0.0,
                0.0,
            ),
            AnchorBox::Tight => (
                self.tight_size.0 as f32,
                self.tight_size.1 as f32,
                self.tight_offset.0,
                self.tight_offset.1,
            ),
        };

        let half_w = box_w / 2.0;
        let half_h = box_h / 2.0;

        match self.anchor_corner {
            AnchorCorner::TopLeft => (-box_offset_x, -box_offset_y),
            AnchorCorner::TopRight => (-box_w - box_offset_x, -box_offset_y),
            AnchorCorner::BottomLeft => (-box_offset_x, -box_h - box_offset_y),
            AnchorCorner::BottomRight => (-box_w - box_offset_x, -box_h - box_offset_y),
            AnchorCorner::TopCenter => (-half_w - box_offset_x, -box_offset_y),
            AnchorCorner::BottomCenter => (-half_w - box_offset_x, -box_h - box_offset_y),
            AnchorCorner::CenterLeft => (-box_offset_x, -half_h - box_offset_y),
            AnchorCorner::CenterRight => (-box_w - box_offset_x, -half_h - box_offset_y),
            AnchorCorner::Center => (-half_w - box_offset_x, -half_h - box_offset_y),
        }
    }

    fn layout_origin(&self) -> (f32, f32) {
        let (ax, ay) = self.anchor_offset();
        (self.position.0 + ax, self.position.1 + ay)
    }
}

impl Drawable for Text {
    fn origin(&self) -> Option<(f32, f32)> {
        Some(self.position)
    }

    fn emit_local_cells(&self, f: &mut dyn FnMut(Cell)) {
        let (ax, ay) = self.anchor_offset();
        let mut cursor_x = 0u32;

        for (i, ch) in self.content.chars().enumerate() {
            if let Some(glyph) = self.font.glyph(ch) {
                if glyph.width == 0 || glyph.height == 0 {
                    cursor_x += self.char_step(ch);
                    continue;
                }

                let (dx, dy) = self
                    .position_map
                    .as_ref()
                    .map(|pm| pm(i, ch))
                    .unwrap_or((0.0, 0.0));
                let color = self
                    .color_map
                    .as_ref()
                    .map(|cm| cm(i, ch))
                    .unwrap_or(self.color);

                let char_x = ax + cursor_x as f32 + dx;
                let char_y = ay + glyph.top as f32 + dy;

                for row in 0..glyph.height as usize {
                    for col in 0..glyph.width as usize {
                        if glyph.pixel(col, row) {
                            let cell =
                                Cell::new(char_x + col as f32, char_y + row as f32).color(color);
                            f(cell);
                        }
                    }
                }

                cursor_x += self.char_step(ch);
            }
        }
    }
}

fn char_step(font: &Font, ch: char, gap_override: Option<usize>) -> u32 {
    let w = font.glyph_width(ch);
    if w == 0 {
        return 0;
    }
    let gap = gap_override.unwrap_or(font.default_gap);
    w as u32 + gap as u32
}

fn compute_highlight_box(font: &Font, content: &str, gap_override: Option<usize>) -> (u32, u32) {
    let chars: Vec<char> = content.chars().collect();
    if chars.is_empty() {
        return (0, font.height as u32);
    }

    let mut width = 0u32;
    for (i, &ch) in chars.iter().enumerate() {
        if i < chars.len() - 1 {
            width += char_step(font, ch, gap_override);
        } else {
            width += font.glyph_width(ch) as u32;
        }
    }

    (width, font.height as u32)
}

fn compute_tight_info(
    font: &Font,
    content: &str,
    gap_override: Option<usize>,
    position_map: Option<&dyn Fn(usize, char) -> (f32, f32)>,
) -> ((f32, f32), (u32, u32)) {
    let mut min_x = f32::MAX;
    let mut min_y = f32::MAX;
    let mut max_x = f32::MIN;
    let mut max_y = f32::MIN;
    let mut cursor_x = 0u32;
    let mut has_pixels = false;

    for (i, ch) in content.chars().enumerate() {
        if let Some(glyph) = font.glyph(ch) {
            if glyph.width == 0 || glyph.height == 0 {
                cursor_x += char_step(font, ch, gap_override);
                continue;
            }

            let (dx, dy) = position_map.map(|pm| pm(i, ch)).unwrap_or((0.0, 0.0));

            let glyph_left = cursor_x as f32 + dx;
            let glyph_top = glyph.top as f32 + dy;
            let glyph_right = glyph_left + glyph.width as f32 - 1.0;
            let glyph_bottom = glyph_top + glyph.height as f32 - 1.0;

            min_x = min_x.min(glyph_left);
            min_y = min_y.min(glyph_top);
            max_x = max_x.max(glyph_right);
            max_y = max_y.max(glyph_bottom);
            has_pixels = true;

            cursor_x += char_step(font, ch, gap_override);
        }
    }

    if !has_pixels {
        return ((0.0, 0.0), (0, 0));
    }

    let w = (max_x - min_x + 1.0) as u32;
    let h = (max_y - min_y + 1.0) as u32;
    ((min_x, min_y), (w, h))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::Color;
    use crate::drawable::Drawable;
    use crate::font::{MONO_5X7, TOM_THUMB_3X5};

    #[test]
    fn text_width_and_height() {
        let t = Text::new(&TOM_THUMB_3X5, "A");
        assert!(t.width() > 0);
        assert_eq!(t.height(), TOM_THUMB_3X5.height as u32);
    }

    #[test]
    fn text_multi_char_width() {
        let single = Text::new(&TOM_THUMB_3X5, "A");
        let double = Text::new(&TOM_THUMB_3X5, "AA");
        assert!(double.width() > single.width());
    }

    #[test]
    fn text_gap_affects_width() {
        let narrow = Text::new(&TOM_THUMB_3X5, "AB").gap(0);
        let wide = Text::new(&TOM_THUMB_3X5, "AB").gap(3);
        assert!(wide.width() > narrow.width());
    }

    #[test]
    fn text_mono_font_uniform_advance() {
        let t = Text::new(&MONO_5X7, "AB");
        let t2 = Text::new(&MONO_5X7, "ii");
        assert_eq!(
            t.width(),
            t2.width(),
            "Mono font should have uniform text width for same char count"
        );
    }

    #[test]
    fn text_empty_string() {
        let t = Text::new(&TOM_THUMB_3X5, "");
        assert_eq!(t.width(), 0);
        assert_eq!(t.height(), TOM_THUMB_3X5.height as u32);
    }

    #[test]
    fn text_emit_cells_produces_cells() {
        let t = Text::new(&TOM_THUMB_3X5, "I").position(0.0, 0.0);
        let mut cells = Vec::new();
        t.emit_cells(&mut |c| cells.push(c));
        assert!(!cells.is_empty(), "Should emit at least one cell");
    }

    #[test]
    fn text_emit_cells_empty_string() {
        let t = Text::new(&TOM_THUMB_3X5, "");
        let mut cells = Vec::new();
        t.emit_cells(&mut |c| cells.push(c));
        assert!(cells.is_empty());
    }

    #[test]
    fn text_position_offsets_cells() {
        let t1 = Text::new(&TOM_THUMB_3X5, "I").position(0.0, 0.0);
        let t2 = Text::new(&TOM_THUMB_3X5, "I").position(10.0, 20.0);

        let mut cells1 = Vec::new();
        let mut cells2 = Vec::new();
        t1.emit_cells(&mut |c| cells1.push(c));
        t2.emit_cells(&mut |c| cells2.push(c));

        assert_eq!(cells1.len(), cells2.len());
        for (c1, c2) in cells1.iter().zip(cells2.iter()) {
            assert!((c2.position.x - c1.position.x - 10.0).abs() < 0.001);
            assert!((c2.position.y - c1.position.y - 20.0).abs() < 0.001);
        }
    }

    #[test]
    fn text_position_does_not_affect_size() {
        let t1 = Text::new(&TOM_THUMB_3X5, "Hi");
        let t2 = Text::new(&TOM_THUMB_3X5, "Hi").position(100.0, 200.0);
        assert_eq!(t1.width(), t2.width());
        assert_eq!(t1.height(), t2.height());
        assert_eq!(t1.bounds().width(), t2.bounds().width());
        assert_eq!(t1.bounds().height(), t2.bounds().height());
    }

    #[test]
    fn text_color_applies() {
        let t = Text::new(&TOM_THUMB_3X5, "I").color(Color::linear(1.0, 0.0, 0.0, 1.0));
        let mut cells = Vec::new();
        t.emit_cells(&mut |c| cells.push(c));
        for cell in &cells {
            assert_eq!(cell.color, [1.0, 0.0, 0.0, 1.0]);
        }
    }

    #[test]
    fn text_map_color_overrides() {
        let t = Text::new(&TOM_THUMB_3X5, "AB").map_color(|i, _| {
            if i == 0 {
                Color::linear(1.0, 0.0, 0.0, 1.0)
            } else {
                Color::linear(0.0, 1.0, 0.0, 1.0)
            }
        });

        let mut cells = Vec::new();
        t.emit_cells(&mut |c| cells.push(c));
        assert!(!cells.is_empty());
    }

    #[test]
    fn text_map_position_shifts_chars() {
        let base = Text::new(&TOM_THUMB_3X5, "I").position(0.0, 0.0);
        let shifted = Text::new(&TOM_THUMB_3X5, "I")
            .position(0.0, 0.0)
            .map_position(|_, _| (5.0, 3.0));

        let mut base_cells = Vec::new();
        let mut shifted_cells = Vec::new();
        base.emit_cells(&mut |c| base_cells.push(c));
        shifted.emit_cells(&mut |c| shifted_cells.push(c));

        assert_eq!(base_cells.len(), shifted_cells.len());
        for (bc, sc) in base_cells.iter().zip(shifted_cells.iter()) {
            assert!((sc.position.x - bc.position.x - 5.0).abs() < 0.001);
            assert!((sc.position.y - bc.position.y - 3.0).abs() < 0.001);
        }
    }

    #[test]
    fn text_bounds_encloses_emitted_cells() {
        let t = Text::new(&TOM_THUMB_3X5, "Hi").position(10.0, 20.0);
        let bounds = t.bounds();

        let mut cells = Vec::new();
        t.emit_cells(&mut |c| cells.push(c));

        for cell in &cells {
            assert!(
                cell.position.x >= bounds.left() && cell.position.x <= bounds.right(),
                "Cell x={} outside bounds [{}, {}]",
                cell.position.x,
                bounds.left(),
                bounds.right()
            );
            assert!(
                cell.position.y >= bounds.top() && cell.position.y <= bounds.bottom(),
                "Cell y={} outside bounds [{}, {}]",
                cell.position.y,
                bounds.top(),
                bounds.bottom()
            );
        }
    }

    #[test]
    fn text_bounds_with_position_map() {
        let t = Text::new(&TOM_THUMB_3X5, "Hi")
            .position(0.0, 0.0)
            .map_position(|i, _| if i == 0 { (0.0, -2.0) } else { (0.0, 2.0) });

        let bounds = t.bounds();

        let mut cells = Vec::new();
        t.emit_cells(&mut |c| cells.push(c));

        for cell in &cells {
            assert!(
                cell.position.x >= bounds.left() && cell.position.x <= bounds.right(),
                "Cell x={} outside bounds [{}, {}]",
                cell.position.x,
                bounds.left(),
                bounds.right()
            );
            assert!(
                cell.position.y >= bounds.top() && cell.position.y <= bounds.bottom(),
                "Cell y={} outside bounds [{}, {}]",
                cell.position.y,
                bounds.top(),
                bounds.bottom()
            );
        }
    }

    #[test]
    fn text_anchor_top_right() {
        let t = Text::new(&TOM_THUMB_3X5, "Hi")
            .position(50.0, 10.0)
            .top_right();

        let mut cells = Vec::new();
        t.emit_cells(&mut |c| cells.push(c));
        assert!(!cells.is_empty());

        for cell in &cells {
            assert!(
                cell.position.x <= 50.0,
                "All cells should be left of anchor x"
            );
        }
    }

    #[test]
    fn text_anchor_tight() {
        let at_origin = Text::new(&TOM_THUMB_3X5, "A")
            .position(0.0, 0.0)
            .tight()
            .top_left();

        let mut cells = Vec::new();
        at_origin.emit_cells(&mut |c| cells.push(c));
        assert!(!cells.is_empty());

        let min_x = cells.iter().map(|c| c.position.x).fold(f32::MAX, f32::min);
        let min_y = cells.iter().map(|c| c.position.y).fold(f32::MAX, f32::min);
        assert!(
            (min_x - 0.0).abs() < 0.001,
            "Tight anchor should start cells at x=0, got {min_x}"
        );
        assert!(
            (min_y - 0.0).abs() < 0.001,
            "Tight anchor should start cells at y=0, got {min_y}"
        );
    }

    #[test]
    fn text_anchor_bottom_left() {
        let t = Text::new(&TOM_THUMB_3X5, "Hi")
            .position(10.0, 50.0)
            .bottom_left();

        let mut cells = Vec::new();
        t.emit_cells(&mut |c| cells.push(c));
        assert!(!cells.is_empty());

        for cell in &cells {
            assert!(
                cell.position.y <= 50.0,
                "All cells should be above anchor y, got {}",
                cell.position.y
            );
            assert!(
                cell.position.x >= 10.0,
                "All cells should be right of anchor x, got {}",
                cell.position.x
            );
        }
    }

    #[test]
    fn text_anchor_bottom_right() {
        let t = Text::new(&TOM_THUMB_3X5, "Hi")
            .position(50.0, 50.0)
            .bottom_right();

        let mut cells = Vec::new();
        t.emit_cells(&mut |c| cells.push(c));
        assert!(!cells.is_empty());

        for cell in &cells {
            assert!(
                cell.position.x <= 50.0,
                "All cells should be left of anchor x, got {}",
                cell.position.x
            );
            assert!(
                cell.position.y <= 50.0,
                "All cells should be above anchor y, got {}",
                cell.position.y
            );
        }
    }

    #[test]
    fn text_anchor_bottom_left_with_flip_y() {
        let t = Text::new(&TOM_THUMB_3X5, "Hi")
            .position(0.0, 0.0)
            .bottom_left();

        let mut cells = Vec::new();
        t.flip_y().emit_cells(&mut |c| cells.push(c));
        assert!(!cells.is_empty());

        for cell in &cells {
            assert!(
                cell.position.y >= 0.0,
                "After flip_y, BottomLeft anchored text should be above position (y >= 0), got {}",
                cell.position.y
            );
        }
    }

    #[test]
    fn text_anchor_bottom_right_with_flip_y() {
        let t = Text::new(&TOM_THUMB_3X5, "Hi")
            .position(0.0, 0.0)
            .bottom_right();

        let mut cells = Vec::new();
        t.flip_y().emit_cells(&mut |c| cells.push(c));
        assert!(!cells.is_empty());

        for cell in &cells {
            assert!(
                cell.position.y >= 0.0,
                "After flip_y, BottomRight anchored text should be above position (y >= 0), got {}",
                cell.position.y
            );
        }
    }

    #[test]
    fn text_anchor_top_left_with_flip_y() {
        let t = Text::new(&TOM_THUMB_3X5, "Hi")
            .position(0.0, 0.0)
            .top_left();

        let mut cells = Vec::new();
        t.flip_y().emit_cells(&mut |c| cells.push(c));
        assert!(!cells.is_empty());

        for cell in &cells {
            assert!(
                cell.position.y <= 0.0,
                "After flip_y, TopLeft anchored text should be below position (y <= 0), got {}",
                cell.position.y
            );
        }
    }

    #[test]
    fn text_anchor_center() {
        let t = Text::new(&TOM_THUMB_3X5, "Hi")
            .position(50.0, 50.0)
            .center();

        let mut cells = Vec::new();
        t.emit_cells(&mut |c| cells.push(c));
        assert!(!cells.is_empty());

        let min_x = cells.iter().map(|c| c.position.x).fold(f32::MAX, f32::min);
        let max_x = cells.iter().map(|c| c.position.x).fold(f32::MIN, f32::max);
        let min_y = cells.iter().map(|c| c.position.y).fold(f32::MAX, f32::min);
        let max_y = cells.iter().map(|c| c.position.y).fold(f32::MIN, f32::max);

        assert!(min_x < 50.0 && max_x >= 50.0, "Center should straddle x");
        assert!(min_y < 50.0 && max_y >= 50.0, "Center should straddle y");
    }

    #[test]
    fn text_anchor_top_center() {
        let t = Text::new(&TOM_THUMB_3X5, "Hi")
            .position(50.0, 10.0)
            .top_center();

        let mut cells = Vec::new();
        t.emit_cells(&mut |c| cells.push(c));
        assert!(!cells.is_empty());

        let min_x = cells.iter().map(|c| c.position.x).fold(f32::MAX, f32::min);
        let max_x = cells.iter().map(|c| c.position.x).fold(f32::MIN, f32::max);

        assert!(min_x < 50.0 && max_x >= 50.0, "TopCenter should straddle x");
        for cell in &cells {
            assert!(cell.position.y >= 10.0, "TopCenter cells should be at or below anchor y");
        }
    }

    #[test]
    fn text_anchor_bottom_center() {
        let t = Text::new(&TOM_THUMB_3X5, "Hi")
            .position(50.0, 50.0)
            .bottom_center();

        let mut cells = Vec::new();
        t.emit_cells(&mut |c| cells.push(c));
        assert!(!cells.is_empty());

        let min_x = cells.iter().map(|c| c.position.x).fold(f32::MAX, f32::min);
        let max_x = cells.iter().map(|c| c.position.x).fold(f32::MIN, f32::max);

        assert!(min_x < 50.0 && max_x >= 50.0, "BottomCenter should straddle x");
        for cell in &cells {
            assert!(cell.position.y <= 50.0, "BottomCenter cells should be at or above anchor y");
        }
    }

    #[test]
    fn text_anchor_center_left() {
        let t = Text::new(&TOM_THUMB_3X5, "Hi")
            .position(10.0, 50.0)
            .center_left();

        let mut cells = Vec::new();
        t.emit_cells(&mut |c| cells.push(c));
        assert!(!cells.is_empty());

        let min_y = cells.iter().map(|c| c.position.y).fold(f32::MAX, f32::min);
        let max_y = cells.iter().map(|c| c.position.y).fold(f32::MIN, f32::max);

        assert!(min_y < 50.0 && max_y >= 50.0, "CenterLeft should straddle y");
        for cell in &cells {
            assert!(cell.position.x >= 10.0, "CenterLeft cells should be at or right of anchor x");
        }
    }

    #[test]
    fn text_anchor_center_right() {
        let t = Text::new(&TOM_THUMB_3X5, "Hi")
            .position(50.0, 50.0)
            .center_right();

        let mut cells = Vec::new();
        t.emit_cells(&mut |c| cells.push(c));
        assert!(!cells.is_empty());

        let min_y = cells.iter().map(|c| c.position.y).fold(f32::MAX, f32::min);
        let max_y = cells.iter().map(|c| c.position.y).fold(f32::MIN, f32::max);

        assert!(min_y < 50.0 && max_y >= 50.0, "CenterRight should straddle y");
        for cell in &cells {
            assert!(cell.position.x <= 50.0, "CenterRight cells should be at or left of anchor x");
        }
    }

    #[test]
    fn text_origin_is_position_invariant_of_anchor() {
        let pos = (25.0, 30.0);

        let builders: Vec<Text> = vec![
            Text::new(&TOM_THUMB_3X5, "Hi").position(pos.0, pos.1).top_left(),
            Text::new(&TOM_THUMB_3X5, "Hi").position(pos.0, pos.1).top_right(),
            Text::new(&TOM_THUMB_3X5, "Hi").position(pos.0, pos.1).bottom_left(),
            Text::new(&TOM_THUMB_3X5, "Hi").position(pos.0, pos.1).bottom_right(),
            Text::new(&TOM_THUMB_3X5, "Hi").position(pos.0, pos.1).top_center(),
            Text::new(&TOM_THUMB_3X5, "Hi").position(pos.0, pos.1).bottom_center(),
            Text::new(&TOM_THUMB_3X5, "Hi").position(pos.0, pos.1).center_left(),
            Text::new(&TOM_THUMB_3X5, "Hi").position(pos.0, pos.1).center_right(),
            Text::new(&TOM_THUMB_3X5, "Hi").position(pos.0, pos.1).center(),
        ];

        for t in &builders {
            assert_eq!(
                t.origin(),
                Some(pos),
                "origin() should equal position regardless of anchor"
            );
        }
    }
}
