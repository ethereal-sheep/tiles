use std::collections::BTreeMap;
use std::fs;
use std::io::Write;
use std::path::Path;

fn main() {
    let fonts_dir = Path::new("fonts");
    let out_dir = Path::new("src/font/generated");
    fs::create_dir_all(out_dir).unwrap();

    println!("cargo::rerun-if-changed=fonts");

    // Clean old generated files
    if let Ok(entries) = fs::read_dir(out_dir) {
        for entry in entries {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.extension().map_or(false, |e| e == "rs") {
                fs::remove_file(path).unwrap();
            }
        }
    }

    let mut font_entries: Vec<(String, String)> = Vec::new();

    for entry in fs::read_dir(fonts_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();

        if path.extension().map_or(false, |e| e == "bdf") {
            let stem = path.file_stem().unwrap().to_str().unwrap().to_string();
            let font = parse_bdf(&path);

            if let Some(font) = font {
                let mod_name = make_font_name(&stem, font.is_mono, font.a_bbx_w, font.a_bbx_h);
                let const_name = mod_name.to_uppercase();
                let out_path = out_dir.join(format!("{mod_name}.rs"));
                write_font_module(&out_path, &const_name, &font);
                font_entries.push((mod_name, const_name));
            }
        }
    }

    font_entries.sort_by(|a, b| a.0.cmp(&b.0));

    let mod_path = out_dir.join("mod.rs");
    let mut f = fs::File::create(mod_path).unwrap();
    for (mod_name, const_name) in &font_entries {
        writeln!(f, "mod {mod_name};").unwrap();
        writeln!(f, "pub use {mod_name}::{const_name};").unwrap();
    }
}

fn strip_trailing_size_info(s: &str) -> &str {
    // Strip trailing _WxH pattern (e.g. _12x24)
    let mut candidate = s;
    if let Some(pos) = candidate.rfind('_') {
        let after = &candidate[pos + 1..];
        if has_wxh_pattern(after) {
            candidate = &candidate[..pos];
        }
    }
    // Strip trailing _N pattern (point-size suffix like _10, _11)
    if let Some(pos) = candidate.rfind('_') {
        let after = &candidate[pos + 1..];
        if !after.is_empty() && after.chars().all(|c| c.is_ascii_digit()) {
            let before = &candidate[..pos];
            if !before.is_empty() {
                return before;
            }
        }
    }
    // If the string is purely a WxH pattern (like "6x12") or a WxH with style suffix (like "6x13b"),
    // return empty to signal no base name
    let without_style = candidate.trim_end_matches(|c: char| c.is_ascii_alphabetic() && c != 'x');
    if !without_style.is_empty() && without_style.chars().all(|c| c.is_ascii_digit() || c == 'x') && has_wxh_pattern(without_style) {
        return "";
    }
    candidate
}

fn has_wxh_pattern(s: &str) -> bool {
    let chars: Vec<char> = s.chars().collect();
    for i in 0..chars.len() {
        if chars[i] == 'x' && i > 0 && i + 1 < chars.len() {
            let before = chars[..i].iter().rev().take_while(|c| c.is_ascii_digit()).count();
            let after = chars[i + 1..].iter().take_while(|c| c.is_ascii_digit()).count();
            if before > 0 && after > 0 {
                return true;
            }
        }
    }
    false
}

