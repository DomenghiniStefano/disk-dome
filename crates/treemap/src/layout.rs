//! Squarified layout (Bruls, Huizing & van Wijk), files only.
//!
//! Folders are divided depth-first with an explicit stack (no recursion: real trees can be deeper
//! than the main thread's stack). Each folder's children are laid out largest first; once a row or
//! an item would be under 2 px, everything left in the folder becomes one *remainder* rectangle
//! coloured with the folder's largest category. Only the `area / 4` largest children of a folder
//! can be visible, so a folder with a million files sorts only those.

use diskdome_core::NONE;

use crate::geom::{FRect, Rect};
use crate::source::MapSource;
use crate::style::LayoutOptions;

/// Below this many pixels on a side a rectangle is merged into its folder's remainder.
const MIN_SIDE: f64 = 2.0;
/// Cushion ridge height at the zoom root and its factor per level (§ 4.4 «Cushion»).
const CUSHION_HEIGHT: f64 = 0.75;
const CUSHION_FALLOFF: f64 = 0.85;

/// What a [`MapRect`] stands for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RectKind {
    /// One file.
    File,
    /// A whole folder drawn as one piece: everything in it is under 2 px.
    Piece,
    /// The rest of a folder: its files too small to draw, merged.
    Remainder,
    /// «Folders» style: the frame and header behind a folder's children.
    Frame,
    /// The drive's free space (entry [`NONE`]).
    Free,
}

/// One rectangle of the layout, in drawing order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MapRect {
    pub rect: Rect,
    /// The file, or the folder for a piece, a remainder or a frame.
    pub entry: u32,
    /// Index into the palette: the file's category, or the folder's largest.
    pub category: u8,
    /// Levels below the zoom root (its children are at 1).
    pub depth: u8,
    pub kind: RectKind,
}

/// «Folders» style: the 18 px header of a framed folder, where Slint draws its name and size.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Header {
    pub rect: Rect,
    pub entry: u32,
    /// 1 for the zoom root's children, down to 4.
    pub level: u8,
}

/// A folder waiting to be divided.
#[derive(Debug, Clone, Copy)]
struct Pending {
    entry: u32,
    rect: FRect,
    depth: u8,
    /// Cushion surface `[x², y², x, y]` coefficients accumulated from the zoom root.
    surface: [f64; 4],
}

/// The layout of one map: its rectangles in drawing order (a frame before what it holds), plus the
/// reusable buffers that compute it.
#[derive(Debug, Default)]
pub struct Layout {
    width: i32,
    height: i32,
    rects: Vec<MapRect>,
    /// Per rectangle: cushion normal `[nx, ny]` at its first pixel and their steps per pixel.
    cushion: Vec<[f32; 4]>,
    headers: Vec<Header>,
    free: Option<Rect>,
    stack: Vec<Pending>,
    kids: Vec<(u64, u32)>,
    placed: Vec<(FRect, u32)>,
    remainders: Vec<FRect>,
}

impl Layout {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn width(&self) -> i32 {
        self.width
    }

    pub fn height(&self) -> i32 {
        self.height
    }

    /// Every rectangle, in drawing order.
    pub fn rects(&self) -> &[MapRect] {
        &self.rects
    }

    /// «Folders» style headers, in drawing order.
    pub fn headers(&self) -> &[Header] {
        &self.headers
    }

    /// The free-space block, when shown.
    pub fn free(&self) -> Option<Rect> {
        self.free
    }

    pub(crate) fn cushion(&self) -> &[[f32; 4]] {
        &self.cushion
    }

    /// Lays out `root` (the zoom root) over `width` × `height` pixels. Reuses the buffers of the
    /// previous call.
    pub fn compute<S: MapSource>(
        &mut self,
        source: &S,
        root: u32,
        width: u32,
        height: u32,
        options: &LayoutOptions,
    ) {
        self.width = i32::try_from(width).unwrap_or(i32::MAX);
        self.height = i32::try_from(height).unwrap_or(i32::MAX);
        self.rects.clear();
        self.cushion.clear();
        self.headers.clear();
        self.free = None;
        self.stack.clear();
        let used = source.size(root);
        if self.width == 0 || self.height == 0 || used == 0 {
            return;
        }
        let full = FRect {
            x0: 0.0,
            y0: 0.0,
            x1: f64::from(self.width),
            y1: f64::from(self.height),
        };
        let data = if options.free_space > 0 {
            self.split_free(full, used, options.free_space)
        } else {
            full
        };
        let mut surface = [0.0; 4];
        add_ridge(&mut surface, data, CUSHION_HEIGHT);
        if source.is_dir(root) {
            self.stack.push(Pending {
                entry: root,
                rect: data,
                depth: 0,
                surface,
            });
        } else {
            self.push(
                data.snap(),
                root,
                source.category(root),
                0,
                RectKind::File,
                &surface,
            );
        }
        while let Some(folder) = self.stack.pop() {
            self.lay_folder(source, folder, options);
        }
    }

