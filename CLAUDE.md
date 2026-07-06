# CLAUDE.md

See `CONTEXT.md` for domain language and glossary. Use its terms exactly — avoid synonyms listed under _Avoid_.

When making a large, breaking, or new change (new public concept, renamed term, removed feature, new crate), update `CONTEXT.md` to reflect it. Move dead terms to the KIV section if planned, or delete entirely if abandoned.

## Build & Run

```bash
cargo build                              # build workspace
cargo test -p tiles                      # run tiles crate tests
cargo test -p tiles-macros               # run macro tests
cargo run -p sandbox --example fire      # run an example
cargo run -p sandbox --example ui        # UI system demo
```

## Workspace

| Crate | Status | Purpose |
|-------|--------|---------|
| `tiles` | active | Core engine: rendering, camera, input, App trait, Node UI |
| `tiles-macros` | active | Proc macros: `widget!`, `widget_fn`, `app_widget_impl`, `Builders` derive |
| `sandbox` | active | Examples and test bed |
| `palette` | active | Palette generation/solving |
| `beeps` | active (not in workspace members) | Audio engine, independent of tiles |
| `panes` | dormant | Legacy UI approach, replaced by Node system in tiles |
| `editor` | dormant | Tilesheet editor, not maintained |

## Architecture

- User implements `App` trait (init, update, draw, ui, on_key, on_mouse)
- `tiles::run(app, config)` starts the event loop
- Rendering is Cell-based: GPU instancing with two-pass depth (opaque then transparent)
- UI is a declarative Node tree: built each frame via `widget!` macro or constructors, three-pass layout, hit-tested and rendered as screen-space Cells
- Fixed timestep for `update()`, variable for `draw()`

## Viewport

Always use 256x256 viewport for examples unless stated otherwise.
