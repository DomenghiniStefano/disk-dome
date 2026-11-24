//! Treemap layout and rendering.
//!
//! Squarified layout over the data model, drawn by the CPU into a pixel buffer split across cores.
//! It is redrawn only on resize or zoom; rectangles under 1–2 px are merged. The interface shows the
//! buffer as an image (Slint `SharedPixelBuffer`) and draws the hover highlight on top.
//! See `docs/architecture.md`.
