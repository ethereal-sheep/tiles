use core::cell::OnceCell;
use core::fmt;
use std::path::Path;
use std::rc::Rc;

use crate::anchor::{AnchorCorner, corner_offset};
use crate::cell::Cell;
use crate::color::{Color, srgb_to_linear};
use crate::drawable::Drawable;
use crate::rect::Rect;

#[derive(Debug)]
pub struct ImageError(image::ImageError);

impl fmt::Display for ImageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl std::error::Error for ImageError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.0)
    }
}

impl From<image::ImageError> for ImageError {
    fn from(err: image::ImageError) -> Self {
        Self(err)
    }
}

struct PixelBuffer {
    pixels: Box<[u8]>,
    width: u32,
    height: u32,
}

struct ImageData {
    base: PixelBuffer,
    // Consumed by Frame::rotate() (RotSprite upscale step), not yet implemented.
    #[allow(dead_code)]
    x8: OnceCell<PixelBuffer>,
}

impl ImageData {
    #[allow(dead_code)]
    fn x8(&self) -> &PixelBuffer {
        self.x8
            .get_or_init(|| self.base.scale2x().scale2x().scale2x())
    }

    #[allow(dead_code)]
    fn rotsprite(&self, degrees: f32) -> PixelBuffer {
        self.x8().nn_rotate(degrees).down8x()
    }
}

impl PixelBuffer {
    fn scale2x(&self) -> PixelBuffer {
        let (p, w, h) = scale2x(&self.pixels, self.width, self.height);
        Self {
            pixels: p.into(),
            width: w,
            height: h,
        }
    }

    #[allow(dead_code)]
    fn down2x(&self) -> PixelBuffer {
        let (p, w, h) = down_nx(&self.pixels, self.width, self.height, 2);
        Self {
            pixels: p.into(),
            width: w,
            height: h,
        }
    }

    fn down8x(&self) -> PixelBuffer {
        let (p, w, h) = down_nx(&self.pixels, self.width, self.height, 8);
        Self {
            pixels: p.into(),
            width: w,
            height: h,
        }
    }

    fn nn_rotate(&self, degrees: f32) -> PixelBuffer {
        let (p, w, h) = nn_rotate(&self.pixels, self.width, self.height, degrees);
        Self {
            pixels: p.into(),
            width: w,
            height: h,
        }
    }

    #[allow(dead_code)]
    fn rotsprite(&self, degrees: f32) -> PixelBuffer {
        let (p, w, h) = rotsprite(&self.pixels, self.width, self.height, degrees);
        Self {
            pixels: p.into(),
            width: w,
            height: h,
        }
    }
}

/// RotSprite rotation: NN-rotates an 8x-upscaled buffer into an expanded
/// bounding box (no cropping), then mode-filter-downscales by 8. `degrees`
/// is clockwise as seen on screen. `width_x8`/`height_x8` must be multiples
/// of 8 (the scale factor produced by three `scale2x` passes).
fn rotsprite(pixels: &[u8], width: u32, height: u32, degrees: f32) -> (Vec<u8>, u32, u32) {
    let (pixels, width, height) = scale2x(pixels, width, height);
    let (pixels, width, height) = scale2x(&pixels, width, height);
    let (pixels, width, height) = scale2x(&pixels, width, height);
    let (pixels, width, height) = nn_rotate(&pixels, width, height, degrees);
    down_nx(&pixels, width, height, 8)
}

fn derive_bbox(width: u32, height: u32, degrees: f32) -> (u32, u32) {
    let radians = degrees.to_radians();
    let cos = radians.cos();
    let sin = radians.sin();

    // Subtract a small epsilon before ceiling so floating-point trig error
    // (e.g. cos(pi) landing a hair above -1.0) doesn't spuriously expand the
    // bounding box for exact multiples of 90 degrees.
    const EPS: f32 = 1e-3;
    let bbox_w = ((width as f32 * cos.abs() + height as f32 * sin.abs() - EPS)
        .ceil()
        .max(1.0)) as u32;
    let bbox_h = ((width as f32 * sin.abs() + height as f32 * cos.abs() - EPS)
        .ceil()
        .max(1.0)) as u32;
    (bbox_w, bbox_h)
}

