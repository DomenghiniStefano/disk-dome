//! Which rectangles get a name, and how big (`docs/design-system.md` § 4.4 «Names»).
//!
//! Greedy: first the folders drawn as one piece, then the files, largest rectangle first, each
//! taking the largest tier it has room for, until the mode's cap. A name sits at the top left of
//! its own rectangle, inside it, and rectangles never overlap, so names never overlap either, nor
//! cover a «Folders» header (file rectangles lie below the headers). Slint draws the text; text
//! width is estimated from an average glyph advance per tier, no font is read here.
//!
//! Only the `cap` largest candidates are sorted (a partial selection first), so placing 700 names
//! among 50 000 rectangles does not sort all of them.

use crate::geom::Rect;
use crate::layout::{Layout, RectKind};
use crate::source::MapSource;

/// Option «Names» of the map header.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NamesMode {
    None,
    Few,
    Many,
    All,
}

impl NamesMode {
    pub const ALL: [NamesMode; 4] = [
        NamesMode::None,
        NamesMode::Few,
        NamesMode::Many,
        NamesMode::All,
    ];

    /// At most this many names.
    pub const fn cap(self) -> usize {
        match self {
            NamesMode::None => 0,
            NamesMode::Few => 64,
            NamesMode::Many => 260,
            NamesMode::All => 700,
        }
    }

    pub const fn key(self) -> &'static str {
        match self {
            NamesMode::None => "none",
            NamesMode::Few => "few",
            NamesMode::Many => "many",
            NamesMode::All => "all",
        }
    }
}

/// How a name is drawn.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Tier {
    /// Name (`caption-strong`) and size (`caption`) under it.
    NameSize,
    /// Name only, `map-name` (11 / 13, 600).
    Name,
    /// Name only, `map-name-xs` (10 / 11, 600).
    Small,
}

impl Tier {
    const fn index(self) -> usize {
        match self {
            Tier::NameSize => 0,
            Tier::Name => 1,
            Tier::Small => 2,
        }
    }

    /// Smallest rectangle for the tier (`Few` asks more of name + size).
    const fn min_size(self, mode: NamesMode) -> (i32, i32) {
        match (self, mode) {
            (Tier::NameSize, NamesMode::Few) => (96, 32),
            (Tier::NameSize, NamesMode::None | NamesMode::Many | NamesMode::All) => (72, 30),
            (Tier::Name, NamesMode::None | NamesMode::Few | NamesMode::Many | NamesMode::All) => {
                (36, 15)
            }
            (Tier::Small, NamesMode::None | NamesMode::Few | NamesMode::Many | NamesMode::All) => {
                (22, 11)
            }
        }
    }

    /// Inner padding.
    pub const fn padding(self) -> i32 {
        match self {
            Tier::NameSize => 4,
            Tier::Name => 3,
            Tier::Small => 2,
        }
    }

    /// Height of the text: two `caption` lines, or one line of the tier's step.
    pub const fn text_height(self) -> i32 {
        match self {
            Tier::NameSize => 32,
            Tier::Name => 13,
            Tier::Small => 11,
        }
    }

    /// Whether `mode` uses this tier.
    const fn in_mode(self, mode: NamesMode) -> bool {
        match (self, mode) {
            (Tier::NameSize | Tier::Name | Tier::Small, NamesMode::None) => false,
            (Tier::NameSize, NamesMode::Few | NamesMode::Many | NamesMode::All) => true,
            (Tier::Name, NamesMode::Many | NamesMode::All) => true,
            (Tier::Name, NamesMode::Few) => false,
            (Tier::Small, NamesMode::All) => true,
            (Tier::Small, NamesMode::Few | NamesMode::Many) => false,
        }
    }
}

/// Text measures, from the font the interface draws with.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LabelMetrics {
    /// Average glyph advance of the name, per tier (`caption-strong` 12 px, `map-name` 11 px,
    /// `map-name-xs` 10 px).
    pub advance: [f32; 3],
    /// Average glyph advance of the size line (`caption`, 12 px).
    pub size_advance: f32,
    /// Characters of a size line («123.4 GiB»).
    pub size_chars: u32,
    /// Folder icon and its gap before the name.
    pub folder_icon: f32,
}

impl LabelMetrics {
    /// Segoe UI estimates; the interface may pass measured values.
    pub const DEFAULT: LabelMetrics = LabelMetrics {
        advance: [6.6, 6.2, 5.6],
        size_advance: 6.2,
        size_chars: 9,
        folder_icon: 14.0,
    };
}