fn make_font_name(stem: &str, is_mono: bool, a_bbx_w: usize, a_bbx_h: usize) -> String {
    let dims = format!("{a_bbx_w}x{a_bbx_h}");

    let raw: String = stem
        .chars()
        .map(|c| if c.is_alphanumeric() { c.to_ascii_lowercase() } else { '_' })
        .collect::<String>()
        .trim_matches('_')
        .to_string();

    // Check if stem is purely a WxH pattern with optional style suffix (like "6x13", "6x13b", "9x15b")
    let without_style = raw.trim_end_matches(|c: char| c.is_ascii_alphabetic() && c != 'x');
    let is_pure_dim_name = !without_style.is_empty()
        && without_style.chars().all(|c| c.is_ascii_digit() || c == 'x')
        && has_wxh_pattern(without_style);

    let named = if is_pure_dim_name {
        // Extract style suffix (b=bold, o=oblique)
        let style_suffix = &raw[without_style.len()..];
        let style_prefix = match style_suffix {
            "b" => "bold_",
            "o" => "oblique_",
            _ => "",
        };
        if is_mono {
            format!("mono_{style_prefix}{dims}")
        } else {
            format!("{style_prefix}{dims}")
        }
    } else {
        let base = strip_trailing_size_info(&raw).to_string();
        if base.is_empty() {
            if is_mono {
                format!("mono_{dims}")
            } else {
                dims.clone()
            }
        } else if is_mono {
            format!("mono_{base}_{dims}")
        } else {
            format!("{base}_{dims}")
        }
    };

    ensure_valid_ident(&named)
}

fn ensure_valid_ident(s: &str) -> String {
    if s.starts_with(|c: char| c.is_ascii_digit()) {
        format!("font_{s}")
    } else {
        s.to_string()
    }
}

struct BdfFont {
    height: usize,
    max_dwidth: usize,
    is_mono: bool,
    a_bbx_w: usize,
    a_bbx_h: usize,
    widths: Vec<u8>,
    glyphs: BTreeMap<u8, Vec<Vec<bool>>>,
}

fn parse_bdf(path: &Path) -> Option<BdfFont> {
    let content = fs::read_to_string(path).ok()?;
    let lines: Vec<&str> = content.lines().collect();

    let mut fbb_w = 0usize;
    let mut fbb_h = 0usize;
    let mut fbb_y_off: i32 = 0;

    let mut font_ascent: Option<i32> = None;
    let mut font_descent: Option<i32> = None;
    let mut spacing: Option<String> = None;

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
        } else if line.starts_with("FONT_ASCENT ") {
            font_ascent = line.split_whitespace().nth(1).and_then(|s| s.parse().ok());
        } else if line.starts_with("FONT_DESCENT ") {
            font_descent = line.split_whitespace().nth(1).and_then(|s| s.parse().ok());
        } else if line.starts_with("SPACING ") {
            spacing = line.split_whitespace().nth(1).map(|s| s.trim_matches('"').to_uppercase());
        }

        if line.starts_with("STARTCHAR ") {
            break;
        }
        i += 1;
    }

    let cell_h = if let (Some(asc), Some(desc)) = (font_ascent, font_descent) {
        (asc + desc) as usize
    } else {
        fbb_h
    };

    let baseline_from_top = font_ascent.unwrap_or(fbb_h as i32 - (-fbb_y_off) as i32) as usize;

    let mut glyphs: BTreeMap<u8, Vec<Vec<bool>>> = BTreeMap::new();
    let mut dwidths: BTreeMap<u8, usize> = BTreeMap::new();
    let mut bbx_data: BTreeMap<u8, (usize, usize)> = BTreeMap::new(); // (bbx_w, bbx_h)
    let mut max_dwidth: usize = 0;

    i = 0;
    while i < lines.len() {
        let line = lines[i];

        if line.starts_with("STARTCHAR ") {
            let mut encoding: Option<u32> = None;
            let mut bbx_w = fbb_w;
            let mut bbx_h = fbb_h;
            let mut bbx_x_off: i32 = 0;
            let mut bbx_y_off: i32 = fbb_y_off;
            let mut dwidth: Option<usize> = None;
            let mut bitmap_lines: Vec<&str> = Vec::new();
            let mut in_bitmap = false;

            i += 1;
            while i < lines.len() && lines[i] != "ENDCHAR" {
                let l = lines[i];
                if l.starts_with("ENCODING ") {
                    encoding = l.split_whitespace().nth(1).and_then(|s| s.parse().ok());
                } else if l.starts_with("DWIDTH ") {
                    dwidth = l.split_whitespace().nth(1).and_then(|s| s.parse().ok());
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
                    if let Some(dw) = dwidth {
                        max_dwidth = max_dwidth.max(dw);
                        dwidths.insert(enc as u8, dw);
                    }
                    bbx_data.insert(enc as u8, (bbx_w, bbx_h));
                    let grid = rasterize_glyph_cell(
                        &bitmap_lines, cell_h, baseline_from_top, bbx_w, bbx_h, bbx_x_off, bbx_y_off,
                    );
                    glyphs.insert(enc as u8, grid);
                }
            }
        }

        i += 1;
    }

    let cell_w = if max_dwidth > 0 { max_dwidth } else { fbb_w };

    if cell_w == 0 || cell_h == 0 || glyphs.is_empty() {
        return None;
    }

    // Must have 'A' (65)
    let a_bbx = bbx_data.get(&65)?;
    let a_bbx_w = a_bbx.0;
    let a_bbx_h = a_bbx.1;

    // Determine mono vs proportional
    let is_mono = match spacing.as_deref() {
        Some("C") | Some("M") => true,
        Some("P") => false,
        _ => {
            // Fallback: check if all dwidths are identical
            let first = dwidths.values().next().copied();
            first.is_some() && dwidths.values().all(|&dw| Some(dw) == first)
        }
    };

    // Build widths array (95 entries for ASCII 32-126)
    let mut widths: Vec<u8> = Vec::with_capacity(95);
    for code in 32u8..=126u8 {
        let w = dwidths.get(&code).copied().unwrap_or(cell_w);
        widths.push(w as u8);
    }

    Some(BdfFont {
        height: cell_h,
        max_dwidth: cell_w,
        is_mono,
        a_bbx_w,
        a_bbx_h,
        widths,
        glyphs,
    })
}