    /// Splits the map along its longer side into the data and the free space, a 1 px gap between.
    fn split_free(&mut self, full: FRect, used: u64, free: u64) -> FRect {
        let share = used as f64 / (used as f64 + free as f64);
        let mut data = full;
        let mut rest = full;
        if full.width() >= full.height() {
            let cut = (full.x0 + full.width() * share).round();
            data.x1 = cut;
            rest.x0 = (cut + 1.0).min(full.x1);
        } else {
            let cut = (full.y0 + full.height() * share).round();
            data.y1 = cut;
            rest.y0 = (cut + 1.0).min(full.y1);
        }
        let free_rect = rest.snap();
        if !free_rect.is_empty() {
            self.free = Some(free_rect);
            self.push(free_rect, NONE, 0, 0, RectKind::Free, &[0.0; 4]);
        }
        data
    }

    fn lay_folder<S: MapSource>(&mut self, source: &S, folder: Pending, options: &LayoutOptions) {
        let category = source.category(folder.entry);
        let child_depth = folder.depth.saturating_add(1);
        let area = folder.rect.width() * folder.rect.height();

        self.kids.clear();
        let mut total = 0u64;
        for &child in source.children(folder.entry) {
            let size = source.size(child);
            if size > 0 {
                self.kids.push((size, child));
                total += size;
            }
        }
        if total == 0 {
            let surface = folder.surface;
            self.push(
                folder.rect.snap(),
                folder.entry,
                category,
                folder.depth,
                RectKind::Piece,
                &surface,
            );
            return;
        }
        // Only children of at least MIN_SIDE² pixels can show: at most area / 4 of them.
        let cap = (area / (MIN_SIDE * MIN_SIDE)) as usize + 1;
        if self.kids.len() > cap {
            self.kids.select_nth_unstable_by(cap, |a, b| b.0.cmp(&a.0));
            self.kids.truncate(cap);
        }
        self.kids.sort_unstable_by_key(|a| std::cmp::Reverse(a.0));
        let kept: u64 = self.kids.iter().map(|k| k.0).sum();

        self.placed.clear();
        self.remainders.clear();
        squarify(
            &self.kids,
            area / total as f64,
            folder.rect,
            kept == total,
            &mut self.placed,
            &mut self.remainders,
        );

        let parent = folder.rect.snap();
        let gap = options
            .gaps
            .get(usize::from(folder.depth))
            .copied()
            .map_or(0, i32::from);
        let height = CUSHION_HEIGHT * CUSHION_FALLOFF.powi(i32::from(child_depth));

        for k in 0..self.placed.len() {
            let (frect, child) = self.placed[k];
            let mut rect = frect.snap();
            if gap > 0 {
                if rect.x1 < parent.x1 && rect.width() > gap {
                    rect.x1 -= gap;
                }
                if rect.y1 < parent.y1 && rect.height() > gap {
                    rect.y1 -= gap;
                }
            }
            let mut surface = folder.surface;
            add_ridge(&mut surface, frect, height);
            if !source.is_dir(child) {
                self.push(
                    rect,
                    child,
                    source.category(child),
                    child_depth,
                    RectKind::File,
                    &surface,
                );
            } else if f64::from(rect.width()) < MIN_SIDE || f64::from(rect.height()) < MIN_SIDE {
                self.push(
                    rect,
                    child,
                    source.category(child),
                    child_depth,
                    RectKind::Piece,
                    &surface,
                );
            } else {
                let framed = options.frames
                    && child_depth <= LayoutOptions::FRAME_LEVELS
                    && rect.width() >= LayoutOptions::FRAME_MIN.0
                    && rect.height() >= LayoutOptions::FRAME_MIN.1;
                let inner = if framed {
                    self.push(
                        rect,
                        child,
                        source.category(child),
                        child_depth,
                        RectKind::Frame,
                        &surface,
                    );
                    self.headers.push(Header {
                        rect: Rect::new(rect.x0, rect.y0, rect.x1, rect.y0 + LayoutOptions::HEADER),
                        entry: child,
                        level: child_depth,
                    });
                    FRect::from_rect(Rect::new(
                        rect.x0 + LayoutOptions::FRAME_INSET,
                        rect.y0 + LayoutOptions::HEADER,
                        rect.x1 - LayoutOptions::FRAME_INSET,
                        rect.y1 - LayoutOptions::FRAME_INSET,
                    ))
                } else if gap > 0 {
                    FRect::from_rect(rect)
                } else {
                    frect
                };
                self.stack.push(Pending {
                    entry: child,
                    rect: inner,
                    depth: child_depth,
                    surface,
                });
            }
        }

        // Nothing placed: the whole folder is one piece.
        let kind = if self.placed.is_empty() {
            RectKind::Piece
        } else {
            RectKind::Remainder
        };
        for k in 0..self.remainders.len() {
            let frect = self.remainders[k];
            let mut surface = folder.surface;
            add_ridge(&mut surface, frect, height);
            let depth = if self.placed.is_empty() {
                folder.depth
            } else {
                child_depth
            };
            self.push(frect.snap(), folder.entry, category, depth, kind, &surface);
        }
    }