#[allow(dead_code)]
fn scale2x(pixels: &[u8], width: u32, height: u32) -> (Vec<u8>, u32, u32) {
    let w = width as i32;
    let h = height as i32;
    let out_w = width * 2;
    let out_h = height * 2;
    let mut out = vec![0u8; (out_w * out_h * 4) as usize];

    let pixel_at = |x: i32, y: i32| -> [u8; 4] {
        let x = x.clamp(0, w - 1);
        let y = y.clamp(0, h - 1);
        let idx = ((y * w + x) * 4) as usize;
        [
            pixels[idx],
            pixels[idx + 1],
            pixels[idx + 2],
            pixels[idx + 3],
        ]
    };

    let mut put = |x: i32, y: i32, px: [u8; 4]| {
        let idx = ((y * out_w as i32 + x) * 4) as usize;
        out[idx..idx + 4].copy_from_slice(&px);
    };

    for y in 0..h {
        for x in 0..w {
            let b = pixel_at(x, y - 1);
            let d = pixel_at(x - 1, y);
            let e = pixel_at(x, y);
            let f = pixel_at(x + 1, y);
            let h_px = pixel_at(x, y + 1);

            let (mut e0, mut e1, mut e2, mut e3) = (e, e, e, e);

            if d == b && d != h_px && b != f {
                e0 = d;
            }
            if b == f && b != d && f != h_px {
                e1 = f;
            }
            if h_px == d && h_px != f && d != b {
                e2 = d;
            }
            if f == h_px && f != b && h_px != d {
                e3 = f;
            }

            put(x * 2, y * 2, e0);
            put(x * 2 + 1, y * 2, e1);
            put(x * 2, y * 2 + 1, e2);
            put(x * 2 + 1, y * 2 + 1, e3);
        }
    }

    (out, out_w, out_h)
}

/// Pull-sampling NN rotate: for each destination pixel, inverse-rotate back
/// into source space. Destination pixels landing outside the source stay
/// transparent (the bounding-box padding added by the expanded canvas).
fn nn_rotate(pixels: &[u8], width: u32, height: u32, degrees: f32) -> (Vec<u8>, u32, u32) {
    let (dst_w, dst_h) = derive_bbox(width, height, degrees);
    let mut out = vec![0u8; (dst_w * dst_h * 4) as usize];

    let radians = degrees.to_radians();
    let cos = radians.cos();
    let sin = radians.sin();

    let cx_src = width as f32 / 2.0;
    let cy_src = height as f32 / 2.0;
    let cx_dst = dst_w as f32 / 2.0;
    let cy_dst = dst_h as f32 / 2.0;

    for oy in 0..dst_h {
        for ox in 0..dst_w {
            let dx = (ox as f32 + 0.5) - cx_dst;
            let dy = (oy as f32 + 0.5) - cy_dst;

            // Inverse of the forward clockwise-on-screen rotation matrix.
            let sx = dx * cos + dy * sin;
            let sy = -dx * sin + dy * cos;

            let src_x = (sx + cx_src).floor() as i32;
            let src_y = (sy + cy_src).floor() as i32;

            if src_x < 0 || src_y < 0 || src_x >= width as i32 || src_y >= height as i32 {
                continue;
            }

            let src_idx = ((src_y as u32 * width + src_x as u32) * 4) as usize;
            let dst_idx = ((oy * dst_w + ox) * 4) as usize;
            out[dst_idx..dst_idx + 4].copy_from_slice(&pixels[src_idx..src_idx + 4]);
        }
    }

    (out, dst_w, dst_h)
}

/// Downscales by exactly nx by voting on the most common color within each
/// nxn block, ignoring fully-transparent pixels in the vote. A block is
/// only transparent in the output if every one of its nxn pixels is.
fn down_nx(pixels: &[u8], width: u32, height: u32, n: u32) -> (Vec<u8>, u32, u32) {
    if n == 0 || n == 1 || n > width.min(height) {
        return (pixels.into(), width, height);
    }

    let out_w = width / n;
    let out_h = height / n;
    let mut out = vec![0u8; (out_w * out_h * 4) as usize];

    for by in 0..out_h {
        for bx in 0..out_w {
            let mut counts: Vec<([u8; 4], u32)> = Vec::new();

            for dy in 0..n {
                for dx in 0..n {
                    let x = bx * n + dx;
                    let y = by * n + dy;
                    let idx = ((y * width + x) * 4) as usize;
                    let px = [
                        pixels[idx],
                        pixels[idx + 1],
                        pixels[idx + 2],
                        pixels[idx + 3],
                    ];

                    if px[3] == 0 {
                        continue;
                    }

                    match counts.iter_mut().find(|(c, _)| *c == px) {
                        Some(entry) => entry.1 += 1,
                        None => counts.push((px, 1)),
                    }
                }
            }

            let mode = counts
                .iter()
                .max_by_key(|(_, count)| *count)
                .map(|(c, _)| *c)
                .unwrap_or([0, 0, 0, 0]);

            let out_idx = ((by * out_w + bx) * 4) as usize;
            out[out_idx..out_idx + 4].copy_from_slice(&mode);
        }
    }

    (out, out_w, out_h)
}

