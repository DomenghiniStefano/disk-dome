//! The map styles of `docs/design-system.md` § 4.4 and what each asks of the layout.

/// How file rectangles are drawn (setting «Map style»).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MapStyle {
    /// Radial gradient, the full colour up to 35 % of the radius, 32 % toward black at the corners.
    Vignette,
    /// Linear gradient at 135°: 10 % toward white, the full colour at 45 %, 22 % toward black.
    DiagonalLight,
    /// Flat colour, a 1 px inner border 35 % toward black.
    Mosaic,
    /// Flat colour, a 1 px right / bottom edge 18 % toward black; 3, 2, 1 px gaps between folders.
    Structure,
    /// Flat colour, a 1 px `bg` line on the right and bottom edges.
    Flat,
    /// van Wijk & van de Wetering cushions.
    Cushion,
    /// Flat colour, a light top-left edge and a dark bottom-right edge.
    Relief,
    /// Folders down to 4 levels framed with an 18 px header; files drawn flat.
    Folders,
}

impl MapStyle {
    /// Every style, in the order of the design system's table.
    pub const ALL: [MapStyle; 8] = [
        MapStyle::Vignette,
        MapStyle::DiagonalLight,
        MapStyle::Mosaic,
        MapStyle::Structure,
        MapStyle::Flat,
        MapStyle::Cushion,
        MapStyle::Relief,
        MapStyle::Folders,
    ];

    /// A short lower-case name, for files and logs (never shown to the user).
    pub const fn key(self) -> &'static str {
        match self {
            MapStyle::Vignette => "vignette",
            MapStyle::DiagonalLight => "diagonal-light",
            MapStyle::Mosaic => "mosaic",
            MapStyle::Structure => "structure",
            MapStyle::Flat => "flat",
            MapStyle::Cushion => "cushion",
            MapStyle::Relief => "relief",
            MapStyle::Folders => "folders",
        }
    }

    /// The layout this style is drawn on.
    pub const fn layout_options(self) -> LayoutOptions {
        match self {
            MapStyle::Structure => LayoutOptions {
                gaps: [3, 2, 1, 0],
                ..LayoutOptions::DEFAULT
            },
            MapStyle::Folders => LayoutOptions {
                gaps: [1, 1, 1, 1],
                frames: true,
                ..LayoutOptions::DEFAULT
            },
            MapStyle::Vignette
            | MapStyle::DiagonalLight
            | MapStyle::Mosaic
            | MapStyle::Flat
            | MapStyle::Cushion
            | MapStyle::Relief => LayoutOptions::DEFAULT,
        }
    }
}

/// What the layout needs to know besides the tree and the size.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LayoutOptions {
    /// `bg` gap in pixels between the children of a folder at depth 0 (the zoom root), 1, 2, 3;
    /// deeper folders have none.
    pub gaps: [u8; 4],
    /// «Folders» style: frame folders with a header (§ 4.4).
    pub frames: bool,
    /// Free space of the drive in bytes, shown as its own block next to the data; 0 hides it.
    /// Only meaningful when the zoom root is the drive's root.
    pub free_space: u64,
}

impl LayoutOptions {
    /// 1 px gaps between the children of the zoom root and of the next level.
    pub const DEFAULT: LayoutOptions = LayoutOptions {
        gaps: [1, 1, 0, 0],
        frames: false,
        free_space: 0,
    };

    /// Frame header height (§ 4.4 «Folders»).
    pub const HEADER: i32 = 18;
    /// Children of a framed folder sit this far inside its frame.
    pub const FRAME_INSET: i32 = 2;
    /// A folder gets a frame from this size up…
    pub const FRAME_MIN: (i32, i32) = (96, 60);
    /// …down to this many levels below the zoom root.
    pub const FRAME_LEVELS: u8 = 4;
}

impl Default for LayoutOptions {
    fn default() -> Self {
        Self::DEFAULT
    }
}