    fn push(
        &mut self,
        rect: Rect,
        entry: u32,
        category: u8,
        depth: u8,
        kind: RectKind,
        surface: &[f64; 4],
    ) {
        if rect.is_empty() {
            return;
        }
        self.rects.push(MapRect {
            rect,
            entry,
            category,
            depth,
            kind,
        });
        // Surface z = a·x² + b·y² + c·x + d·y; the normal is (−∂z/∂x, −∂z/∂y, 1), taken at
        // pixel centres and stored relative to the rectangle (f32 keeps its precision there).
        let cx = f64::from(rect.x0) + 0.5;
        let cy = f64::from(rect.y0) + 0.5;
        let nx = -(2.0 * surface[0] * cx + surface[2]);
        let ny = -(2.0 * surface[1] * cy + surface[3]);
        self.cushion.push([
            nx as f32,
            ny as f32,
            (-2.0 * surface[0]) as f32,
            (-2.0 * surface[1]) as f32,
        ]);
    }
}

/// Adds a parabolic ridge of height `h` over `r` to a cushion surface (van Wijk & van de Wetering).
fn add_ridge(surface: &mut [f64; 4], r: FRect, h: f64) {
    let w = r.width();
    if w > 0.0 {
        let f = 4.0 * h / w;
        surface[2] += f * (r.x1 + r.x0);
        surface[0] -= f;
    }
    let ht = r.height();
    if ht > 0.0 {
        let f = 4.0 * h / ht;
        surface[3] += f * (r.y1 + r.y0);
        surface[1] -= f;
    }
}

/// Worst aspect ratio of a row with largest `max`, smallest `min`, total `sum` along `side`.
fn worst(max: f64, min: f64, sum: f64, side: f64) -> f64 {
    let s2 = side * side;
    let sum2 = sum * sum;
    (s2 * max / sum2).max(sum2 / (s2 * min))
}

