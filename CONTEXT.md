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
