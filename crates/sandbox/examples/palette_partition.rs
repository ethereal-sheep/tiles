use palette::solver::{
    Buckets, ColorSpace::Hsl, HueBuckets, HuePartition, LightnessBuckets, LightnessPartition,
    Partition,
};
use tiles::{
    App, Color, Config, Drawable, KeyCode, KeyEvent, KeyState, MouseEvent, Rect, Shape, State, Text,
};

const SWATCH_SIZE: u32 = 4;
const GAP: f32 = 1.0;
const MARGIN: f32 = 0.0;
const LIGHTNESS_BUCKETS: usize = 5;
const HUE_BUCKETS: usize = 6;

fn make_palette() -> Vec<Color> {
    vec![
        Color::hex(0x2e222f),
        Color::hex(0x3e3546),
        Color::hex(0x625565),
        Color::hex(0x966c6c),
        Color::hex(0xab947a),
        Color::hex(0x694f62),
        Color::hex(0x7f708a),
        Color::hex(0x9babb2),
        Color::hex(0xc7dcd0),
        Color::hex(0xffffff),
        Color::hex(0x6e2727),
        Color::hex(0xb33831),
        Color::hex(0xea4f36),
        Color::hex(0xf57d4a),
        Color::hex(0xae2334),
        Color::hex(0xe83b3b),
        Color::hex(0xfb6b1d),
        Color::hex(0xf79617),
        Color::hex(0xf9c22b),
        Color::hex(0x7a3045),
        Color::hex(0x9e4539),
        Color::hex(0xcd683d),
        Color::hex(0xe6904e),
        Color::hex(0xfbb954),
        Color::hex(0x4c3e24),
        Color::hex(0x676633),
        Color::hex(0xa2a947),
        Color::hex(0xd5e04b),
        Color::hex(0xfbff86),
        Color::hex(0x165a4c),
        Color::hex(0x239063),
        Color::hex(0x1ebc73),
        Color::hex(0x91db69),
        Color::hex(0xcddf6c),
        Color::hex(0x313638),
        Color::hex(0x374e4a),
        Color::hex(0x547e64),
        Color::hex(0x92a984),
        Color::hex(0xb2ba90),
        Color::hex(0x0b5e65),
        Color::hex(0x0b8a8f),
        Color::hex(0x0eaf9b),
        Color::hex(0x30e1b9),
        Color::hex(0x8ff8e2),
        Color::hex(0x323353),
        Color::hex(0x484a77),
        Color::hex(0x4d65b4),
        Color::hex(0x4d9be6),
        Color::hex(0x8fd3ff),
        Color::hex(0x45293f),
        Color::hex(0x6b3e75),
        Color::hex(0x905ea9),
        Color::hex(0xa884f3),
        Color::hex(0xeaaded),
        Color::hex(0x753c54),
        Color::hex(0xa24b6f),
        Color::hex(0xcf657f),
        Color::hex(0xed8099),
        Color::hex(0x831c5d),
        Color::hex(0xc32454),
        Color::hex(0xf04f78),
        Color::hex(0xf68181),
        Color::hex(0xfca790),
        Color::hex(0xfdcbb0),
    ]
}

fn combined_partition(palette: &[Color], hue_offset: f32) -> Vec<Vec<Vec<Color>>> {
    let hue_buckets: HueBuckets = HuePartition::new(HUE_BUCKETS)
        .color_space(Hsl)
        .chroma_threshold(0.03)
        .offset(hue_offset)
        .build(palette)
        .unwrap();

    let lightness_buckets: LightnessBuckets = LightnessPartition::new(LIGHTNESS_BUCKETS)
        .distribution(palette::solver::Distribution::Normal { sigma: 0.5 })
        .build(palette)
        .unwrap();

    let mut grid = Vec::new();

    let hue_sorted = hue_buckets.sort(palette);
    for h_bucket in &hue_sorted {
        let l_sorted = lightness_buckets.sort(h_bucket);
        grid.push(l_sorted);
    }

    let found = (|| {
        for c in palette.iter() {
            for v1 in grid.iter() {
                for v2 in v1 {
                    for vc in v2 {
                        if c == vc {
                            return true;
                        }
                    }
                }
            }
        }
        return false;
    })();

    assert!(found);

    grid
}

const HUE_STEP: f32 = 10.0;

struct PalettePartition {
    palette: Vec<Color>,
    hue_offset: f32,
    grid: Vec<Vec<Vec<Color>>>,
}

impl App for PalettePartition {
    fn init(&mut self, state: &mut State) {
        state.set_viewport_background(0.0, 0.0, 0.0, 1.0);
        state.set_window_background(0.12, 0.12, 0.15, 1.0);
        state.set_ambient_illumination(1.0);
    }

    fn update(&mut self, _state: &mut State) {}

    fn draw(&mut self, state: &mut State) {
        let offset_string = format!("{}", self.hue_offset);
        state.draw_screen(Text::new(&tiles::font::TINY5_4X5, offset_string));

        let step = SWATCH_SIZE as f32 + GAP;
        let mut y = MARGIN;

        for (j, row) in self.grid.iter().enumerate() {
            y = step * j as f32 + GAP * 4.0 + 100.0;
            for (col_group, cell) in row.iter().enumerate() {
                let x_base = MARGIN + col_group as f32 * (step * 12.0);

                for (i, &color) in cell.iter().enumerate() {
                    let x = x_base + i as f32 * step;
                    state.draw_screen(
                        Rect::from_top_left(x, y, SWATCH_SIZE, SWATCH_SIZE)
                            .fill()
                            .color(color),
                    );
                }
            }
        }
    }

    fn on_key(&mut self, state: &mut State, event: KeyEvent) {
        if event.state != KeyState::Pressed {
            return;
        }
        match event.key {
            KeyCode::Escape => state.quit = true,
            KeyCode::Right => {
                self.hue_offset = (self.hue_offset + HUE_STEP) % 360.0;
                self.grid = combined_partition(&self.palette, self.hue_offset);
            }
            KeyCode::Left => {
                self.hue_offset = (self.hue_offset - HUE_STEP + 360.0) % 360.0;
                self.grid = combined_partition(&self.palette, self.hue_offset);
            }
            _ => {}
        }
    }

    fn on_mouse(&mut self, _state: &mut State, _event: MouseEvent) {}
}

fn main() {
    let palette = make_palette();
    let hue_offset = 0.0;
    let grid = combined_partition(&palette, hue_offset);

    let config: Config = Config::builder()
        .title("Palette Partition")
        .width(512)
        .height(512)
        .viewport(256.0, 256.0)
        .no_file()
        .build();

    tiles::run(
        PalettePartition {
            palette,
            hue_offset,
            grid,
        },
        config,
    )
    .unwrap();
}
