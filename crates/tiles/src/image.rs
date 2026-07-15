use core::fmt;
use std::path::Path;
use std::rc::Rc;

use crate::anchor::{AnchorCorner, corner_offset};
use crate::cell::Cell;
use crate::color::{Color, srgb_to_linear};
use crate::drawable::Drawable;

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

pub struct Image {
    pixels: Rc<[u8]>,
    width: u32,
    height: u32,
}

impl fmt::Debug for Image {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Image")
            .field("width", &self.width)
            .field("height", &self.height)
            .finish()
    }
}

impl Image {
    pub fn from_path(path: impl AsRef<Path>) -> Result<Self, ImageError> {
        let decoded = image::open(path)?.to_rgba8();
        let (width, height) = decoded.dimensions();
        Ok(Self {
            pixels: Rc::from(decoded.into_raw()),
            width,
            height,
        })
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn instance(&self) -> Frame {
        self.frame(0)
    }

    pub fn frame(&self, _index: usize) -> Frame {
        Frame {
            pixels: self.pixels.clone(),
            width: self.width,
            height: self.height,
            position: (0.0, 0.0),
            anchor_corner: AnchorCorner::default(),
        }
    }
}

#[derive(Clone)]
pub struct Frame {
    pixels: Rc<[u8]>,
    width: u32,
    height: u32,
    position: (f32, f32),
    anchor_corner: AnchorCorner,
}

impl fmt::Debug for Frame {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Frame")
            .field("width", &self.width)
            .field("height", &self.height)
            .field("position", &self.position)
            .finish()
    }
}

impl Frame {
    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn position(mut self, x: f32, y: f32) -> Self {
        self.position = (x, y);
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
            self.width as f32,
            self.height as f32,
            0.0,
            0.0,
        )
    }
}

impl Drawable for Frame {
    fn origin(&self) -> Option<(f32, f32)> {
        Some(self.position)
    }

    fn emit_local_cells(&self, f: &mut dyn FnMut(Cell)) {
        let (ax, ay) = self.anchor_offset();

        for row in 0..self.height {
            for col in 0..self.width {
                let idx = ((row * self.width + col) * 4) as usize;
                let [r, g, b, a] = self.pixels[idx..idx + 4] else {
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
        assert_eq!(image.instance().to_cells().len(), image.frame(0).to_cells().len());
    }

    #[test]
    fn frame_ignores_index() {
        let img = RgbaImage::from_pixel(2, 2, Rgba([255, 255, 255, 255]));
        let path = write_temp_png("frame_ignores_index", &img);

        let image = Image::from_path(&path).unwrap();
        assert_eq!(image.frame(0).to_cells().len(), image.frame(7).to_cells().len());
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
        assert!(Rc::ptr_eq(&frame.pixels, &cloned.pixels));
    }
}
