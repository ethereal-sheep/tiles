use std::collections::BTreeMap;
use std::fs;
use std::io::Write;
use std::path::Path;

fn main() {
    let fonts_dir = Path::new("fonts");
    let out_dir = Path::new("src/font/generated");
    fs::create_dir_all(out_dir).unwrap();

    println!("cargo::rerun-if-changed=fonts");

    let mut font_entries: Vec<(String, String, usize, usize)> = Vec::new();

    for entry in fs::read_dir(fonts_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();

        if path.extension().map_or(false, |e| e == "bdf") {
            let stem = path.file_stem().unwrap().to_str().unwrap().to_string();
            let font = parse_bdf(&path);

            if let Some(font) = font {
                let mod_name = sanitize_name(&stem);
                let const_name = mod_name.to_uppercase();
                let out_path = out_dir.join(format!("{mod_name}.rs"));
                write_font_module(&out_path, &const_name, &font);
                font_entries.push((mod_name, const_name, font.width, font.height));
            }
        }
    }

    font_entries.sort_by(|a, b| a.0.cmp(&b.0));

    let mod_path = out_dir.join("mod.rs");
    let mut f = fs::File::create(mod_path).unwrap();
    for (mod_name, const_name, _, _) in &font_entries {
        writeln!(f, "mod {mod_name};").unwrap();
        writeln!(f, "pub use {mod_name}::{const_name};").unwrap();
    }
}

fn sanitize_name(s: &str) -> String {
    let raw: String = s
        .chars()
        .map(|c| if c.is_alphanumeric() { c.to_ascii_lowercase() } else { '_' })
        .collect::<String>()
        .trim_matches('_')
        .to_string();
    if raw.starts_with(|c: char| c.is_ascii_digit()) {
        format!("font_{raw}")
    } else {
        raw
    }
}

struct BdfFont {
    width: usize,
    height: usize,
    glyphs: BTreeMap<u8, Vec<Vec<bool>>>,
}

fn parse_bdf(path: &Path) -> Option<BdfFont> {
    let content = fs::read_to_string(path).ok()?;
    let lines: Vec<&str> = content.lines().collect();

    let mut fbb_w = 0usize;
    let mut fbb_h = 0usize;
    let mut fbb_y_off: i32 = 0;

    let mut glyphs: BTreeMap<u8, Vec<Vec<bool>>> = BTreeMap::new();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i];

        if line.starts_with("FONTBOUNDINGBOX ") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 5 {
                fbb_w = parts[1].parse().unwrap_or(0);
                fbb_h = parts[2].parse().unwrap_or(0);
                fbb_y_off = parts[4].parse().unwrap_or(0);
            }
        }

        if line.starts_with("STARTCHAR ") {
            let mut encoding: Option<u32> = None;
            let mut bbx_w = fbb_w;
            let mut bbx_h = fbb_h;
            let mut bbx_x_off: i32 = 0;
            let mut bbx_y_off: i32 = fbb_y_off;
            let mut bitmap_lines: Vec<&str> = Vec::new();
            let mut in_bitmap = false;

            i += 1;
            while i < lines.len() && lines[i] != "ENDCHAR" {
                let l = lines[i];
                if l.starts_with("ENCODING ") {
                    encoding = l.split_whitespace().nth(1).and_then(|s| s.parse().ok());
                } else if l.starts_with("BBX ") {
                    let parts: Vec<&str> = l.split_whitespace().collect();
                    if parts.len() >= 5 {
                        bbx_w = parts[1].parse().unwrap_or(fbb_w);
                        bbx_h = parts[2].parse().unwrap_or(fbb_h);
                        bbx_x_off = parts[3].parse().unwrap_or(0);
                        bbx_y_off = parts[4].parse().unwrap_or(fbb_y_off);
                    }
                } else if l == "BITMAP" {
                    in_bitmap = true;
                } else if in_bitmap {
                    bitmap_lines.push(l);
                }
                i += 1;
            }

            if let Some(enc) = encoding {
                if enc >= 32 && enc <= 126 {
                    let grid = rasterize_glyph(
                        &bitmap_lines, fbb_w, fbb_h, fbb_y_off, bbx_w, bbx_h, bbx_x_off, bbx_y_off,
                    );
                    glyphs.insert(enc as u8, grid);
                }
            }
        }

        i += 1;
    }

    if fbb_w == 0 || fbb_h == 0 || glyphs.is_empty() {
        return None;
    }

    Some(BdfFont {
        width: fbb_w,
        height: fbb_h,
        glyphs,
    })
}

