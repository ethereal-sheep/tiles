use core::cell::OnceCell;
use core::fmt;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::rc::Rc;

use image::AnimationDecoder;
use image::codecs::gif::GifDecoder;

use crate::anchor::AnchorCorner;
use crate::cell::Cell;
use crate::drawable::Drawable;
use crate::rect::Rect;

/// A default frame duration (seconds) used where no real timing is known:
/// grid-sliced frames, and any `Sprite::frame_at` lookup against a `FrameData`
/// with `duration: None`.
const DEFAULT_FRAME_DURATION: f32 = 0.1;

/// One sub-region of a multi-frame `Image`. `duration` is `Some` only when
/// decoded from a source with real per-frame timing (e.g. a GIF); grid
/// slicing (`Sprite::grid`) always sets it explicitly instead.
#[derive(Clone, Copy, Debug)]
pub struct FrameData {
    pub offset: (u32, u32),
    pub size: (u32, u32),
    pub duration: Option<f32>,
}

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

#[derive(Clone)]
pub struct PixelBufferView<'a> {
    pixels: &'a [u8],
    stride: u32, // width of the *parent* buffer, in pixels
    x: u32,
    y: u32,
    width: u32,
    height: u32,
}

struct ImageData {
    base: PixelBuffer,
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
    #[inline]
    pub fn len(&self) -> usize {
        self.width as usize * self.height as usize * 4
    }

    #[inline]
    pub fn width(&self) -> u32 {
        self.width
    }

    #[inline]
    pub fn height(&self) -> u32 {
        self.height
    }

    /// Single pixel as an RGBA array.
    #[inline]
    pub fn pixel(&self, x: u32, y: u32) -> [u8; 4] {
        self.view().pixel(x, y)
    }

    #[inline]
    fn row(&self, y: u32) -> &[u8] {
        self.view().row(y)
    }

    /// Borrow as a read-only view.
    pub fn view(&self) -> PixelBufferView<'_> {
        self.subrect(0, 0, self.width, self.height)
    }

    /// Borrow a sub-rectangle of this buffer as a read-only view.
    pub fn subrect(&self, x: u32, y: u32, width: u32, height: u32) -> PixelBufferView<'_> {
        assert!(
            x + width <= self.width && y + height <= self.height,
            "view out of bounds"
        );
        PixelBufferView {
            pixels: &self.pixels,
            stride: self.width,
            x,
            y,
            width,
            height,
        }
    }

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

    fn rotsprite(&self, degrees: f32) -> PixelBuffer {
        let (p, w, h) = rotsprite(&self.pixels, self.width, self.height, degrees);
        Self {
            pixels: p.into(),
            width: w,
            height: h,
        }
    }
}

impl<'a> From<PixelBufferView<'a>> for PixelBuffer {
    fn from(value: PixelBufferView<'a>) -> Self {
        value.to_pixel_buffer()
    }
}

impl<'a> PixelBufferView<'a> {
    #[inline]
    pub fn len(&self) -> usize {
        self.width as usize * self.height as usize * 4
    }

    #[inline]
    pub fn width(&self) -> u32 {
        self.width
    }

    #[inline]
    pub fn height(&self) -> u32 {
        self.height
    }

    /// Byte offset (into the *parent* buffer) of local pixel (x, y).
    #[inline]
    fn pixel_offset(&self, x: u32, y: u32) -> usize {
        assert!(
            x < self.width && y < self.height,
            "pixel ({x}, {y}) out of bounds"
        );
        (((self.y + y) * self.stride + (self.x + x)) * 4) as usize
    }

    #[inline]
    fn pixel(&self, x: u32, y: u32) -> [u8; 4] {
        let off = self.pixel_offset(x, y);
        (&self.pixels[off..off + 4]).try_into().unwrap()
    }