/// Squarifies `kids` (largest first, sizes × `scale` = pixels) into `rect`. `exact` says the kids
/// are all the folder holds, so the last row fills the rectangle; otherwise what is left after
/// them is a remainder. Rows or items under [`MIN_SIDE`] end the folder in a remainder.
fn squarify(
    kids: &[(u64, u32)],
    scale: f64,
    rect: FRect,
    exact: bool,
    placed: &mut Vec<(FRect, u32)>,
    remainders: &mut Vec<FRect>,
) {
    let mut free = rect;
    let mut i = 0;
    let n = kids.len();
    while i < n {
        let (w, h) = (free.width(), free.height());
        if w < MIN_SIDE || h < MIN_SIDE {
            remainders.push(free);
            return;
        }
        let side = w.min(h);
        let start = i;
        let largest = kids[start].0 as f64 * scale;
        let mut row = 0.0;
        let mut row_worst = f64::INFINITY;
        while i < n {
            let s = kids[i].0 as f64 * scale;
            let next = worst(largest, s, row + s, side);
            if i > start && next > row_worst {
                break;
            }
            row += s;
            row_worst = next;
            i += 1;
        }
        let last = i == n && exact;
        let vertical = w >= h; // a column at the left, items stacked downwards
        let extent = if vertical { w } else { h };
        let thickness = if last { extent } else { row / side };
        if thickness < MIN_SIDE {
            remainders.push(free);
            return;
        }
        let mut tiny = false;
        if vertical {
            let x1 = if last { free.x1 } else { free.x0 + thickness };
            let mut y = free.y0;
            for (k, kid) in kids.iter().enumerate().take(i).skip(start) {
                let y1 = if k + 1 == i {
                    free.y1
                } else {
                    y + kid.0 as f64 * scale / thickness
                };
                if y1 - y < MIN_SIDE {
                    remainders.push(FRect {
                        x0: free.x0,
                        y0: y,
                        x1,
                        y1: free.y1,
                    });
                    tiny = true;
                    break;
                }
                placed.push((
                    FRect {
                        x0: free.x0,
                        y0: y,
                        x1,
                        y1,
                    },
                    kid.1,
                ));
                y = y1;
            }
            free.x0 = x1;
        } else {
            let y1 = if last { free.y1 } else { free.y0 + thickness };
            let mut x = free.x0;
            for (k, kid) in kids.iter().enumerate().take(i).skip(start) {
                let x1 = if k + 1 == i {
                    free.x1
                } else {
                    x + kid.0 as f64 * scale / thickness
                };
                if x1 - x < MIN_SIDE {
                    remainders.push(FRect {
                        x0: x,
                        y0: free.y0,
                        x1: free.x1,
                        y1,
                    });
                    tiny = true;
                    break;
                }
                placed.push((
                    FRect {
                        x0: x,
                        y0: free.y0,
                        x1,
                        y1,
                    },
                    kid.1,
                ));
                x = x1;
            }
            free.y0 = y1;
        }
        if tiny {
            break;
        }
    }
    // After an exact layout the last row consumed the rectangle and nothing is left.
    if free.width() >= 0.5 && free.height() >= 0.5 {
        remainders.push(free);
    }
}

#[cfg(test)]
mod tests {
    use super::{Layout, RectKind};
    use crate::source::{MapSource, SimpleTree};
    use crate::style::{LayoutOptions, MapStyle};

    fn no_gaps() -> LayoutOptions {
        LayoutOptions {
            gaps: [0; 4],
            ..LayoutOptions::DEFAULT
        }
    }

    /// Two levels of folders, files of varied sizes, none too small to show at 800 × 600.
    fn sample() -> SimpleTree {
        let mut t = SimpleTree::new();
        for d in 0..6u64 {
            let dir = t.add_dir(SimpleTree::ROOT, 8);
            for s in 0..5u64 {
                let sub = t.add_dir(dir, 8);
                for f in 0..8u64 {
                    t.add_file(
                        sub,
                        1000 + (d * 37 + s * 11 + f * 101) % 900,
                        (f % 12) as u8,
                        8,
                    );
                }
            }
            t.add_file(dir, 5000 + d * 300, 3, 8);
        }
        t.finish();
        t
    }

    fn assert_disjoint(layout: &Layout) {
        let rects: Vec<_> = layout
            .rects()
            .iter()
            .filter(|r| r.kind != RectKind::Frame)
            .collect();
        for (i, a) in rects.iter().enumerate() {
            for b in &rects[i + 1..] {
                assert!(
                    !a.rect.intersects(&b.rect),
                    "{:?} overlaps {:?}",
                    a.rect,
                    b.rect
                );
            }
        }
    }

    #[test]
    fn areas_sum_to_the_whole_map_without_gaps() {
        let t = sample();
        let mut layout = Layout::new();
        layout.compute(&t, SimpleTree::ROOT, 800, 600, &no_gaps());
        let covered: u64 = layout.rects().iter().map(|r| r.rect.area()).sum();
        assert_eq!(covered, 800 * 600);
        assert_disjoint(&layout);
    }

