# Audio engine as an independent crate on cpal

The audio system is a separate `crates/audio` library with no dependency on the `tiles` rendering engine. It uses `cpal` directly as its output backend rather than a higher-level library like `rodio` or `kira`.

## Why independent from tiles

The rendering engine is purely visual. Not every tiles app needs audio (the editor doesn't). Coupling audio into the engine would force the dependency on all consumers and conflate two unrelated concerns. Applications that want audio depend on both `tiles` and `audio` independently and wire them together in their own update loop.

## Why cpal over rodio/kira

The primary consumer is a sound tracker with a built-in synthesizer. A tracker needs sample-accurate timing and direct control over the output buffer — it generates audio procedurally per-callback rather than "playing files." Higher-level libraries abstract away the exact thing a tracker requires (filling buffers with precise per-sample synthesis). We'd fight the abstraction. `cpal` gives one callback ("fill this buffer with f32 samples") which is exactly what a synth/tracker needs. The general-purpose playback API (VoicePool) is built as a convenience layer on top.

## Considered alternatives

- **rodio** — handles decoding and mixing, but assumes a "play a sound" model that doesn't fit procedural synthesis. Would need to be bypassed for the tracker, making it dead weight.
- **kira** — game-oriented with clock-synced events, but its audio graph model conflicts with our pull-based mixer + lock-free command queue design. Higher-level than needed.
- **Audio integrated into tiles** — rejected because it violates the engine's single responsibility (rendering) and forces audio deps on non-audio projects.
