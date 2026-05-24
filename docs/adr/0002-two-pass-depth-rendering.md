# Two-pass depth rendering for correct alpha blending

Z is a real spatial axis in world space. We use a two-pass rendering strategy: opaque Cells (alpha == 1.0) render first with depth test and depth write enabled in any order, then transparent Cells render second sorted back-to-front by Z with depth test on but depth write off.

This avoids forcing the user to manage draw order while producing correct transparency. The engine partitions and sorts automatically each frame. The alternative — user-managed submission order with no depth buffer — pushes complexity onto every user for a problem the engine can solve generically.