pub struct Image {
    data: Rc<ImageData>,
}

impl fmt::Debug for Image {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Image")
            .field("width", &self.width())
            .field("height", &self.height())
            .finish()
    }
}

impl Image {
    pub fn from_path(path: impl AsRef<Path>) -> Result<Self, ImageError> {
        let decoded = image::open(path)?.to_rgba8();
        let (width, height) = decoded.dimensions();
        Ok(Self {
            data: Rc::new(ImageData {
                base: PixelBuffer {
                    pixels: decoded.into_raw().into(),
                    width,
                    height,
                },
                x8: OnceCell::new(),
            }),
        })
    }

    pub fn width(&self) -> u32 {
        self.data.base.width
    }

    pub fn height(&self) -> u32 {
        self.data.base.height
    }

    pub fn instance(&self) -> Frame {
        self.frame(0)
    }

    pub fn frame(&self, _index: usize) -> Frame {
        Frame {
            data: self.data.clone(),
            rect: Rect::from_top_left(0.0, 0.0, self.width(), self.height()),
            offset: (0, 0),
            anchor_corner: AnchorCorner::default(),
        }
    }
}

#[derive(Clone)]
pub struct Frame {
    data: Rc<ImageData>,
    rect: Rect,
    offset: (u32, u32),
    anchor_corner: AnchorCorner,
}

impl fmt::Debug for Frame {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Frame")
            .field("rect", &self.rect)
            .field("offset", &self.offset)
            .finish()
    }
}

impl Frame {
    pub fn width(&self) -> u32 {
        self.rect.width()
    }

    pub fn height(&self) -> u32 {
        self.rect.height()
    }

    pub fn position(mut self, x: f32, y: f32) -> Self {
        self.rect = Rect::from_top_left(x, y, self.rect.width(), self.rect.height());
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

    fn anchor_offset(&self) -> (f32, f32) {
        corner_offset(
            self.anchor_corner,
            self.rect.width() as f32,
            self.rect.height() as f32,
            0.0,
            0.0,
        )
    }
}

impl Drawable for Frame {
    fn origin(&self) -> Option<(f32, f32)> {
        Some((self.rect.x(), self.rect.y()))
    }