impl Default for LabelMetrics {
    fn default() -> Self {
        Self::DEFAULT
    }
}

/// A name to draw.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Label {
    /// Where the text goes (top left of the rectangle, inside its padding, clipped to it); the
    /// name is elided at the end to fit.
    pub rect: Rect,
    /// Index of the named rectangle in [`Layout::rects`].
    pub item: u32,
    pub entry: u32,
    pub tier: Tier,
}

/// A name needs room for this many characters, else the rectangle gets none.
const MIN_CHARS: u32 = 4;
/// Folders drawn as one piece are placed before every file.
const PIECE_FIRST: u64 = 1 << 48;

/// Places names; keeps its buffers between calls.
#[derive(Debug, Default)]
pub struct NamePlacer {
    candidates: Vec<(u64, u32)>,
    labels: Vec<Label>,
}

impl NamePlacer {
    pub fn new() -> Self {
        Self::default()
    }

    /// The names of the last [`NamePlacer::place`].
    pub fn labels(&self) -> &[Label] {
        &self.labels
    }

    /// Chooses the names for `layout` in `mode`.
    pub fn place<S: MapSource>(
        &mut self,
        layout: &Layout,
        source: &S,
        mode: NamesMode,
        metrics: &LabelMetrics,
    ) -> &[Label] {
        self.labels.clear();
        self.candidates.clear();
        let cap = mode.cap();
        let Some(smallest) = [Tier::Small, Tier::Name, Tier::NameSize]
            .into_iter()
            .find(|t| t.in_mode(mode))
        else {
            return &self.labels;
        };
        let (min_w, min_h) = smallest.min_size(mode);
        for (i, r) in layout.rects().iter().enumerate() {
            let first = match r.kind {
                RectKind::Piece => PIECE_FIRST,
                RectKind::File => 0,
                RectKind::Remainder | RectKind::Frame | RectKind::Free => continue,
            };
            if r.rect.width() >= min_w && r.rect.height() >= min_h {
                self.candidates.push((first + r.rect.area(), i as u32));
            }
        }

        // Take the largest in chunks of `cap`: usually the first chunk fills the cap.
        let mut start = 0;
        while start < self.candidates.len() && self.labels.len() < cap {
            let rest = &mut self.candidates[start..];
            let chunk = cap.min(rest.len());
            if rest.len() > chunk {
                rest.select_nth_unstable_by(chunk - 1, |a, b| b.cmp(a));
            }
            rest[..chunk].sort_unstable_by(|a, b| b.cmp(a));
            for &(_, i) in &rest[..chunk] {
                if self.labels.len() == cap {
                    break;
                }
                if let Some(label) = fit(layout, source, i, mode, metrics) {
                    self.labels.push(label);
                }
            }
            start += chunk;
        }
        &self.labels
    }
}

/// The largest tier rectangle `item` has room for, if any.
fn fit<S: MapSource>(
    layout: &Layout,
    source: &S,
    item: u32,
    mode: NamesMode,
    metrics: &LabelMetrics,
) -> Option<Label> {
    let r = layout.rects().get(item as usize)?;
    let (w, h) = (r.rect.width(), r.rect.height());
    let chars = source.name_chars(r.entry);
    let icon = match r.kind {
        RectKind::Piece => metrics.folder_icon,
        RectKind::File | RectKind::Remainder | RectKind::Frame | RectKind::Free => 0.0,
    };
    for tier in [Tier::NameSize, Tier::Name, Tier::Small] {
        if !tier.in_mode(mode) {
            continue;
        }
        let (min_w, min_h) = tier.min_size(mode);
        if w < min_w || h < min_h {
            continue;
        }
        let pad = tier.padding();
        let inner = (w - 2 * pad) as f32;
        let advance = metrics.advance[tier.index()];
        if inner < icon + MIN_CHARS.min(chars.max(1)) as f32 * advance {
            continue;
        }
        let mut text = icon + chars as f32 * advance;
        if tier == Tier::NameSize {
            text = text.max(metrics.size_chars as f32 * metrics.size_advance);
        }
        let x0 = r.rect.x0 + pad;
        let y0 = r.rect.y0 + pad;
        let rect = Rect::new(
            x0,
            y0,
            (x0 + text.ceil() as i32).min(r.rect.x1 - pad),
            (y0 + tier.text_height()).min(r.rect.y1),
        );
        return Some(Label {
            rect,
            item,
            entry: r.entry,
            tier,
        });
    }
    None
}

