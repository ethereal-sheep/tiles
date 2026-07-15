# Tiles

A 2D rendering engine for pixel-art-style graphics, primarily used for particle simulation. Provides windowing, an orthographic camera, timing, and input — simulation logic is entirely user-owned.

## Language

**Cell**:
The atomic rendering primitive. A square drawn at a continuous position in world space. Always uniform in size — one Cell is exactly one world unit wide and tall.
_Avoid_: Tile, pixel, particle, quad

**World Space**:
The 3D coordinate system the simulation operates in. X-right, Y-up, Z-towards-camera (right-handed). One unit in world space equals one **Cell** width. Origin is (0, 0, 0).
_Avoid_: Screen space, pixel space, NDC

**Viewport**:
The rectangular region of **World Space** visible to the camera, defined in world units (e.g. 128x128). The engine fits the Viewport inside the window with uniform scaling and letterboxing. Determines how large **Cells** appear on screen.
_Avoid_: View, screen, canvas

**Window Background**:
The color rendered in letterbox/pillarbox margins outside the **Viewport**.
_Avoid_: Margin color, border color

**Viewport Background**:
The color used to clear the area inside the **Viewport** each frame — the visible "empty world."
_Avoid_: Clear color, background (ambiguous)


**Fixed Timestep**:
The simulation update rate. The engine calls the user's update function at a constant interval regardless of frame rate, accumulating leftover time between frames. The user always sees a constant `dt`.
_Avoid_: Frame time, variable dt

**Alpha (interpolation)**:
A value between 0.0 and 1.0 representing how far between the last two **Fixed Timestep** updates the current render frame falls. Exposed to the user for smooth rendering via interpolation.
_Avoid_: Lerp factor, blend

**Voice**:
A single active instance of a **Source** playing through the **Mixer**. Created on note-on or play, reclaimed when the Source signals completion or after note-off release finishes. One note trigger produces one Voice.
_Avoid_: Channel (ambiguous with tracker channels and stereo channels), instance

**Source**:
Something that produces audio samples on demand (pull-based). A Source is mono or stereo and returns a status (playing or finished) each render call. Types: oscillator, wavetable player, sample player.
_Avoid_: Generator, sound, clip

**Bus**:
A stereo submix group. Voices route to a Bus; each Bus has its own gain and **Effect** chain. All Buses mix into the master output. One level of hierarchy — Buses do not nest.
_Avoid_: Group, track, submix (when used as a verb)

**Effect**:
A processor that transforms a stereo audio buffer in-place. Attaches to a **Bus** or the master output. Stateful, operates per-buffer-callback.
_Avoid_: Plugin, insert, processor

**Modulator**:
A control signal generator attached to a **Voice**. Produces per-sample values that modify a target parameter (amplitude, pitch, or pan). Types: ADSR envelope, LFO.
_Avoid_: Envelope (when referring to the concept generically), controller


**Node**:
The fundamental unit of the declarative UI tree. Has a style (sizing, axis, gap, padding, justify, align, colors), optional event handlers, and content (either child Nodes or text). Built via `pane()`, `row()`, `col()`, `text()` constructors or the `widget!` macro. Resolved through a three-pass layout (pre-process → size → position) then evaluated for hit-testing and rendering. Rebuilt each frame; stable IDs (auto-generated or user-provided) maintain interaction continuity.
_Avoid_: Pane (legacy term), component, element (ambiguous with the world-space Element concept)

**Widget**:
A composable **Node** constructor. Defined by a trait (`render(self, children) -> Node<A>`). Built-in constructors: `pane()`, `row()`, `col()`, `text()`. User-extensible via `#[widget_fn]` macro which generates a function returning a Node with custom parameters and children.
_Avoid_: Component, control, element

**Element**:
A user-implemented interactive visual with a **Shape** and an appearance that varies by **ElementState**. Defined by a trait (`shape` + `draw`) with default methods (`handle_screen`, `handle_world`) that hit-test, compute visual state, draw to the overlay buffer, and return **HitState**. Lives in the tiles crate.
_Avoid_: Widget (different concept), Node, component, control

**ElementState**:
The visual state of an **Element**: Default, Hovered, Pressed, or Captured. Derived from **HitState** by the Element trait's default methods. Captured means the press originated inside but the cursor has since left the **Shape**.
_Avoid_: InputState (internal engine concept), interaction state

**HitState**:
The result of hit-testing a **Shape** against current input. Provides methods (`is_hovered`, `is_clicked`, `is_dragging`, etc.) over private fields. Constructed internally by `test_shape_screen` / `test_shape_world`. Replaces the former RectInputState.
_Avoid_: RectInputState, InputState, interaction result

**DragInfo**:
Data returned by `HitState::is_dragging()` when a drag is active. Contains delta and origin in both screen and world coordinates.
_Avoid_: DragState, drag data

**Drawable**:
A trait that produces **Cells** via a visitor callback. The unified interface for submitting visual content to the renderer. **Cell**, **Text**, **Frame**, **Line**, **Fill**, and **Stroke** implement Drawable. Combinators (`.color()`, `.map_cell()`, `.flip_x()`, `.translate()`, etc.) wrap any Drawable in a `Mapped<T>` adapter.
_Avoid_: Renderable, primitive

**Shape**:
A trait for closed geometry that can be filled or stroked. Requires `fill_cells`, `stroke_cells`, and `offset`. Provides `.fill()` → **Fill** and `.stroke(width, position)` → **Stroke** builders. **Rect** and **RoundedRect** implement Shape.
_Avoid_: Drawable (different concept), primitive

