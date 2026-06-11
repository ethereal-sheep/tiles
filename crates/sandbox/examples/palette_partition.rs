use palette::solver::{
    Buckets,
    ColorSpace::{Hsl, Oklch},
    HueBuckets, HuePartition, LightnessBuckets, LightnessPartition, Partition,
};
use tiles::{
    App, Color, Config, Drawable, KeyCode, KeyEvent, KeyState, Line, MouseEvent, Rect, Shape,
    State, Text,
};

const SWATCH_SIZE: u32 = 4;
const GAP: f32 = 3.0;
const MARGIN: f32 = 0.0;
const LIGHTNESS_BUCKETS: usize = 6;
const HUE_BUCKETS: usize = 9;

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
        .color_space(Oklch)
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

const HUE_STEP: f32 = 2.0;

struct DagNode {
    color: Color,
    x: f32,
    y: f32,
    hue_idx: usize,
}

struct DagEdge {
    from: usize,
    to: usize,
}

struct Dag {
    nodes: Vec<DagNode>,
    edges: Vec<DagEdge>,
    adjacency: Vec<Vec<usize>>,
}

type Path = Vec<usize>;

fn find_paths(dag: &Dag, edge_count: usize) -> Vec<Path> {
    let mut paths = Vec::new();
    let mut seen = std::collections::HashSet::new();
    let target_len = edge_count + 1;
    let num_nodes = dag.nodes.len();

    // Stack: (path, hue_mask, hue_transition_happened)
    for start in 0..num_nodes {
        let mut stack: Vec<(Vec<usize>, u64, bool)> =
            vec![(vec![start], 1u64 << dag.nodes[start].hue_idx, false)];

        while let Some((path, hue_mask, transitioned)) = stack.pop() {
            if path.len() == target_len {
                let color_key: Vec<[u32; 3]> = path
                    .iter()
                    .map(|&n| {
                        let c = &dag.nodes[n].color;
                        [c.r.to_bits(), c.g.to_bits(), c.b.to_bits()]
                    })
                    .collect();
                if seen.insert(color_key) {
                    paths.push(path);
                }
                continue;
            }

            let current = *path.last().unwrap();
            let current_hue = dag.nodes[current].hue_idx;
            let edge_idx = path.len() - 1;

            for &next in &dag.adjacency[current] {
                let next_hue = dag.nodes[next].hue_idx;
                let next_mask = hue_mask | (1u64 << next_hue);
                if next_mask.count_ones() > 2 {
                    continue;
                }

                let is_hue_change = next_hue != current_hue;
                if is_hue_change && transitioned {
                    // Already had a transition; only allow another at the last edge
                    if edge_idx != edge_count - 1 {
                        continue;
                    }
                }
                if is_hue_change && !transitioned {
                    // First transition: only allow at first or last edge
                    if edge_idx != 0 && edge_idx != edge_count - 1 {
                        continue;
                    }
                }

                let mut new_path = path.clone();
                new_path.push(next);
                stack.push((new_path, next_mask, transitioned || is_hue_change));
            }
        }
    }

    paths
}

const MAX_CHROMA_STEP: f32 = 0.5;

fn linear_to_srgb(l: f32) -> f32 {
    if l <= 0.0031308 {
        l * 12.92
    } else {
        1.055 * l.powf(1.0 / 2.4) - 0.055
    }
}

fn chroma(c: &Color) -> f32 {
    let r = linear_to_srgb(c.r);
    let g = linear_to_srgb(c.g);
    let b = linear_to_srgb(c.b);
    r.max(g).max(b) - r.min(g).min(b)
}

fn build_dag(grid: &[Vec<Vec<Color>>]) -> Dag {
    let mut nodes = Vec::new();
    let mut edges = Vec::new();

    let num_hue_families = grid.len();
    let num_lightness_levels = grid.first().map_or(0, |row| row.len());
    // Last bucket from HueBuckets::sort is achromatic
    let num_chromatic = num_hue_families.saturating_sub(1);

    let level_spacing_y = (SWATCH_SIZE as f32 + GAP) * 3.0;
    let family_spacing_x = (SWATCH_SIZE as f32 + GAP) * 4.5;

    // node_indices[hue_idx][lightness_idx] = range of node indices for that cell
    let mut node_indices: Vec<Vec<Vec<usize>>> =
        vec![vec![Vec::new(); num_lightness_levels]; num_hue_families];

    for (h_idx, hue_family) in grid.iter().enumerate() {
        for (l_idx, cell) in hue_family.iter().enumerate() {
            for (i, &color) in cell.iter().enumerate() {
                let x = h_idx as f32 * family_spacing_x + i as f32 * (SWATCH_SIZE as f32 + GAP);
                let y = (num_lightness_levels - 1 - l_idx) as f32 * level_spacing_y;
                let node_idx = nodes.len();
                node_indices[h_idx][l_idx].push(node_idx);
                nodes.push(DagNode {
                    color,
                    x,
                    y,
                    hue_idx: h_idx,
                });
            }
        }
    }

    // Edges for chromatic families only (excludes the last/achromatic bucket)
    for h_idx in 0..num_chromatic {
        for l_idx in 0..num_lightness_levels.saturating_sub(1) {
            let next_l = l_idx + 1;
            let adjacent_hues: Vec<usize> = if num_chromatic <= 1 {
                vec![h_idx]
            } else {
                let prev_h = if h_idx == 0 {
                    num_chromatic - 1
                } else {
                    h_idx - 1
                };
                let next_h = (h_idx + 1) % num_chromatic;
                vec![prev_h, h_idx, next_h]
            };

            for &from_node in &node_indices[h_idx][l_idx] {
                let from_chroma = chroma(&nodes[from_node].color);
                for &adj_h in &adjacent_hues {
                    for &to_node in &node_indices[adj_h][next_l] {
                        let to_chroma = chroma(&nodes[to_node].color);
                        if (from_chroma - to_chroma).abs() < MAX_CHROMA_STEP {
                            edges.push(DagEdge {
                                from: from_node,
                                to: to_node,
                            });
                        }
                    }
                }
            }
        }
    }

    // Achromatic bucket: only self-adjacent (no wrapping to chromatic families)
    if num_hue_families > 0 {
        let achromatic_idx = num_hue_families - 1;
        for l_idx in 0..num_lightness_levels.saturating_sub(1) {
            let next_l = l_idx + 1;
            for &from_node in &node_indices[achromatic_idx][l_idx] {
                for &to_node in &node_indices[achromatic_idx][next_l] {
                    edges.push(DagEdge {
                        from: from_node,
                        to: to_node,
                    });
                }
            }
        }
    }

    let mut adjacency = vec![Vec::new(); nodes.len()];
    for edge in &edges {
        adjacency[edge.from].push(edge.to);
    }

    Dag {
        nodes,
        edges,
        adjacency,
    }
}

