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

**Palette**:
An ordered, mutable list of colors that a **Tilesheet** uses. Every **Cell** in a **Tile** references a Palette index rather than a raw color. Modifying a Palette entry recolors all Cells using that index.
_Avoid_: Color set, swatch, theme

**Tile**:
A square grid of **Cells** that forms a reusable visual unit. Defined by a uniform size (e.g. 8x8, 16x16). One Tile contains NxN Cells, each referencing a **Palette** index.
_Avoid_: Sprite, stamp, block

**Tilesheet**:
An ordered collection of **Tiles**, all sharing the same dimensions and **Palette**. The export artifact of the editor and the import artifact of the engine.
_Avoid_: Spritesheet, atlas, tilemap

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

**Pattern**:
A fixed-length grid of rows × channels containing note events. Each cell holds: note, instrument, volume, and two effect commands. The atomic unit of composition in the tracker.
_Avoid_: Block, sequence, clip

**Order List**:
A sequence of **Pattern** indices defining song playback order. Allows reusing Patterns. The song-level structure.
_Avoid_: Playlist, arrangement, song list

**Instrument**:
A creative preset in the tracker combining one **Source** with **Modulators** and default parameters (volume, pan). One Instrument maps to one Source type. Layering is achieved by using multiple tracker channels.
_Avoid_: Patch, program, preset

**Pane**:
A rectangular screen-space region that contains UI elements. Positioned in viewport coordinates (origin top-left, Y-down). Owns layout state: cursor position, computed size, draw-order rank. A Pane may be movable (via title bar drag) or resizable. Identified by a user-provided string ID.
_Avoid_: Window (ambiguous with OS window), panel, dialog

**PaneContext**:
The user-owned object that orchestrates all Panes. Manages focus, draw order, input routing, and stores persistent pane state (position, size) across frames. Receives input via `feed_mouse`/`feed_key`, evaluates layout and interaction in `pre_update`, and emits Cells in draw-order via `render_all`.
_Avoid_: UI manager, GUI system, UI state

**Widget**:
A UI element placed inside a **Pane** via the layout cursor. Defined by a trait (`size` + `render`) and eagerly consumed into a positioned rectangle of **Cells**. Built-in Widgets: button, text, slider, checkbox, separator, spacer. User-extensible by implementing the Widget trait.
_Avoid_: Component, control, node

**Drawable**:
A trait that produces **Cells** via a visitor callback. The unified interface for submitting visual content to the renderer. **Cell**, **Text**, **Line**, **Fill**, and **Stroke** implement Drawable. The `.colored()` combinator wraps any Drawable to override cell color.
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

**Rect**:
An axis-aligned bounding box defined by position and size (all f32). Constructed from any corner or from two opposing corners. Provides accessors for edges and corners. Implements **Shape** for fill/stroke. Not directly **Drawable** — use `.fill()` or `.stroke()`.
_Avoid_: BoundingBox, AABB, bounds

**RoundedRect**:
A **Rect** with per-corner radii. Constructed via `Rect::rounded(r)`. Supports per-corner overrides (`.top_left(r)`, etc.) and a `.radius(r)` to set all. Implements **Shape**. Radii adjust with `offset`.
_Avoid_: RoundRect, pill, capsule

**Line**:
A segment between two endpoints with a configurable width. Implements **Drawable** directly (not **Shape**). Width expansion is centered; even widths bias one side. Rasterized via DDA walk with perpendicular thickening.
_Avoid_: Segment, stroke (ambiguous with Shape stroke)

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
