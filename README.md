# wgpu 2D renderer

A minimal, well-commented 2D batch renderer in Rust using wgpu 22 + winit 0.30.

## Quick start

```bash
cargo run
```

Press **Escape** or close the window to quit.

## What it demonstrates

- winit 0.30 `ApplicationHandler` event loop
- wgpu surface setup with correct error handling (Lost / Outdated / OOM)
- Pre-allocated `VERTEX | COPY_DST` buffer — uploaded via `queue.write_buffer` each frame
- Alpha blending (`BlendState::ALPHA_BLENDING`)
- Pixel-space → NDC coordinate helper (`px_rect`)
- Animated demo: gradient background bars, 5 bouncing squares, pulsing centre rect
- HSV colour utility for smooth hue animation

## Architecture

```
App (ApplicationHandler)
 └─ Renderer          — owns Device, Queue, Surface, pipeline, vertex buffer
 └─ Batch             — CPU Vec<Vertex> rebuilt each frame
      └─ push_quad()  — appends 6 vertices (2 triangles) per quad
```

## Extending it

### Add a texture / sprite

1. Create a `wgpu::Texture` + `TextureView` + `Sampler`.
2. Add a `BindGroupLayout` with a `texture_2d` + `sampler` entry.
3. Add UV coords to `Vertex` (`@location(2) uv: vec2<f32>`).
4. In the fragment shader: `textureSample(t, s, in.uv) * in.color`.
5. Call `pass.set_bind_group(0, &bind_group, &[])` before drawing.

### Batch by texture

Maintain a `Vec<(TextureId, Batch)>`. Flush each batch with its bind group,
resetting between texture switches.

### Add a camera / transform

Add a uniform buffer holding a `mat4x4<f32>` projection matrix:

```wgsl
@group(0) @binding(0) var<uniform> proj: mat4x4<f32>;
// in vs_main:
out.clip_position = proj * vec4<f32>(in.position, 0.0, 1.0);
```

Use `glam::Mat4::orthographic_rh(0.0, width, height, 0.0, -1.0, 1.0)` for
a pixel-space ortho projection.

### Depth sorting

For layered 2D, sort `push_quad` calls back-to-front before `flush()`, or
enable a depth buffer with `Depth32Float` and pass a `z` value per quad.

## Examples

```bash
cargo run --example fonts
cargo run --example fire
cargo run --example nbody
cargo run --example lighting
cargo run --example boids
cargo run --example particles
cargo run --example rotation
```

## Crate versions

| Crate      | Version |
|------------|---------|
| wgpu       | 22      |
| winit      | 0.30    |
| bytemuck   | 1       |
| pollster   | 0.4     |