const DEFAULT_PATH_LENGTH: usize = 4;

struct PalettePartition {
    palette: Vec<Color>,
    hue_offset: f32,
    path_length: usize,
    grid: Vec<Vec<Vec<Color>>>,
    dag: Dag,
    paths: Vec<Path>,
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

        let ox = MARGIN + 10.0;
        let oy = MARGIN + 10.0;
        let half = SWATCH_SIZE as f32 / 2.0;

        // Draw edges
        for edge in &self.dag.edges {
            let from = &self.dag.nodes[edge.from];
            let to = &self.dag.nodes[edge.to];
            if from.hue_idx.min(to.hue_idx) == 0
                && from.hue_idx.max(to.hue_idx) == (HUE_BUCKETS - 1)
            {
                state.draw_screen(
                    Line::new(
                        ox + from.x + half,
                        oy + from.y + half,
                        ox + from.x + half + ((from.x - to.x) / HUE_BUCKETS as f32),
                        oy + to.y + half,
                    )
                    .color(from.color),
                );
                state.draw_screen(
                    Line::new(
                        ox + to.x + half - ((from.x - to.x) / HUE_BUCKETS as f32),
                        oy + from.y + half,
                        ox + to.x + half,
                        oy + to.y + half,
                    )
                    .color(from.color),
                );
            } else {
                state.draw_screen(
                    Line::new(
                        ox + from.x + half,
                        oy + from.y + half,
                        ox + to.x + half,
                        oy + to.y + half,
                    )
                    .color(from.color),
                );
            }
        }

        // Draw nodes
        for node in &self.dag.nodes {
            state.draw_screen(
                Rect::from_top_left(ox + node.x, oy + node.y, SWATCH_SIZE, SWATCH_SIZE)
                    .fill()
                    .color(node.color),
            );
        }

        // Draw paths below the DAG
        let path_ox = MARGIN + 10.0;
        let path_oy = 160.0;
        let path_step = SWATCH_SIZE as f32 + GAP;
        let path_row_height = SWATCH_SIZE as f32 + GAP;
        // let max_path_rows = ((256.0 - path_oy) / path_row_height) as usize;

        for (row, path) in self.paths.iter().enumerate() {
            let y = path_oy + (row % 13) as f32 * path_row_height as f32;
            let x_offset =
                ((row as f32 / 13.0).floor()) * path_step as f32 * (self.path_length + 3) as f32;
            for (i, &node_idx) in path.iter().enumerate() {
                let x = path_ox + i as f32 * path_step + x_offset;
                let color = self.dag.nodes[node_idx].color;

                // if i > 0 {
                //     let prev_x = path_ox + (i - 1) as f32 * path_step + half * 2.0;
                //     state.draw_screen(Line::new(prev_x, y + half, x, y + half).color(color));
                // }

                state.draw_screen(
                    Rect::from_top_left(x, y, SWATCH_SIZE, SWATCH_SIZE)
                        .fill()
                        .color(color),
                );
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
                self.dag = build_dag(&self.grid);
                self.paths = find_paths(&self.dag, self.path_length);
            }
            KeyCode::Left => {
                self.hue_offset = (self.hue_offset - HUE_STEP + 360.0) % 360.0;
                self.grid = combined_partition(&self.palette, self.hue_offset);
                self.dag = build_dag(&self.grid);
                self.paths = find_paths(&self.dag, self.path_length);
            }
            KeyCode::Up => {
                self.path_length += 1;
                self.paths = find_paths(&self.dag, self.path_length);
            }
            KeyCode::Down => {
                if self.path_length > 1 {
                    self.path_length -= 1;
                    self.paths = find_paths(&self.dag, self.path_length);
                }
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
    let dag = build_dag(&grid);
    let path_length = DEFAULT_PATH_LENGTH;
    let paths = find_paths(&dag, path_length);

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
            path_length,
            grid,
            dag,
            paths,
        },
        config,
    )
    .unwrap();
}
