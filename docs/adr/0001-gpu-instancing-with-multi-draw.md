# GPU instancing with fixed-buffer multi-draw

We render Cells via GPU instancing rather than CPU-expanded vertex batching. Each Cell is a single instance (position + color) and the vertex shader expands it into a unit quad. The instance buffer is a fixed 64K capacity; when the user submits more Cells than fit, the engine issues multiple instanced draw calls per frame rather than growing the buffer.

## Considered Options

- **CPU vertex batching** (expand 6 vertices per Cell on CPU, upload full vertex buffer). Simpler but CPU-bound at ~10K cells — inadequate for particle simulation workloads.
- **Dynamic buffer growth** (reallocate a larger GPU buffer when exceeded). Avoids multi-draw but introduces allocation stalls on spike frames and permanently holds high-water-mark memory.
- **Compute shader pipeline** (simulation + rendering on GPU). Maximum throughput but requires simulation logic in WGSL — incompatible with the design goal of user-owned Rust update loops.

Instancing with multi-draw gives predictable fixed memory, zero allocation stalls, and ~100K+ Cells at 60fps with room to grow. The migration path to compute shaders remains open for a future GPU simulation layer.