    #[inline]
    fn row(&self, y: u32) -> &'a [u8] {
        assert!(y < self.height, "row {y} out of bounds");
        let start = (((self.y + y) * self.stride + self.x) * 4) as usize;
        let end = start + (self.width * 4) as usize;
        &self.pixels[start..end]
    }

    /// Take a sub-rect of this view (relative coordinates).
    #[allow(dead_code)]
    fn subrect(&self, x: u32, y: u32, width: u32, height: u32) -> PixelBufferView<'_> {
        assert!(
            x + width <= self.width && y + height <= self.height,
            "sub-view out of bounds"
        );
        PixelBufferView {
            pixels: self.pixels,
            stride: self.stride,
            x: self.x + x,
            y: self.y + y,
            width,
            height,
        }
    }

    /// Copy this view's pixels into a new, owned, tightly-packed PixelBuffer.
    fn to_pixel_buffer(&self) -> PixelBuffer {
        let mut out = vec![0u8; self.len()];
        for y in 0..self.height {
            let src = self.row(y);
            let dst_start = (y * self.width * 4) as usize;
            let dst_end = dst_start + (self.width * 4) as usize;
            out[dst_start..dst_end].copy_from_slice(src);
        }
        PixelBuffer {
            pixels: out.into(),
            width: self.width,
            height: self.height,
        }
    }

    #[allow(dead_code)]
    fn scale2x(&self) -> PixelBuffer {
        self.to_pixel_buffer().scale2x()
    }

    #[allow(dead_code)]
    fn down2x(&self) -> PixelBuffer {
        self.to_pixel_buffer().down2x()
    }

    #[allow(dead_code)]
    fn down8x(&self) -> PixelBuffer {
        self.to_pixel_buffer().down8x()
    }

    #[allow(dead_code)]
    fn nn_rotate(&self, degrees: f32) -> PixelBuffer {
        self.to_pixel_buffer().nn_rotate(degrees)
    }

    #[allow(dead_code)]
    fn rotsprite(&self, degrees: f32) -> PixelBuffer {
        self.to_pixel_buffer().rotsprite(degrees)
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
    let (pixels, width, height) = pad_to_multiple(&pixels, width, height, 8);
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

/// Centers `pixels` in a transparent canvas whose dimensions are the next
/// multiple of `n` at or above `width`/`height`. Padding is split as evenly
/// as possible on both axes so the downstream block-sampling grid in
/// `down_nx` stays centered on the original content instead of biased
/// toward one edge.
fn pad_to_multiple(pixels: &[u8], width: u32, height: u32, n: u32) -> (Vec<u8>, u32, u32) {
    let out_w = width.div_ceil(n) * n;
    let out_h = height.div_ceil(n) * n;

    if out_w == width && out_h == height {
        return (pixels.into(), width, height);
    }

    let pad_x = (out_w - width) / 2;
    let pad_y = (out_h - height) / 2;

    let mut out = vec![0u8; (out_w * out_h * 4) as usize];
    for y in 0..height {
        let src_start = ((y * width) * 4) as usize;
        let src_end = src_start + (width * 4) as usize;
        let dst_start = (((y + pad_y) * out_w + pad_x) * 4) as usize;
        let dst_end = dst_start + (width * 4) as usize;
        out[dst_start..dst_end].copy_from_slice(&pixels[src_start..src_end]);
    }

    (out, out_w, out_h)
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
    frame_data: Vec<FrameData>,
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
        let path = path.as_ref();
        let is_gif = path
            .extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| ext.eq_ignore_ascii_case("gif"));

        let (base, frame_data) = if is_gif {
            Self::decode_gif(path)?
        } else {
            let decoded = image::open(path)?.to_rgba8();
            let (width, height) = decoded.dimensions();
            (
                PixelBuffer {
                    pixels: decoded.into_raw().into(),
                    width,
                    height,
                },
                Vec::new(),
            )
        };

        Ok(Self {
            data: Rc::new(ImageData {
                base,
                x8: OnceCell::new(),
            }),
            frame_data,
        })
    }

    /// Decodes a GIF's frames (already fully composited to canvas size by
    /// the `image` crate's disposal-method handling) into a horizontal strip
    /// buffer, with one `FrameData` per source frame.
    fn decode_gif(path: &Path) -> Result<(PixelBuffer, Vec<FrameData>), ImageError> {
        let file = File::open(path).map_err(image::ImageError::from)?;
        let decoder = GifDecoder::new(BufReader::new(file))?;

        let decoded: Vec<(image::RgbaImage, f32)> = decoder
            .into_frames()
            .map(|frame| {
                let frame = frame?;
                let (numer, denom) = frame.delay().numer_denom_ms();
                let duration_secs = numer as f32 / denom as f32 / 1000.0;
                Ok((frame.into_buffer(), duration_secs))
            })
            .collect::<image::ImageResult<_>>()?;

        let frame_w = decoded.first().map_or(0, |(buf, _)| buf.width());
        let frame_h = decoded.first().map_or(0, |(buf, _)| buf.height());
        let strip_width = frame_w * decoded.len() as u32;

        let mut pixels = vec![0u8; strip_width as usize * frame_h as usize * 4];
        let mut frame_data = Vec::with_capacity(decoded.len());

        for (i, (buffer, duration_secs)) in decoded.iter().enumerate() {
            let x_offset = frame_w * i as u32;
            for y in 0..frame_h {
                let src_start = (y * frame_w * 4) as usize;
                let src_end = src_start + (frame_w * 4) as usize;
                let dst_start = ((y * strip_width + x_offset) * 4) as usize;
                let dst_end = dst_start + (frame_w * 4) as usize;
                pixels[dst_start..dst_end].copy_from_slice(&buffer.as_raw()[src_start..src_end]);
            }
            frame_data.push(FrameData {
                offset: (x_offset, 0),
                size: (frame_w, frame_h),
                duration: Some(*duration_secs),
            });
        }

        Ok((
            PixelBuffer {
                pixels: pixels.into(),
                width: strip_width,
                height: frame_h,
            },
            frame_data,
        ))
    }

    pub fn width(&self) -> u32 {
        self.data.base.width
    }

    pub fn height(&self) -> u32 {
        self.data.base.height
    }

    pub fn instance(&self) -> Frame {
        Frame {
            data: self.data.clone(),
            rect: Rect::from_top_left(0.0, 0.0, self.width(), self.height()),
            offset: (0, 0),
            anchor_corner: AnchorCorner::default(),
            degrees: 0.0,
        }
    }
}

