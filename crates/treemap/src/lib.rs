//! Treemap layout and rendering.
//!
//! Squarified layout over the data model, drawn by the CPU into a pixel buffer split across cores.
//! It is redrawn only on resize, zoom, rescan, theme, style or option change; rectangles under 2 px
//! are merged into their folder's remainder. The interface shows the buffer as an image (Slint
//! `SharedPixelBuffer`) and draws the hover, the selection, the folder headers' text and the names
//! on top. See `docs/architecture.md` and `docs/design-system.md` § 4.4.
//!
//! The pipeline, each step with reusable buffers (nothing allocated per rectangle per frame):
//!
//! 1. [`Layout::compute`] lays the files of a [`MapSource`] out for a [`MapStyle`].
//! 2. [`Renderer::render`] draws the layout into `u32` pixels.
//! 3. [`NamePlacer::place`] chooses which rectangles get a name, and how big.

mod geom;
mod layout;
mod names;
mod render;
mod source;
mod style;

pub use geom::Rect;
pub use layout::{Header, Layout, MapRect, RectKind};
pub use names::{Label, LabelMetrics, NamePlacer, NamesMode, Tier};
pub use render::{Palette, Renderer, Theme, rgba};
pub use source::{CoreSource, MapSource, SimpleTree};
pub use style::{LayoutOptions, MapStyle};

/// Number of file categories, the length of a [`Palette`] (`docs/design-system.md` § 3.2).
pub const CATEGORIES: usize = 12;
