# Tiles

A 2D rendering engine for pixel-art-style graphics in Rust. Provides windowing, an orthographic camera, fixed-timestep updates, input handling, and a declarative UI system — simulation logic is entirely user-owned.

## Quick start

```bash
cargo run -p sandbox --example fire
```

Press **Escape** or close the window to quit.

## Workspace

| Crate | Purpose |
|-------|---------|
| `tiles` | Core engine — rendering, camera, input, App trait, Node-based UI |
| `tiles-macros` | Proc macros (`widget!`, `widget_fn`, `Builders` derive) |
| `sandbox` | Examples and test bed |
| `palette` | Palette generation/solving utilities |
| `editor` | Tilesheet editor |
| `beeps` | Audio engine (independent, cpal-based) |

## Architecture

```
tiles::run(app, config)
 └─ Event loop (winit 0.30 ApplicationHandler)
     └─ State        — camera, input, timing, cell buffers
     └─ Renderer     — wgpu 22, GPU instancing, two-pass depth
     └─ App trait    — user implements update/draw/ui/on_key/on_mouse
```

The engine renders **Cells** — uniform square instances positioned in world space. The user submits Cells each frame; the engine handles instancing, depth sorting, and transparency.

## App trait

```rust
use tiles::{App, State, Config, Cell, Color};

struct MyApp;

impl App for MyApp {
    fn update(&mut self, state: &mut State) {
        // Fixed-timestep simulation
    }

    fn draw(&mut self, state: &mut State) {
        state.draw_world(Cell::new(0.0, 0.0).color(Color::hex(0xFF0000)));
    }
}

fn main() {
    let config = Config::builder()
        .title("My App")
        .viewport(256, 256)
        .build();
    tiles::run(MyApp, config).unwrap();
}
```

## Examples

```bash
cargo run -p sandbox --example fire
cargo run -p sandbox --example particles
cargo run -p sandbox --example nbody
cargo run -p sandbox --example boids
cargo run -p sandbox --example lighting
cargo run -p sandbox --example fonts
cargo run -p sandbox --example shapes
cargo run -p sandbox --example rotation
cargo run -p sandbox --example element
cargo run -p sandbox --example ui
cargo run -p sandbox --example palette
```

## Dependencies

| Crate | Version |
|-------|---------|
| wgpu | 22 |
| winit | 0.30 |
| glam | 0.29 |
| bytemuck | 1 |
| pollster | 0.4 |