/// An 8x8 fallback image (transparent background, red border, red X) drawn
/// in place of any `img(key)` widget whose key has no loaded `Image`.
pub(crate) fn placeholder_image() -> Frame {
    const SIZE: u32 = 8;
    const RED: [u8; 4] = [255, 0, 0, 255];

    let mut pixels = vec![0u8; (SIZE * SIZE * 4) as usize];
    for y in 0..SIZE {
        for x in 0..SIZE {
            let is_border = x == 0 || x == SIZE - 1 || y == 0 || y == SIZE - 1;
            let is_diagonal = x == y || x + y == SIZE - 1;
            if is_border || is_diagonal {
                let idx = ((y * SIZE + x) * 4) as usize;
                pixels[idx..idx + 4].copy_from_slice(&RED);
            }
        }
    }

    Image {
        data: Rc::new(ImageData {
            base: PixelBuffer {
                pixels: pixels.into(),
                width: SIZE,
                height: SIZE,
            },
            x8: OnceCell::new(),
        }),
        frame_data: Vec::new(),
    }
    .instance()
}

#[derive(Clone)]
pub struct Frame {
    data: Rc<ImageData>,
    rect: Rect,
    offset: (u32, u32),
    anchor_corner: AnchorCorner,
    degrees: f32,
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

    pub fn rotate(mut self, degrees: f32) -> Self {
        self.degrees = (self.degrees + degrees).rem_euclid(360.0);
        self
    }
}

impl Drawable for Frame {
    fn origin(&self) -> Option<(f32, f32)> {
        Some((self.rect.x(), self.rect.y()))
    }

    fn emit_local_cells(&self, f: &mut dyn FnMut(Cell)) {
        let (offset_x, offset_y) = self.offset;
        let subrect =
            &self
                .data
                .base
                .subrect(offset_x, offset_y, self.rect.width(), self.rect.height());

        let transformed = if self.degrees < f32::EPSILON || self.degrees > 360.0 - f32::EPSILON {
            subrect.to_pixel_buffer()
        } else {
            subrect.rotsprite(self.degrees)
        };

        let (ax, ay) = self.anchor_corner.corner_offset(
            transformed.width() as f32,
            transformed.height() as f32,
            0.0,
            0.0,
        );

        for row in 0..transformed.height() {
            for col in 0..transformed.width() {
                let color = transformed.pixel(col, row);
                if color[3] == 0 {
                    continue;
                }
                let cell = Cell::new(ax + col as f32, ay + row as f32).color(color.into());
                f(cell);
            }
        }
    }
}

/// An `Image` paired with a list of sub-regions (`FrameData`) and a loop
/// mode, producing `Frame`s for drawing either by index or by elapsed time.
pub struct Sprite {
    data: Rc<ImageData>,
    frame_data: Vec<FrameData>,
    repeat: bool,
}