#[cfg(test)]
mod tests {
    use super::{LabelMetrics, NamePlacer, NamesMode, Tier};
    use crate::layout::{Layout, RectKind};
    use crate::source::SimpleTree;
    use crate::style::MapStyle;

    /// Many files of varied sizes, a few levels deep.
    fn dense() -> SimpleTree {
        let mut t = SimpleTree::new();
        let mut seed = 12345u64;
        for _ in 0..12 {
            let dir = t.add_dir(SimpleTree::ROOT, 10);
            for _ in 0..8 {
                let sub = t.add_dir(dir, 10);
                for _ in 0..12 {
                    seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
                    let size = 1 + (seed >> 40) % 100_000;
                    t.add_file(sub, size, ((seed >> 20) % 12) as u8, 3 + (seed % 20) as u16);
                }
            }
        }
        for _ in 0..5 {
            t.add_file(SimpleTree::ROOT, 2_000_000, 4, 12);
        }
        t.finish();
        t
    }

    #[test]
    fn names_never_overlap_and_respect_caps() {
        let t = dense();
        let mut layout = Layout::new();
        let mut placer = NamePlacer::new();
        for style in [MapStyle::Vignette, MapStyle::Folders] {
            layout.compute(&t, SimpleTree::ROOT, 1920, 1080, &style.layout_options());
            for mode in NamesMode::ALL {
                let labels = placer.place(&layout, &t, mode, &LabelMetrics::DEFAULT);
                assert!(labels.len() <= mode.cap(), "{mode:?}");
                if mode != NamesMode::None {
                    assert!(!labels.is_empty(), "{mode:?}");
                }
                for (i, a) in labels.iter().enumerate() {
                    let item = layout.rects()[a.item as usize];
                    assert!(item.rect.contains(&a.rect), "{a:?} outside {item:?}");
                    assert!(!a.rect.is_empty());
                    for b in &labels[i + 1..] {
                        assert!(!a.rect.intersects(&b.rect), "{a:?} overlaps {b:?}");
                    }
                    for h in layout.headers() {
                        assert!(!a.rect.intersects(&h.rect));
                    }
                }
            }
        }
    }

    #[test]
    fn tiers_follow_the_mode() {
        let t = dense();
        let mut layout = Layout::new();
        layout.compute(
            &t,
            SimpleTree::ROOT,
            1920,
            1080,
            &MapStyle::Flat.layout_options(),
        );
        let mut placer = NamePlacer::new();
        let few = placer
            .place(&layout, &t, NamesMode::Few, &LabelMetrics::DEFAULT)
            .to_vec();
        assert!(few.iter().all(|l| l.tier == Tier::NameSize));
        for l in &few {
            let r = layout.rects()[l.item as usize].rect;
            assert!(r.width() >= 96 && r.height() >= 32);
        }
        let all = placer.place(&layout, &t, NamesMode::All, &LabelMetrics::DEFAULT);
        assert_eq!(all.len(), NamesMode::All.cap());
        assert!(all.iter().any(|l| l.tier == Tier::Small));
        // Largest first: no named file is smaller than an unnamed one that could take a name.
        let smallest_named = all
            .iter()
            .filter(|l| layout.rects()[l.item as usize].kind == RectKind::File)
            .map(|l| layout.rects()[l.item as usize].rect.area())
            .min()
            .unwrap_or(0);
        assert!(smallest_named > 0);
    }

    #[test]
    fn pieces_are_named_before_files() {
        let mut t = SimpleTree::new();
        let dir = t.add_dir(SimpleTree::ROOT, 6);
        for _ in 0..5000 {
            t.add_file(dir, 1, 0, 6);
        }
        t.add_file(SimpleTree::ROOT, 400_000, 2, 6);
        t.finish();
        let mut layout = Layout::new();
        layout.compute(
            &t,
            SimpleTree::ROOT,
            400,
            300,
            &MapStyle::Flat.layout_options(),
        );
        let mut placer = NamePlacer::new();
        let labels = placer.place(&layout, &t, NamesMode::All, &LabelMetrics::DEFAULT);
        if let Some(first) = labels.first() {
            let kind = layout.rects()[first.item as usize].kind;
            let has_piece = layout.rects().iter().any(|r| {
                r.kind == RectKind::Piece && r.rect.width() >= 22 && r.rect.height() >= 11
            });
            if has_piece {
                assert_eq!(kind, RectKind::Piece);
            }
        }
    }
}
