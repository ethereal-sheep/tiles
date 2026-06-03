# Semi-immediate UI library (panes) as a separate crate

We need a UI system that draws using Cells in screen space. We chose a semi-immediate-mode design in a standalone `crates/panes/` crate, with a user-owned `PaneContext` that manages focus, draw order, and layout — rather than a retained widget tree, a fully immediate system, or embedding UI into the `tiles` engine itself.

## Considered Options

**Fully immediate (stateless ctx):** Every frame the user declares widgets and gets results — ctx holds nothing between frames. Rejected because panes need persistent state across frames: position after drag, focus, draw order, resize state. Somebody has to remember where a pane was moved to.

**Retained widget tree:** Build a tree of widget objects, mutate it, let the system diff and redraw. Rejected because it adds complexity (diffing, lifecycle events, tree manipulation API) that doesn't match the existing "push Cells each frame" philosophy of tiles.

**Embedded in tiles crate:** UI built directly into `State`. Rejected to keep tiles focused on rendering/timing/input. Panes is opt-in and depends on tiles's public API only.

**Library-managed pane state with hook system (React-style):** ctx stores per-element local state, keyed by call order or string IDs. Rejected because conditional show/hide of elements breaks call-order indexing, and string keys per element are verbose. Instead, all widget value state is user-owned — passed in by value, returned as `Option<T>`.

**Closure-scoped panes (drop guard via closure):** `ctx.pane(id, opts, |p| { ... })` to guarantee end-of-pane cleanup. Rejected because the closure captures `&mut self` while `self.ctx` is already mutably borrowed — the borrow checker prevents accessing app state inside the closure.

## Consequences

- `tiles` gains a `pre_update` lifecycle method on `App` (once-per-frame, runs before fixed-timestep update ticks). This is general-purpose, not pane-specific.
- `tiles`'s `MouseEvent` becomes a struct carrying both screen-space and world-space coordinates (plus delta on move). Breaking API change.
- Element identity is positional (pane ID + declaration index). Reordering elements between frames invalidates cached interaction state for that pane.
- Extensibility is via a `Widget` trait (`size` + `render`). Widgets are eagerly consumed into `(Rect, Vec<Cell>)` — no stored trait objects, no heap allocation per element.
- Pane draw order and focus are managed by ctx — the user calls `render_all(state)` once in `draw()` and gets correct back-to-front ordering.
- The ctx tracks minimal mechanical state (active element for mouse capture) but no widget value state.