impl Sprite {
    pub fn new(image: &Image) -> Self {
        let frame_data = if image.frame_data.is_empty() {
            vec![FrameData {
                offset: (0, 0),
                size: (image.width(), image.height()),
                duration: None,
            }]
        } else {
            image.frame_data.clone()
        };

        Self {
            data: image.data.clone(),
            frame_data,
            repeat: false,
        }
    }

    /// Overwrites the frame data with `cols * rows` equal-size cells sliced
    /// from the image in row-major order, each defaulted to
    /// `DEFAULT_FRAME_DURATION`.
    pub fn grid(mut self, cols: u32, rows: u32) -> Self {
        let cell_w = self.data.base.width / cols;
        let cell_h = self.data.base.height / rows;

        self.frame_data = (0..rows)
            .flat_map(|row| (0..cols).map(move |col| (col, row)))
            .map(|(col, row)| FrameData {
                offset: (col * cell_w, row * cell_h),
                size: (cell_w, cell_h),
                duration: Some(DEFAULT_FRAME_DURATION),
            })
            .collect();

        self
    }

    pub fn repeat(mut self) -> Self {
        self.repeat = true;
        self
    }

    pub fn one_shot(mut self) -> Self {
        self.repeat = false;
        self
    }

    pub fn frame_count(&self) -> usize {
        self.frame_data.len()
    }

    /// Looks up a frame by index, wrapping via modulo.
    pub fn frame(&self, index: usize) -> Option<Frame> {
        let frame_data = self.frame_data.get(index)?;
        Some(self.frame_from_data(frame_data))
    }

    /// Looks up the frame active at elapsed time `t` (seconds). Wraps `t` via
    /// modulo the total duration when looping; clamps to the last frame
    /// otherwise.
    pub fn frame_at(&self, t: f32) -> Frame {
        let durations: Vec<f32> = self
            .frame_data
            .iter()
            .map(|f| f.duration.unwrap_or(DEFAULT_FRAME_DURATION))
            .collect();
        let total: f32 = durations.iter().sum();

        let t = if self.repeat && total > 0.0 {
            t.rem_euclid(total)
        } else {
            t.clamp(0.0, total)
        };

        let mut elapsed = 0.0;
        for (i, duration) in durations.iter().enumerate() {
            elapsed += duration;
            if t < elapsed || i == durations.len() - 1 {
                return self.frame_from_data(&self.frame_data[i]);
            }
        }

        self.frame_from_data(&self.frame_data[0])
    }