fn rasterize_glyph(
    bitmap_lines: &[&str],
    fbb_w: usize,
    fbb_h: usize,
    fbb_y_off: i32,
    bbx_w: usize,
    bbx_h: usize,
    bbx_x_off: i32,
    bbx_y_off: i32,
) -> Vec<Vec<bool>> {
    let mut grid = vec![vec![false; fbb_w]; fbb_h];

    // BDF y-coordinates: baseline is at fbb_y_off rows from the bottom of the fbb.
    // A glyph's bitmap bottom edge is bbx_y_off rows from the baseline.
    // So the glyph's bottom row in the grid is at:
    //   grid_bottom = (fbb_h - 1) - (bbx_y_off - fbb_y_off)
    // And its top row is:
    //   grid_top = grid_bottom - (bbx_h - 1)
    let grid_bottom = (fbb_h as i32 - 1) - (bbx_y_off - fbb_y_off);
    let y_start = grid_bottom - (bbx_h as i32 - 1);

    for (row_idx, hex_str) in bitmap_lines.iter().enumerate() {
        let row_val = u64::from_str_radix(hex_str.trim(), 16).unwrap_or(0);
        let hex_bits = hex_str.trim().len() * 4;

        let y = y_start + row_idx as i32;
        if y < 0 {
            continue;
        }
        let y = y as usize;
        if y >= fbb_h {
            break;
        }

        for col in 0..bbx_w {
            let bit_pos = hex_bits as i32 - 1 - col as i32;
            let pixel_on = if bit_pos >= 0 {
                (row_val >> bit_pos) & 1 == 1
            } else {
                false
            };

            let x = bbx_x_off + col as i32;
            if pixel_on && x >= 0 && (x as usize) < fbb_w {
                grid[y][x as usize] = true;
            }
        }
    }

    grid
}

fn write_font_module(path: &Path, const_name: &str, font: &BdfFont) {
    let mut f = fs::File::create(path).unwrap();

    let bytes_per_row = (font.width + 7) / 8;
    let bytes_per_glyph = bytes_per_row * font.height;
    let total_glyphs = 95; // ASCII 32-126

    writeln!(f, "use crate::font::Font;").unwrap();
    writeln!(f).unwrap();
    writeln!(
        f,
        "pub static {const_name}: Font = Font::new({}, {}, 1, GLYPH_DATA, 32, 126);",
        font.width, font.height
    ).unwrap();
    writeln!(f).unwrap();
    writeln!(f, "#[rustfmt::skip]").unwrap();
    writeln!(f, "static GLYPH_DATA: &[u8] = &[").unwrap();

    for code in 32u8..=126u8 {
        let ch = code as char;
        let display_ch = if ch == '\\' {
            "\\\\".to_string()
        } else if ch == '\'' {
            "\\'".to_string()
        } else {
            ch.to_string()
        };
        writeln!(f, "    // '{}' ({})", display_ch, code).unwrap();

        if let Some(grid) = font.glyphs.get(&code) {
            for row in 0..font.height {
                let mut byte_vals = Vec::new();
                for byte_idx in 0..bytes_per_row {
                    let mut val = 0u8;
                    for bit in 0..8 {
                        let col = byte_idx * 8 + bit;
                        if col < font.width && row < grid.len() && col < grid[row].len() && grid[row][col] {
                            val |= 1 << (7 - bit);
                        }
                    }
                    byte_vals.push(val);
                }
                let hex_str: Vec<String> = byte_vals.iter().map(|b| format!("0x{:02X}", b)).collect();
                writeln!(f, "    {},", hex_str.join(", ")).unwrap();
            }
        } else {
            for _ in 0..font.height {
                let zeros: Vec<String> = (0..bytes_per_row).map(|_| "0x00".to_string()).collect();
                writeln!(f, "    {},", zeros.join(", ")).unwrap();
            }
        }
    }

    writeln!(f, "];").unwrap();

    let expected = total_glyphs * bytes_per_glyph;
    let _ = expected; // sanity check could go here
}
