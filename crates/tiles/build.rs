use std::collections::BTreeMap;
use std::fs;
use std::io::Write;
use std::path::Path;

fn main() {
    let fonts_dir = Path::new("fonts");
    let out_dir = Path::new("src/font/generated");
    fs::create_dir_all(out_dir).unwrap();

    println!("cargo::rerun-if-changed=fonts");

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
    let mut candidate = s;
    if let Some(pos) = candidate.rfind('_') {
        let after = &candidate[pos + 1..];
        if has_wxh_pattern(after) {
            candidate = &candidate[..pos];
        }
    }
    if let Some(pos) = candidate.rfind('_') {
        let after = &candidate[pos + 1..];
        if !after.is_empty() && after.chars().all(|c| c.is_ascii_digit()) {
            let before = &candidate[..pos];
            if !before.is_empty() {
                return before;
            }
        }
    }
    let without_style = candidate.trim_end_matches(|c: char| c.is_ascii_alphabetic() && c != 'x');
    if !without_style.is_empty()
        && without_style
            .chars()
            .all(|c| c.is_ascii_digit() || c == 'x')
        && has_wxh_pattern(without_style)
    {
        return "";
    }
    candidate
}

fn has_wxh_pattern(s: &str) -> bool {
    let chars: Vec<char> = s.chars().collect();
    for i in 0..chars.len() {
        if chars[i] == 'x' && i > 0 && i + 1 < chars.len() {
            let before = chars[..i]
                .iter()
                .rev()
                .take_while(|c| c.is_ascii_digit())
                .count();
            let after = chars[i + 1..]
                .iter()
                .take_while(|c| c.is_ascii_digit())
                .count();
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
        .map(|c| {
            if c.is_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect::<String>()
        .trim_matches('_')
        .to_string();

    let without_style = raw.trim_end_matches(|c: char| c.is_ascii_alphabetic() && c != 'x');
    let is_pure_dim_name = !without_style.is_empty()
        && without_style
            .chars()
            .all(|c| c.is_ascii_digit() || c == 'x')
        && has_wxh_pattern(without_style);

    let named = if is_pure_dim_name {
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
    is_mono: bool,
    a_bbx_w: usize,
    a_bbx_h: usize,
    glyphs: BTreeMap<u8, GlyphData>,
}

struct GlyphData {
    dwidth: usize,
    tight_width: u8,
    tight_height: u8,
    tight_top: usize,
    tight_bytes_per_row: usize,
    tight_data: Vec<u8>,
}

fn compute_tight_bounds(
    grid: &[Vec<bool>],
    cell_h: usize,
    cell_w: usize,
) -> (u8, u8, usize, usize) {
    let mut min_col = cell_w;
    let mut max_col = 0usize;
    let mut min_row = cell_h;
    let mut max_row = 0usize;

    for row in 0..cell_h {
        for col in 0..cell_w {
            if row < grid.len() && col < grid[row].len() && grid[row][col] {
                min_col = min_col.min(col);
                max_col = max_col.max(col);
                min_row = min_row.min(row);
                max_row = max_row.max(row);
            }
        }
    }

    if max_col < min_col || max_row < min_row {
        return (0, 0, 0, 0);
    }

    let tight_width = (max_col - min_col + 1) as u8;
    let tight_height = (max_row - min_row + 1) as u8;
    (tight_width, tight_height, min_row, min_col)
}

fn pack_tight_data(
    grid: &[Vec<bool>],
    tight_top: usize,
    tight_left: usize,
    tight_width: u8,
    tight_height: u8,
) -> (Vec<u8>, usize) {
    let w = tight_width as usize;
    let h = tight_height as usize;
    let bytes_per_row = (w + 7) / 8;
    let mut data = Vec::with_capacity(bytes_per_row * h);

    for row in tight_top..tight_top + h {
        for byte_idx in 0..bytes_per_row {
            let mut val = 0u8;
            for bit in 0..8 {
                let col = tight_left + byte_idx * 8 + bit;
                if col < tight_left + w
                    && row < grid.len()
                    && col < grid[row].len()
                    && grid[row][col]
                {
                    val |= 1 << (7 - bit);
                }
            }
            data.push(val);
        }
    }

    (data, bytes_per_row)
}

fn compute_default_gap(glyphs: &BTreeMap<u8, GlyphData>, is_mono: bool) -> usize {
    if is_mono {
        return 0;
    }

    let mut gap_counts: BTreeMap<usize, usize> = BTreeMap::new();

    for glyph in glyphs.values() {
        if glyph.tight_width == 0 {
            continue;
        }
        let trailing = glyph.dwidth.saturating_sub(glyph.tight_width as usize);
        *gap_counts.entry(trailing).or_insert(0) += 1;
    }

    gap_counts
        .into_iter()
        .max_by_key(|&(_, count)| count)
        .map(|(gap, _)| gap)
        .unwrap_or(0)
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
            spacing = line
                .split_whitespace()
                .nth(1)
                .map(|s| s.trim_matches('"').to_uppercase());
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

    let mut raw_glyphs: BTreeMap<u8, Vec<Vec<bool>>> = BTreeMap::new();
    let mut dwidths: BTreeMap<u8, usize> = BTreeMap::new();
    let mut bbx_data: BTreeMap<u8, (usize, usize)> = BTreeMap::new();
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
                        &bitmap_lines,
                        cell_h,
                        baseline_from_top,
                        bbx_w,
                        bbx_h,
                        bbx_x_off,
                        bbx_y_off,
                    );
                    raw_glyphs.insert(enc as u8, grid);
                }
            }
        }

        i += 1;
    }

    let cell_w = if max_dwidth > 0 { max_dwidth } else { fbb_w };

    if cell_w == 0 || cell_h == 0 || raw_glyphs.is_empty() {
        return None;
    }

    let a_bbx = bbx_data.get(&65)?;
    let a_bbx_w = a_bbx.0;
    let a_bbx_h = a_bbx.1;

    let is_mono = match spacing.as_deref() {
        Some("C") | Some("M") => true,
        Some("P") => false,
        _ => {
            let first = dwidths.values().next().copied();
            first.is_some() && dwidths.values().all(|&dw| Some(dw) == first)
        }
    };

    let mut glyphs: BTreeMap<u8, GlyphData> = BTreeMap::new();

    for code in 32u8..=126u8 {
        let dw = dwidths.get(&code).copied().unwrap_or(cell_w);
        if let Some(grid) = raw_glyphs.get(&code) {
            let (tight_width, tight_height, tight_top, tight_left) =
                compute_tight_bounds(grid, cell_h, cell_w);

            // For mono fonts: keep full advance width (don't strip horizontal padding)
            // For all fonts: if glyph has no lit pixels, use dwidth as width (e.g. space)
            let (final_width, final_left) = if is_mono || tight_width == 0 {
                (dw as u8, 0usize)
            } else {
                (tight_width, tight_left)
            };

            let (tight_data, tight_bytes_per_row) =
                pack_tight_data(grid, tight_top, final_left, final_width, tight_height);
            glyphs.insert(
                code,
                GlyphData {
                    dwidth: dw,
                    tight_width: final_width,
                    tight_height,
                    tight_top,
                    tight_bytes_per_row,
                    tight_data,
                },
            );
        } else {
            glyphs.insert(
                code,
                GlyphData {
                    dwidth: dw,
                    tight_width: 0,
                    tight_height: 0,
                    tight_top: 0,
                    tight_bytes_per_row: 0,
                    tight_data: Vec::new(),
                },
            );
        }
    }

    Some(BdfFont {
        height: cell_h,
        is_mono,
        a_bbx_w,
        a_bbx_h,
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

    let default_gap = compute_default_gap(&font.glyphs, font.is_mono);

    writeln!(f, "use crate::font::{{Font, Glyph}};").unwrap();
    writeln!(f).unwrap();

    // Write glyph data blobs
    for code in 32u8..=126u8 {
        let glyph = &font.glyphs[&code];
        let ch = code as char;
        let display_ch = if ch == '\\' {
            "\\\\".to_string()
        } else if ch == '\'' {
            "\\'".to_string()
        } else {
            ch.to_string()
        };

        if glyph.tight_data.is_empty() {
            writeln!(f, "// '{}' ({}) — empty", display_ch, code).unwrap();
            writeln!(f, "#[rustfmt::skip]").unwrap();
            writeln!(f, "static GLYPH_{code}: &[u8] = &[];").unwrap();
        } else {
            writeln!(f, "// '{}' ({})", display_ch, code).unwrap();
            writeln!(f, "#[rustfmt::skip]").unwrap();
            write!(f, "static GLYPH_{code}: &[u8] = &[").unwrap();
            for (i, byte) in glyph.tight_data.iter().enumerate() {
                if i > 0 {
                    write!(f, ", ").unwrap();
                }
                write!(f, "0x{:02X}", byte).unwrap();
            }
            writeln!(f, "];").unwrap();
        }
    }

    writeln!(f).unwrap();
    writeln!(f, "#[rustfmt::skip]").unwrap();
    writeln!(f, "static GLYPHS: [Glyph; 95] = [").unwrap();

    for code in 32u8..=126u8 {
        let glyph = &font.glyphs[&code];
        writeln!(
            f,
            "    Glyph {{ width: {}, height: {}, top: {}, bytes_per_row: {}, data: GLYPH_{} }},",
            glyph.tight_width, glyph.tight_height, glyph.tight_top, glyph.tight_bytes_per_row, code
        )
        .unwrap();
    }

    writeln!(f, "];").unwrap();
    writeln!(f).unwrap();

    writeln!(
        f,
        "pub static {const_name}: Font = Font::new({}, {}, &GLYPHS);",
        font.height, default_gap
    )
    .unwrap();
}