    #[test]
    fn areas_follow_sizes_within_rounding() {
        let t = sample();
        let mut layout = Layout::new();
        layout.compute(&t, SimpleTree::ROOT, 800, 600, &no_gaps());
        let per_byte = (800.0 * 600.0) / t.size(SimpleTree::ROOT) as f64;
        for r in layout.rects() {
            assert_eq!(r.kind, RectKind::File);
            let expected = t.size(r.entry) as f64 * per_byte;
            let (w, h) = (f64::from(r.rect.width()), f64::from(r.rect.height()));
            // Each edge may move by half a pixel when snapped.
            let slack = w + h + 1.0;
            assert!(
                (r.rect.area() as f64 - expected).abs() <= slack,
                "entry {} area {} expected {expected}",
                r.entry,
                r.rect.area()
            );
        }
    }

    #[test]
    fn gaps_keep_rectangles_disjoint_and_inside() {
        let t = sample();
        let mut layout = Layout::new();
        for style in MapStyle::ALL {
            layout.compute(&t, SimpleTree::ROOT, 800, 600, &style.layout_options());
            assert_disjoint(&layout);
            for r in layout.rects() {
                assert!(r.rect.x0 >= 0 && r.rect.y0 >= 0 && r.rect.x1 <= 800 && r.rect.y1 <= 600);
            }
        }
    }

    #[test]
    fn tiny_files_are_merged_into_the_remainder() {
        let mut t = SimpleTree::new();
        let dir = t.add_dir(SimpleTree::ROOT, 3);
        t.add_file(dir, 1_000_000, 1, 3);
        for _ in 0..10_000 {
            t.add_file(dir, 10, 2, 3);
        }
        t.add_file(SimpleTree::ROOT, 1_000_000, 4, 3);
        t.finish();
        let mut layout = Layout::new();
        layout.compute(&t, SimpleTree::ROOT, 200, 100, &no_gaps());
        let files = layout
            .rects()
            .iter()
            .filter(|r| r.kind == RectKind::File)
            .count();
        let remainders: Vec<_> = layout
            .rects()
            .iter()
            .filter(|r| r.kind == RectKind::Remainder)
            .collect();
        assert_eq!(files, 2);
        assert!(!remainders.is_empty());
        assert!(remainders.iter().all(|r| r.entry == dir && r.category == 1));
        for r in layout.rects() {
            assert!(r.rect.width() >= 1 && r.rect.height() >= 1);
            if r.kind == RectKind::File {
                assert!(r.rect.width() >= 2 && r.rect.height() >= 2);
            }
        }
        let covered: u64 = layout.rects().iter().map(|r| r.rect.area()).sum();
        assert_eq!(covered, 200 * 100);
    }

    #[test]
    fn a_folder_too_small_is_one_piece() {
        let mut t = SimpleTree::new();
        let dir = t.add_dir(SimpleTree::ROOT, 3);
        for _ in 0..5000 {
            t.add_file(dir, 1, 0, 3);
        }
        t.add_file(SimpleTree::ROOT, 100_000, 4, 3);
        t.finish();
        let mut layout = Layout::new();
        layout.compute(&t, SimpleTree::ROOT, 300, 200, &no_gaps());
        assert!(
            layout
                .rects()
                .iter()
                .any(|r| r.kind == RectKind::Piece && r.entry == dir)
        );
    }

    #[test]
    fn free_space_takes_its_share() {
        let t = sample();
        let mut layout = Layout::new();
        let used = t.size(SimpleTree::ROOT);
        let options = LayoutOptions {
            free_space: used,
            ..no_gaps()
        };
        layout.compute(&t, SimpleTree::ROOT, 800, 600, &options);
        let free = layout.free().map_or(0, |r| r.area());
        // Half the map, less the 1 px gap.
        assert!((free as i64 - 400 * 600).abs() <= 600, "free {free}");
        assert_disjoint(&layout);
    }

    #[test]
    fn folders_style_frames_large_folders() {
        let t = sample();
        let mut layout = Layout::new();
        layout.compute(
            &t,
            SimpleTree::ROOT,
            1920,
            1080,
            &MapStyle::Folders.layout_options(),
        );
        assert!(!layout.headers().is_empty());
        for h in layout.headers() {
            assert_eq!(h.rect.height(), LayoutOptions::HEADER);
            assert!((1..=LayoutOptions::FRAME_LEVELS).contains(&h.level));
            // No file rectangle covers a header.
            for r in layout.rects() {
                if r.kind != RectKind::Frame {
                    assert!(!r.rect.intersects(&h.rect));
                }
            }
        }
    }
}