    fn emit_local_cells(&self, f: &mut dyn FnMut(Cell)) {
        let (ax, ay) = self.anchor_offset();
        let base = &self.data.base;
        let (offset_x, offset_y) = self.offset;

        for row in 0..self.rect.height() {
            for col in 0..self.rect.width() {
                let px = offset_x + col;
                let py = offset_y + row;
                let idx = ((py * base.width + px) * 4) as usize;
                let [r, g, b, a] = base.pixels[idx..idx + 4] else {
                    unreachable!("pixel buffer sized as width * height * 4")
                };

                if a == 0 {
                    continue;
                }

                let color = Color::linear(
                    srgb_to_linear(r as f32 / 255.0),
                    srgb_to_linear(g as f32 / 255.0),
                    srgb_to_linear(b as f32 / 255.0),
                    a as f32 / 255.0,
                );

                let cell = Cell::new(ax + col as f32, ay + row as f32).color(color);
                f(cell);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{Rgba, RgbaImage};

    fn write_temp_png(name: &str, img: &RgbaImage) -> std::path::PathBuf {
        let mut path = std::env::temp_dir();
        path.push(format!("tiles_image_test_{name}.png"));
        img.save(&path).unwrap();
        path
    }

    #[test]
    fn from_path_reports_dimensions() {
        let img = RgbaImage::from_pixel(3, 2, Rgba([255, 0, 0, 255]));
        let path = write_temp_png("dimensions", &img);

        let image = Image::from_path(&path).unwrap();
        assert_eq!(image.width(), 3);
        assert_eq!(image.height(), 2);
    }

    #[test]
    fn from_path_missing_file_errors() {
        let result = Image::from_path("/nonexistent/path/does-not-exist.png");
        assert!(result.is_err());
    }

    #[test]
    fn instance_matches_frame_zero() {
        let img = RgbaImage::from_pixel(2, 2, Rgba([255, 255, 255, 255]));
        let path = write_temp_png("instance_matches_frame", &img);

        let image = Image::from_path(&path).unwrap();
        assert_eq!(
            image.instance().to_cells().len(),
            image.frame(0).to_cells().len()
        );
    }

    #[test]
    fn frame_ignores_index() {
        let img = RgbaImage::from_pixel(2, 2, Rgba([255, 255, 255, 255]));
        let path = write_temp_png("frame_ignores_index", &img);

        let image = Image::from_path(&path).unwrap();
        assert_eq!(
            image.frame(0).to_cells().len(),
            image.frame(7).to_cells().len()
        );
    }

    #[test]
    fn frame_reports_dimensions() {
        let img = RgbaImage::from_pixel(3, 2, Rgba([255, 0, 0, 255]));
        let path = write_temp_png("frame_dimensions", &img);

        let image = Image::from_path(&path).unwrap();
        let frame = image.instance();
        assert_eq!(frame.width(), 3);
        assert_eq!(frame.height(), 2);
    }

    #[test]
    fn alpha_zero_pixels_are_skipped() {
        let mut img = RgbaImage::from_pixel(2, 1, Rgba([255, 255, 255, 255]));
        img.put_pixel(1, 0, Rgba([0, 0, 0, 0]));
        let path = write_temp_png("alpha_skip", &img);

        let image = Image::from_path(&path).unwrap();
        let cells = image.instance().to_cells();
        assert_eq!(cells.len(), 1);
        assert_eq!(cells[0].position.x, 0.0);
    }

    #[test]
    fn srgb_converted_to_linear() {
        let img = RgbaImage::from_pixel(1, 1, Rgba([128, 128, 128, 255]));
        let path = write_temp_png("srgb", &img);

        let image = Image::from_path(&path).unwrap();
        let cells = image.instance().to_cells();
        let expected = srgb_to_linear(128.0 / 255.0);
        assert!((cells[0].color[0] - expected).abs() < 1e-5);
        assert_eq!(cells[0].color[3], 1.0);
    }

    #[test]
    fn position_offsets_cells() {
        let img = RgbaImage::from_pixel(1, 1, Rgba([255, 255, 255, 255]));
        let path = write_temp_png("position", &img);

        let image = Image::from_path(&path).unwrap();
        let cells = image.instance().position(10.0, 20.0).to_cells();
        assert_eq!(cells[0].position.x, 10.0);
        assert_eq!(cells[0].position.y, 20.0);
    }

    #[test]
    fn center_anchor_straddles_position() {
        let img = RgbaImage::from_pixel(4, 4, Rgba([255, 255, 255, 255]));
        let path = write_temp_png("center", &img);

        let image = Image::from_path(&path).unwrap();
        let cells = image.instance().position(0.0, 0.0).center().to_cells();

        let min_x = cells.iter().map(|c| c.position.x).fold(f32::MAX, f32::min);
        let max_x = cells.iter().map(|c| c.position.x).fold(f32::MIN, f32::max);
        assert!(min_x < 0.0 && max_x >= 0.0);
    }

    #[test]
    fn frame_clone_shares_pixel_buffer() {
        let img = RgbaImage::from_pixel(1, 1, Rgba([255, 255, 255, 255]));
        let path = write_temp_png("clone", &img);

        let image = Image::from_path(&path).unwrap();
        let frame = image.instance();
        let cloned = frame.clone();
        assert!(Rc::ptr_eq(&frame.data, &cloned.data));
    }

    // --- ImageData x8 (lazy upscale) ---

    fn test_image_data(colors: &[[u8; 4]], width: u32, height: u32) -> ImageData {
        ImageData {
            base: PixelBuffer {
                pixels: rgba_buffer(colors).into(),
                width,
                height,
            },
            x8: OnceCell::new(),
        }
    }

    #[test]
    fn x8_scales_dimensions_by_eight() {
        let data = test_image_data(&[[255, 0, 0, 255]; 4], 2, 2);
        let x8 = data.x8();
        assert_eq!(x8.width, 16);
        assert_eq!(x8.height, 16);
    }

    #[test]
    fn x8_matches_three_manual_scale2x_passes() {
        let data = test_image_data(
            &[
                [10, 20, 30, 255],
                [40, 50, 60, 255],
                [70, 80, 90, 255],
                [100, 110, 120, 255],
            ],
            2,
            2,
        );

        let (p1, w1, h1) = scale2x(&data.base.pixels, data.base.width, data.base.height);
        let (p2, w2, h2) = scale2x(&p1, w1, h1);
        let (p3, w3, h3) = scale2x(&p2, w2, h2);

        let x8 = data.x8();
        assert_eq!(x8.width, w3);
        assert_eq!(x8.height, h3);
        assert_eq!(&*x8.pixels, p3.as_slice());
    }

    #[test]
    fn x8_is_cached_across_calls() {
        let data = test_image_data(&[[255, 0, 0, 255]; 4], 2, 2);
        let first = data.x8() as *const PixelBuffer;
        let second = data.x8() as *const PixelBuffer;
        assert_eq!(first, second);
    }

    // --- scale2x ---

    fn rgba_buffer(colors: &[[u8; 4]]) -> Vec<u8> {
        colors.iter().flat_map(|c| c.iter().copied()).collect()
    }

    #[test]
    fn scale2x_doubles_dimensions() {
        let pixels = rgba_buffer(&[[255, 0, 0, 255]; 4]);
        let (_out, w, h) = scale2x(&pixels, 2, 2);
        assert_eq!(w, 4);
        assert_eq!(h, 4);
    }

    #[test]
    fn scale2x_solid_color_stays_solid() {
        let color = [10, 20, 30, 255];
        let pixels = rgba_buffer(&[color; 9]);
        let (out, w, h) = scale2x(&pixels, 3, 3);

        for i in 0..(w * h) as usize {
            let idx = i * 4;
            assert_eq!(&out[idx..idx + 4], &color);
        }
    }

    #[test]
    fn scale2x_diagonal_edge_fills_corner() {
        // 2x2 source:
        // A A
        // A B
        // Diagonal edge between A (top-left/top-right/bottom-left) and B (bottom-right).
        let a = [255, 255, 255, 255];
        let b = [0, 0, 0, 255];
        let pixels = rgba_buffer(&[a, a, a, b]);
        let (out, w, _h) = scale2x(&pixels, 2, 2);

        // B is at source (1,1) -> output quadrant (2,2)-(3,3).
        // e0 for B: d(=A at (0,1)) == b(=A at (1,0))? d != h(=B itself), b != f(=B itself) -> true -> e0 = A.
        let get = |x: u32, y: u32| {
            let idx = ((y * w + x) * 4) as usize;
            [out[idx], out[idx + 1], out[idx + 2], out[idx + 3]]
        };
        assert_eq!(
            get(2, 2),
            a,
            "corner nearest the diagonal should fill with A"
        );
        assert_eq!(get(3, 3), b, "corner farthest from the diagonal stays B");
    }

    // --- rotate ---

    /// Builds a buffer with 4 color quadrants: TL=colors[0], TR=colors[1],
    /// BL=colors[2], BR=colors[3]. `cols` x `rows` total pixel dimensions.
    fn quadrant_buffer(colors: &[[u8; 4]; 4], cols: u32, rows: u32) -> (Vec<u8>, u32, u32) {
        let mut out = vec![0u8; (cols * rows * 4) as usize];
        let h_w = cols / 2;
        let h_h = rows / 2;

        for row in 0..rows {
            for col in 0..cols {
                let qi = (row / h_h) * 2 + col / h_w;
                let color = colors[qi as usize];
                let idx = ((row * cols + col) * 4) as usize;
                out[idx..idx + 4].copy_from_slice(&color);
            }
        }

        (out, cols, rows)
    }

    fn pixel_at(pixels: &[u8], width: u32, x: u32, y: u32) -> [u8; 4] {
        let idx = ((y * width + x) * 4) as usize;
        [
            pixels[idx],
            pixels[idx + 1],
            pixels[idx + 2],
            pixels[idx + 3],
        ]
    }

    const RED: [u8; 4] = [255, 0, 0, 255];
    const GREEN: [u8; 4] = [0, 255, 0, 255];
    const BLUE: [u8; 4] = [0, 0, 255, 255];
    const YELLOW: [u8; 4] = [255, 255, 0, 255];

    #[test]
    fn rotate_90_moves_quadrants_clockwise() {
        // TL=Red TR=Green / BL=Blue BR=Yellow, 16x16 pixels
        let (pixels, w, h) = quadrant_buffer(&[RED, GREEN, BLUE, YELLOW], 16, 16);
        let (out, out_w, out_h) = nn_rotate(&pixels, w, h, 90.0);

        assert_eq!(out_w, 16);
        assert_eq!(out_h, 16);
        // After 90° CW: TL←BL, TR←TL, BR←TR, BL←BR
        assert_eq!(pixel_at(&out, out_w, 0, 0), BLUE, "new TL was old BL");
        assert_eq!(pixel_at(&out, out_w, 15, 0), RED, "new TR was old TL");
        assert_eq!(pixel_at(&out, out_w, 15, 15), GREEN, "new BR was old TR");
        assert_eq!(pixel_at(&out, out_w, 0, 15), YELLOW, "new BL was old BR");
    }

    #[test]
    fn rotate_180_swaps_opposite_corners() {
        let (pixels, w, h) = quadrant_buffer(&[RED, GREEN, BLUE, YELLOW], 16, 16);
        let (out, out_w, out_h) = nn_rotate(&pixels, w, h, 180.0);

        assert_eq!(out_w, 16);
        assert_eq!(out_h, 16);
        assert_eq!(pixel_at(&out, out_w, 0, 0), YELLOW);
        assert_eq!(pixel_at(&out, out_w, 15, 15), RED);
    }

    #[test]
    fn rotate_270_moves_quadrants_counter_to_90() {
        let (pixels, w, h) = quadrant_buffer(&[RED, GREEN, BLUE, YELLOW], 16, 16);
        let (out, out_w, out_h) = nn_rotate(&pixels, w, h, 270.0);

        assert_eq!(out_w, 16);
        assert_eq!(out_h, 16);
        assert_eq!(pixel_at(&out, out_w, 0, 0), GREEN);
        assert_eq!(pixel_at(&out, out_w, 15, 15), BLUE);
    }

    #[test]
    fn rotate_90_multiples_do_not_expand_bounding_box() {
        let (pixels, w, h) = quadrant_buffer(&[RED, GREEN, BLUE, YELLOW], 16, 16);
        for angle in [0.0, 90.0, 180.0, 270.0] {
            let (_out, out_w, out_h) = nn_rotate(&pixels, w, h, angle);
            assert_eq!(
                (out_w, out_h),
                (16, 16),
                "angle {angle} should not expand bbox"
            );
        }
    }

    #[test]
    fn rotate_45_expands_bounding_box() {
        let (pixels, w, h) = quadrant_buffer(&[RED, GREEN, BLUE, YELLOW], 16, 16);
        let (_out, out_w, out_h) = nn_rotate(&pixels, w, h, 45.0);

        assert_eq!(out_w, 23);
        assert_eq!(out_h, 23);
    }

    #[test]
    fn mode_downscale_picks_majority_color() {
        let mut block = vec![0u8; 8 * 8 * 4];
        for i in 0..48 {
            block[i * 4..i * 4 + 4].copy_from_slice(&RED);
        }
        for i in 48..64 {
            block[i * 4..i * 4 + 4].copy_from_slice(&GREEN);
        }

        let (out, out_w, out_h) = down_nx(&block, 8, 8, 8);
        assert_eq!((out_w, out_h), (1, 1));
        assert_eq!(&out[0..4], &RED);
    }

    #[test]
    fn mode_downscale_single_opaque_pixel_wins_over_transparent() {
        let mut block = vec![0u8; 8 * 8 * 4];
        block[0..4].copy_from_slice(&RED);

        let (out, _w, _h) = down_nx(&block, 8, 8, 8);
        assert_eq!(
            &out[0..4],
            &RED,
            "the only opaque pixel should win the vote"
        );
    }

    #[test]
    fn mode_downscale_all_transparent_block_stays_transparent() {
        let block = vec![0u8; 8 * 8 * 4];
        let (out, _w, _h) = down_nx(&block, 8, 8, 8);
        assert_eq!(
            out[3], 0,
            "block with zero opaque pixels must stay transparent"
        );
    }

    #[test]
    fn nn_rotate_pads_expanded_corners_as_transparent() {
        // 8x8 solid red block rotated 45° → bbox expands to 12x12.
        // Corners of expanded canvas should be transparent.
        let pixels = vec![RED; 8 * 8].into_iter().flatten().collect::<Vec<_>>();
        let (rotated, out_w, _out_h) = nn_rotate(&pixels, 8, 8, 45.0);
        let corner = pixel_at(&rotated, out_w, 0, 0);
        assert_eq!(
            corner[3], 0,
            "expanded corner should be transparent, not sampled"
        );
    }
}