fn rasterize_glyph_cell(
    bitmap_lines: &[&str],
    cell_h: usize,
    baseline_from_top: usize,
    bbx_w: usize,
    bbx_h: usize,
    bbx_x_off: i32,
    bbx_y_off: i32,
) -> Vec<Vec<bool>> {
    let grid_w = 64;
    let mut grid = vec![vec![false; grid_w]; cell_h];

    let glyph_bottom = baseline_from_top as i32 - bbx_y_off;
    let y_start = glyph_bottom - bbx_h as i32;

    for (row_idx, hex_str) in bitmap_lines.iter().enumerate() {
        let row_val = u64::from_str_radix(hex_str.trim(), 16).unwrap_or(0);
        let hex_bits = hex_str.trim().len() * 4;

        let y = y_start + row_idx as i32;
        if y < 0 {
            continue;
        }
        let y = y as usize;
        if y >= cell_h {
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
            if pixel_on && x >= 0 && (x as usize) < grid_w {
                grid[y][x as usize] = true;
            }
        }
    }

    grid
}

fn write_font_module(path: &Path, const_name: &str, font: &BdfFont) {
    let mut f = fs::File::create(path).unwrap();

    let bytes_per_row = (font.max_dwidth + 7) / 8;

    writeln!(f, "use crate::font::Font;").unwrap();
    writeln!(f).unwrap();

    // Write widths array
    writeln!(f, "#[rustfmt::skip]").unwrap();
    write!(f, "static WIDTHS: &[u8] = &[").unwrap();
    for (i, w) in font.widths.iter().enumerate() {
        if i > 0 { write!(f, ", ").unwrap(); }
        write!(f, "{}", w).unwrap();
    }
    writeln!(f, "];").unwrap();
    writeln!(f).unwrap();

    writeln!(
        f,
        "pub static {const_name}: Font = Font::new({}, {}, WIDTHS, GLYPH_DATA, 32, 126);",
        font.height, bytes_per_row
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
                        if col < font.max_dwidth && row < grid.len() && col < grid[row].len() && grid[row][col] {
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
}