**Fill**:
A wrapper produced by `Shape::fill()` that implements **Drawable** by emitting all interior **Cells** of the wrapped **Shape**.
_Avoid_: Filled, solid

**Stroke**:
A wrapper produced by `Shape::stroke(width, position)` that implements **Drawable** by emitting boundary **Cells** of the wrapped **Shape**. Supports inner, outer, and middle positioning. For width > 1, emits layers via repeated `offset` + `stroke_cells`.
_Avoid_: Outline, border

**StrokePosition**:
Determines where stroke layers land relative to the **Shape** boundary: Inner (inward), Outer (outward), or Middle (straddles, even bias outward).
_Avoid_: Alignment, placement

**Text**:
A builder that produces **Cells** from a **Font** and a string. Holds position, anchor, color, and per-character mappings. Computes its bounding **Rect** eagerly. Ephemeral — built fresh each frame.
_Avoid_: Label, string renderer, text primitive

**Glyph**:
A per-character bitmap with pre-computed tight bounding dimensions (width and height of actual lit pixels). Stored in a **Font**'s static glyph array. Not directly drawable — consumed by **Text** internally.
_Avoid_: Character, letter, char data

**Image**:
A static resource holding a decoded PNG or JPEG (`Image::from_path`) as an RGBA pixel buffer. Not directly **Drawable** — call `.instance()` (or `.frame(n)`) to get a **Frame**. Meant to be loaded once and kept around; `.instance()`/`.frame(n)` are cheap to call repeatedly since the pixel buffer is shared, not re-decoded.
_Avoid_: Sprite, Texture, Bitmap (imply GPU texturing, which the engine does not have)

**Frame**:
A **Drawable** view over an **Image**'s pixel buffer, produced by `Image::instance()` or `Image::frame(n)`. Holds position, anchor, and its own width/height (decoupled from the source Image for future sprite-sheet support). Emits one **Cell** per non-transparent pixel — one source pixel is one Cell, no scaling. `frame(n)`'s index is currently ignored (multi-frame/sprite-sheet slicing not yet implemented); every Frame from a given Image is the same single frame. Cheap to `Clone` (shares its pixel buffer).
_Avoid_: Sprite, Texture, Bitmap

**Rect**:
An axis-aligned bounding box defined by position and size (all f32). Constructed from any corner or from two opposing corners. Provides accessors for edges and corners. Implements **Shape** for fill/stroke. Not directly **Drawable** — use `.fill()` or `.stroke()`.
_Avoid_: BoundingBox, AABB, bounds

**RoundedRect**:
A **Rect** with per-corner radii. Constructed via `Rect::rounded(r)`. Supports per-corner overrides (`.top_left(r)`, etc.) and a `.radius(r)` to set all. Implements **Shape**. Radii adjust with `offset`.
_Avoid_: RoundRect, pill, capsule

**Line**:
A segment between two endpoints with a configurable width. Implements **Drawable** directly (not **Shape**). Width expansion is centered; even widths bias one side. Rasterized via DDA walk with perpendicular thickening.
_Avoid_: Segment, stroke (ambiguous with Shape stroke)

**Rotation**:
A per-**Cell** transform applied by the vertex shader. Parameterized by a float t: Z(t) rotates in the screen plane (0→1 = 0→90°), FlipX/FlipY rotate around the X/Y axis (0→1 = 0→180°), DiagonalTL/DiagonalTR rotate around diagonal axes. Stored internally as a quaternion.
_Avoid_: Transform, orientation

**Emissive**:
A **Cell** property meaning it is unaffected by **Ambient Illumination** — always rendered at full brightness. A Cell with a positive light radius is automatically emissive. Calling `.emissive()` sets light radius to zero (self-lit but no area illumination).
_Avoid_: Lit, fullbright, unlit (opposite meaning)

**Ambient Illumination**:
A global scalar (0.0–1.0) that dims non-**Emissive** Cells. At 1.0, all Cells render at full color. At 0.0, only Emissive Cells are visible. Set per-frame by the user.
_Avoid_: Global light, brightness, exposure

## KIV (planned, not yet implemented)

**Palette** — An ordered, mutable list of colors that a Tilesheet uses. Every Cell in a Tile references a Palette index rather than a raw color. Modifying a Palette entry recolors all Cells using that index.

**Tile** — A square grid of Cells that forms a reusable visual unit (e.g. 8x8, 16x16). Each Cell references a Palette index.

**Tilesheet** — An ordered collection of Tiles, all sharing the same dimensions and Palette. The export artifact of the editor and the import artifact of the engine.

**Pattern** — A fixed-length grid of rows × channels containing note events (note, instrument, volume, effect commands). The atomic unit of composition in a tracker.

**Order List** — A sequence of Pattern indices defining song playback order. The song-level structure.

**Instrument** — A creative preset combining one Source with Modulators and default parameters (volume, pan). One Instrument maps to one Source type.

## Example dialogue

> **Dev:** If I spawn 10,000 Cells at random positions, what happens?
>
> **Domain expert:** The engine uploads them as instances to the GPU and draws them. If it exceeds the buffer capacity, it splits into multiple draw calls — you won't notice. Each Cell is one world unit, so with a 128x128 Viewport you'd see a 128x128 region of them.
>
> **Dev:** What if some Cells are transparent?
>
> **Domain expert:** The engine partitions Cells into opaque and transparent. Opaques render first with depth writes. Transparents render second, sorted back-to-front by Z, with depth test on but writes off. You just submit Cells — the engine handles ordering.
>
> **Dev:** And if I resize the window?
>
> **Domain expert:** The Viewport stays the same size in world units. The engine scales it uniformly to fit the new window. If the aspect ratios don't match, you get margins in the Window Background color.
