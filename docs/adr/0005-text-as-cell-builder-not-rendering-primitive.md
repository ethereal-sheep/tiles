# Text is a Cell builder, not a rendering primitive

Text produces Cells from font data — it does not introduce a new GPU-level concept. The renderer stays Cell-only; Text is a convenience type that expands into positioned, colored Cells via the Drawable trait.

The alternative was making Text a native rendering primitive with its own vertex layout or draw call (batch-submit glyph quads, GPU-side atlas lookup). That would improve throughput for large text volumes but adds pipeline complexity, a second instance format, and font atlas management. Since text in this engine is UI labels and debug overlays (hundreds of characters, not thousands), the Cell-expansion approach is fast enough and keeps the renderer simple — one instance type, one draw path.
