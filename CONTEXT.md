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
