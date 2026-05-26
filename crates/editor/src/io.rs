use std::fs;
use std::path::Path;
use crate::document::{Document, Canvas, Palette, Grid};

pub fn save_tiles(doc: &Document, path: &Path) -> Result<(), String> {
    let content = toml::to_string_pretty(doc).map_err(|e| e.to_string())?;
    fs::write(path, content).map_err(|e| e.to_string())
}

pub fn load_tiles(path: &Path) -> Result<Document, String> {
    let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
    toml::from_str(&content).map_err(|e| e.to_string())
}

pub fn export_png(doc: &Document, path: &Path) -> Result<(), String> {
    let w = doc.canvas.width;
    let h = doc.canvas.height;
    let mut rgba_data = vec![0u8; w * h * 4];

    for y in 0..h {
        for x in 0..w {
            let idx = doc.canvas.get(x, y);
            let color = doc.palette.color_at(idx);
            let offset = (y * w + x) * 4;
            rgba_data[offset] = (color[0] * 255.0) as u8;
            rgba_data[offset + 1] = (color[1] * 255.0) as u8;
            rgba_data[offset + 2] = (color[2] * 255.0) as u8;
            rgba_data[offset + 3] = (color[3] * 255.0) as u8;
        }
    }

    write_png(path, w as u32, h as u32, &rgba_data)
}

fn write_png(path: &Path, width: u32, height: u32, rgba: &[u8]) -> Result<(), String> {
    use std::io::BufWriter;
    let file = fs::File::create(path).map_err(|e| e.to_string())?;
    let ref mut w = BufWriter::new(file);

    let mut encoder = png::Encoder::new(w, width, height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);

    let mut writer = encoder.write_header().map_err(|e| e.to_string())?;
    writer.write_image_data(rgba).map_err(|e| e.to_string())?;
    Ok(())
}