    fn frame_from_data(&self, frame_data: &FrameData) -> Frame {
        let (w, h) = frame_data.size;
        Frame {
            data: self.data.clone(),
            rect: Rect::from_top_left(0.0, 0.0, w, h),
            offset: frame_data.offset,
            anchor_corner: AnchorCorner::default(),
            degrees: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::srgb_to_linear;
    use image::{Rgba, RgbaImage};

    fn write_temp_png(name: &str, img: &RgbaImage) -> std::path::PathBuf {
        let mut path = std::env::temp_dir();
        path.push(format!("tiles_image_test_{name}.png"));
        img.save(&path).unwrap();
        path
    }

    fn write_temp_gif(name: &str, frames: &[(RgbaImage, u32)]) -> std::path::PathBuf {
        use image::Delay;
        use image::codecs::gif::GifEncoder;

        let mut path = std::env::temp_dir();
        path.push(format!("tiles_image_test_{name}.gif"));
        let file = File::create(&path).unwrap();
        let mut encoder = GifEncoder::new(file);
        let anim_frames = frames.iter().map(|(img, delay_ms)| {
            image::Frame::from_parts(img.clone(), 0, 0, Delay::from_numer_denom_ms(*delay_ms, 1))
        });
        encoder.encode_frames(anim_frames).unwrap();
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

    // --- Frame::rotate ---

    #[test]
    fn rotate_accumulates_across_calls() {
        let img = RgbaImage::from_pixel(2, 2, Rgba([255, 255, 255, 255]));
        let path = write_temp_png("rotate_accumulates", &img);

        let image = Image::from_path(&path).unwrap();
        let frame = image.instance().rotate(30.0).rotate(40.0);
        assert!((frame.degrees - 70.0).abs() < 1e-4);
    }

    #[test]
    fn rotate_wraps_past_360_degrees() {
        let img = RgbaImage::from_pixel(2, 2, Rgba([255, 255, 255, 255]));
        let path = write_temp_png("rotate_wraps", &img);

        let image = Image::from_path(&path).unwrap();
        let frame = image.instance().rotate(200.0).rotate(200.0);
        assert!((frame.degrees - 40.0).abs() < 1e-4);
    }

    #[test]
    fn rotate_by_zero_does_not_change_pixel_count() {
        let img = RgbaImage::from_pixel(4, 4, Rgba([255, 255, 255, 255]));
        let path = write_temp_png("rotate_zero", &img);

        let image = Image::from_path(&path).unwrap();
        let cells = image.instance().rotate(0.0).to_cells();
        assert_eq!(cells.len(), 16);
    }

    #[test]
    fn rotate_near_360_degrees_stays_a_no_op() {
        let img = RgbaImage::from_pixel(4, 4, Rgba([255, 255, 255, 255]));
        let path = write_temp_png("rotate_near_360", &img);

        let image = Image::from_path(&path).unwrap();
        let cells = image.instance().rotate(-1e-6).to_cells();
        assert_eq!(
            cells.len(),
            16,
            "degrees just below 360 should skip rotsprite"
        );
    }

    #[test]
    fn rotate_90_expands_cell_count_via_rotsprite() {
        let img = RgbaImage::from_pixel(4, 4, Rgba([255, 255, 255, 255]));
        let path = write_temp_png("rotate_90_cells", &img);

        let image = Image::from_path(&path).unwrap();
        let cells = image.instance().rotate(90.0).to_cells();
        // 90-degree rotations don't expand the bounding box.
        assert_eq!(cells.len(), 16);
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

    // --- PixelBuffer / PixelBufferView ---

    fn test_pixel_buffer(colors: &[[u8; 4]], width: u32, height: u32) -> PixelBuffer {
        PixelBuffer {
            pixels: rgba_buffer(colors).into(),
            width,
            height,
        }
    }

    #[test]
    fn pixel_buffer_len_is_width_height_times_four() {
        let buf = test_pixel_buffer(&[RED; 6], 3, 2);
        assert_eq!(buf.len(), 3 * 2 * 4);
    }

    #[test]
    fn pixel_buffer_pixel_reads_correct_pixel() {
        let buf = test_pixel_buffer(&[RED, GREEN, BLUE, YELLOW], 2, 2);
        assert_eq!(buf.pixel(0, 0), RED);
        assert_eq!(buf.pixel(1, 0), GREEN);
        assert_eq!(buf.pixel(0, 1), BLUE);
        assert_eq!(buf.pixel(1, 1), YELLOW);
    }

    #[test]
    #[should_panic(expected = "pixel (1, 0) out of bounds")]
    fn pixel_buffer_pixel_out_of_bounds_panics() {
        let buf = test_pixel_buffer(&[RED], 1, 1);
        buf.pixel(1, 0);
    }

    #[test]
    fn pixel_buffer_row_returns_full_row() {
        let buf = test_pixel_buffer(&[RED, GREEN, BLUE, YELLOW], 2, 2);
        assert_eq!(buf.row(1), [BLUE, YELLOW].concat().as_slice());
    }

    #[test]
    fn view_of_full_buffer_matches_source_dimensions_and_pixels() {
        let buf = test_pixel_buffer(&[RED, GREEN, BLUE, YELLOW], 2, 2);
        let view = buf.view();
        assert_eq!(view.width(), 2);
        assert_eq!(view.height(), 2);
        assert_eq!(view.pixel(1, 1), YELLOW);
    }

    #[test]
    fn subrect_indexes_relative_to_its_own_origin() {
        // 3x2 buffer; take the right 2x2 sub-rect (columns 1..3).
        let buf = test_pixel_buffer(&[RED, GREEN, BLUE, BLUE, YELLOW, RED], 3, 2);
        let sub = buf.subrect(1, 0, 2, 2);
        assert_eq!(sub.width(), 2);
        assert_eq!(sub.height(), 2);
        // sub-local (0,0) is parent (1,0) = GREEN; sub-local (1,1) is parent (2,1) = RED.
        assert_eq!(sub.pixel(0, 0), GREEN);
        assert_eq!(sub.pixel(1, 1), RED);
    }

    #[test]
    fn subrect_row_uses_parent_stride() {
        let buf = test_pixel_buffer(&[RED, GREEN, BLUE, BLUE, YELLOW, RED], 3, 2);
        let sub = buf.subrect(1, 0, 2, 2);
        assert_eq!(sub.row(1), [YELLOW, RED].concat().as_slice());
    }

    #[test]
    #[should_panic(expected = "out of bounds")]
    fn subrect_beyond_parent_bounds_panics() {
        let buf = test_pixel_buffer(&[RED; 4], 2, 2);
        buf.subrect(1, 1, 2, 2);
    }

    #[test]
    fn nested_subrect_composes_offsets() {
        let buf = test_pixel_buffer(
            &[RED, GREEN, BLUE, BLUE, YELLOW, RED, RED, RED, GREEN],
            3,
            3,
        );
        let outer = buf.subrect(1, 1, 2, 2); // local (0,0) = parent (1,1) = RED
        let inner = outer.subrect(1, 1, 1, 1); // local (0,0) = outer (1,1) = parent (2,2) = GREEN
        assert_eq!(inner.pixel(0, 0), GREEN);
    }

    #[test]
    fn to_pixel_buffer_copies_view_tightly_packed() {
        let buf = test_pixel_buffer(&[RED, GREEN, BLUE, BLUE, YELLOW, RED], 3, 2);
        let sub = buf.subrect(1, 0, 2, 2);
        let owned = sub.to_pixel_buffer();

        assert_eq!(owned.width(), 2);
        assert_eq!(owned.height(), 2);
        assert_eq!(owned.pixel(0, 0), GREEN);
        assert_eq!(owned.pixel(1, 0), BLUE);
        assert_eq!(owned.pixel(0, 1), YELLOW);
        assert_eq!(owned.pixel(1, 1), RED);
    }

    #[test]
    fn from_pixel_buffer_view_matches_to_pixel_buffer() {
        let buf = test_pixel_buffer(&[RED, GREEN, BLUE, YELLOW], 2, 2);
        let view = buf.subrect(1, 0, 1, 2);
        let owned: PixelBuffer = view.clone().into();
        assert_eq!(owned.width(), view.width());
        assert_eq!(owned.height(), view.height());
        assert_eq!(owned.pixel(0, 0), GREEN);
        assert_eq!(owned.pixel(0, 1), YELLOW);
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

    // --- Sprite ---

    /// Builds a 4x1-grid image, one solid color per cell, `cell_size` square.
    fn grid_image(name: &str, colors: &[[u8; 4]; 4], cell_size: u32) -> Image {
        let mut img = RgbaImage::new(cell_size * 4, cell_size);
        for (i, color) in colors.iter().enumerate() {
            for y in 0..cell_size {
                for x in 0..cell_size {
                    img.put_pixel(i as u32 * cell_size + x, y, Rgba(*color));
                }
            }
        }
        let path = write_temp_png(&format!("grid_{name}"), &img);
        Image::from_path(&path).unwrap()
    }

    #[test]
    fn frame_out_of_bounds_index_returns_none() {
        let image = grid_image("wraps", &[RED, GREEN, BLUE, YELLOW], 2);
        let sprite = Sprite::new(&image).grid(4, 1);

        assert!(sprite.frame(1).is_some());
        assert!(sprite.frame(5).is_none());
    }

    #[test]
    fn grid_slices_into_equal_cells() {
        let image = grid_image("slices", &[RED, GREEN, BLUE, YELLOW], 2);
        let sprite = Sprite::new(&image).grid(4, 1);

        assert_eq!(sprite.frame(0).unwrap().width(), 2);
        assert_eq!(sprite.frame(0).unwrap().height(), 2);
    }

    #[test]
    fn grid_frame_colors_match_source_cells() {
        let image = grid_image("colors", &[RED, GREEN, BLUE, YELLOW], 2);
        let sprite = Sprite::new(&image).grid(4, 1);

        for (i, expected) in [RED, GREEN, BLUE, YELLOW].iter().enumerate() {
            let cells = sprite.frame(i).unwrap().to_cells();
            let expected_linear = srgb_to_linear(expected[0] as f32 / 255.0);
            assert!(
                (cells[0].color[0] - expected_linear).abs() < 1e-5,
                "frame {i} did not match expected cell color"
            );
        }
    }

    #[test]
    fn no_grid_call_treats_image_as_single_frame() {
        let image = grid_image("single_frame", &[RED, GREEN, BLUE, YELLOW], 2);
        let sprite = Sprite::new(&image);

        assert_eq!(sprite.frame(0).unwrap().width(), image.width());
        assert_eq!(sprite.frame(0).unwrap().height(), image.height());
    }

    #[test]
    fn frame_at_picks_frame_by_elapsed_time() {
        let image = grid_image("elapsed_time", &[RED, GREEN, BLUE, YELLOW], 2);
        // 100ms per frame by default.
        let sprite = Sprite::new(&image).grid(4, 1);

        let at_start = sprite.frame_at(0.0).to_cells();
        let at_frame_two = sprite.frame_at(0.25).to_cells();
        assert_eq!(
            at_start[0].color,
            sprite.frame(0).unwrap().to_cells()[0].color
        );
        assert_eq!(
            at_frame_two[0].color,
            sprite.frame(2).unwrap().to_cells()[0].color
        );
    }

    #[test]
    fn frame_at_one_shot_clamps_to_last_frame_past_end() {
        let image = grid_image("one_shot_clamp", &[RED, GREEN, BLUE, YELLOW], 2);
        let sprite = Sprite::new(&image).grid(4, 1).one_shot();

        let past_end = sprite.frame_at(1000.0).to_cells();
        let last = sprite.frame(3).unwrap().to_cells();
        assert_eq!(past_end[0].color, last[0].color);
    }

    #[test]
    fn frame_at_looping_wraps_past_total_duration() {
        let image = grid_image("looping_wrap", &[RED, GREEN, BLUE, YELLOW], 2);
        let sprite = Sprite::new(&image).grid(4, 1).repeat();

        // Total duration is 0.4s; 0.45s should wrap to the same frame as 0.05s.
        let wrapped = sprite.frame_at(0.45).to_cells();
        let expected = sprite.frame_at(0.05).to_cells();
        assert_eq!(wrapped[0].color, expected[0].color);
    }

    #[test]
    fn one_shot_is_default_before_looping_called() {
        let image = grid_image("default_one_shot", &[RED, GREEN, BLUE, YELLOW], 2);
        let sprite = Sprite::new(&image).grid(4, 1);

        let past_end = sprite.frame_at(1000.0).to_cells();
        let last = sprite.frame(3).unwrap().to_cells();
        assert_eq!(past_end[0].color, last[0].color);
    }

    // --- GIF decoding ---

    #[test]
    fn gif_decode_produces_one_frame_data_entry_per_frame() {
        let frames = [
            (RgbaImage::from_pixel(2, 2, Rgba(RED)), 50),
            (RgbaImage::from_pixel(2, 2, Rgba(GREEN)), 150),
            (RgbaImage::from_pixel(2, 2, Rgba(BLUE)), 200),
        ];
        let path = write_temp_gif("frame_count", &frames);

        let image = Image::from_path(&path).unwrap();
        assert_eq!(image.frame_data.len(), 3);
    }

    #[test]
    fn gif_decode_reads_real_per_frame_delay() {
        let frames = [
            (RgbaImage::from_pixel(2, 2, Rgba(RED)), 50),
            (RgbaImage::from_pixel(2, 2, Rgba(GREEN)), 150),
        ];
        let path = write_temp_gif("delay", &frames);

        let image = Image::from_path(&path).unwrap();
        assert!((image.frame_data[0].duration.unwrap() - 0.05).abs() < 1e-3);
        assert!((image.frame_data[1].duration.unwrap() - 0.15).abs() < 1e-3);
    }

    #[test]
    fn gif_backed_sprite_plays_back_via_frame_at() {
        let frames = [
            (RgbaImage::from_pixel(2, 2, Rgba(RED)), 100),
            (RgbaImage::from_pixel(2, 2, Rgba(GREEN)), 100),
        ];
        let path = write_temp_gif("playback", &frames);

        let image = Image::from_path(&path).unwrap();
        let sprite = Sprite::new(&image);

        let first = sprite.frame_at(0.0).to_cells();
        let second = sprite.frame_at(0.15).to_cells();
        let red_linear = srgb_to_linear(RED[0] as f32 / 255.0);
        let green_linear = srgb_to_linear(GREEN[1] as f32 / 255.0);
        assert!((first[0].color[0] - red_linear).abs() < 1e-5);
        assert!((second[0].color[1] - green_linear).abs() < 1e-5);
    }
}
