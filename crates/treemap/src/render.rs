//! Draws a [`Layout`] into `u32` pixels, every style of `docs/design-system.md` § 4.4.
//!
//! The buffer is cut into bands of rows drawn in parallel (rayon); each band draws only the
//! rectangles that cross it, listed once per frame in reusable buffers (a counting sort by band),
//! in layout order so a frame is drawn before what it holds.
//!
//! Pixels are RGBA8 in memory (byte 0 red … byte 3 alpha), the layout of Slint's `Rgba8Pixel`:
//! as a little-endian `u32`, `0xAABBGGRR`. See [`rgba`].

use rayon::prelude::{IndexedParallelIterator, ParallelIterator, ParallelSliceMut};

use crate::CATEGORIES;
use crate::geom::Rect;
use crate::layout::{Layout, MapRect, RectKind};
use crate::style::MapStyle;

/// Rows per band: small enough to spread the work over the cores, large enough that a band holds
/// many rectangles.
const BAND: usize = 16;

/// The pixel of a `0xRRGGBB` colour, opaque.
pub const fn rgba(rgb: u32) -> u32 {
    let r = (rgb >> 16) & 0xff;
    let g = (rgb >> 8) & 0xff;
    let b = rgb & 0xff;
    r | (g << 8) | (b << 16) | (0xff << 24)
}

/// The theme colours the map uses, `0xRRGGBB` (§ 3.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Theme {
    /// Gaps, lines, the ground of folder headers.
    pub bg: u32,
    /// Folder headers are `bg` mixed toward it.
    pub ink: u32,
    /// Free-space block.
    pub surface: u32,
    /// Free-space stripes and edge.
    pub line: u32,
}

/// One colour per file category, `0xRRGGBB`, indexed by [`crate::MapSource::category`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Palette(pub [u32; CATEGORIES]);

type Rgb = [f32; 3];

fn channels(rgb: u32) -> Rgb {
    [
        ((rgb >> 16) & 0xff) as f32,
        ((rgb >> 8) & 0xff) as f32,
        (rgb & 0xff) as f32,
    ]
}

/// `a` moved `t` of the way toward `b`.
fn mix(a: Rgb, b: Rgb, t: f32) -> Rgb {
    [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
    ]
}

#[inline(always)]
fn pack(c: Rgb) -> u32 {
    // Values stay within 0..=255 by construction; `as` saturates anyway.
    let r = (c[0] + 0.5) as u32;
    let g = (c[1] + 0.5) as u32;
    let b = (c[2] + 0.5) as u32;
    r | (g << 8) | (b << 16) | 0xff00_0000
}

/// `c · s + 255 · a`: `s = 1 − k` darkens by `k` toward black; `s = 1 − a` with `a` lightens
/// toward white.
#[inline(always)]
fn shade(c: Rgb, s: f32, a: f32) -> u32 {
    let w = 255.0 * a;
    pack([c[0] * s + w, c[1] * s + w, c[2] * s + w])
}

const BLACK: Rgb = [0.0, 0.0, 0.0];
const WHITE: Rgb = [255.0, 255.0, 255.0];

// Style numbers, from § 4.4.
const VIGNETTE_FULL: f32 = 0.35;
const VIGNETTE_CENTRE_Y: f32 = 0.42;
const VIGNETTE_DARK: f32 = 0.32;
const DIAGONAL_LIGHT: f32 = 0.10;
const DIAGONAL_FULL: f32 = 0.45;
const DIAGONAL_DARK: f32 = 0.22;
const MOSAIC_EDGE: f32 = 0.35;
const STRUCTURE_EDGE: f32 = 0.18;
const RELIEF_EDGE: f32 = 0.38;
const CUSHION_AMBIENT: f32 = 0.40;
const CUSHION_DIFFUSE: f32 = 0.60;
/// Scales the cushion's light so the lit top-left peaks 10 % toward white, the colour § 4.4 gives
/// for names on Cushion; below 1 the colour darkens toward black.
const CUSHION_BRIGHTNESS: f32 = 1.10;
/// Light from the top left and above, as in WinDirStat: (−1, −1, 10), normalised.
const CUSHION_LIGHT: [f32; 3] = [-0.098_532_93, -0.098_532_93, 0.985_329_3];
/// Folder headers: `bg` toward `ink` 6 %, +2.5 % per level.
const HEADER_MIX: f32 = 0.06;
const HEADER_MIX_STEP: f32 = 0.025;
/// Free space: a 1 px stripe every 7 px measured across the 135° stripes (every 10 in `x + y`).
const FREE_STRIPE: i32 = 10;

/// Colours derived once per frame.
struct Paint {
    style: MapStyle,
    bg: u32,
    palette: [Rgb; CATEGORIES],
    headers: [u32; 4],
    surface: u32,
    line: u32,
}

/// Where a band draws: its pixels and the rows they hold.
struct Band<'a> {
    pixels: &'a mut [u32],
    width: usize,
    y0: i32,
    y1: i32,
}

impl Band<'_> {
    /// The pixels of row `y` (absolute) between `x0` and `x1`.
    #[inline(always)]
    fn row(&mut self, y: i32, x0: i32, x1: i32) -> &mut [u32] {
        let start = (y - self.y0) as usize * self.width;
        &mut self.pixels[start + x0 as usize..start + x1 as usize]
    }
}

/// Draws layouts; keeps its buffers between frames.
#[derive(Debug, Default)]
pub struct Renderer {
    band_start: Vec<u32>,
    band_items: Vec<u32>,
    cursor: Vec<u32>,
}

impl Renderer {
    pub fn new() -> Self {
        Self::default()
    }

    /// Draws `layout` in `style` into `pixels`, `layout.width()` per row, at least
    /// `layout.height()` rows (see [`rgba`] for the format). A buffer too small draws nothing.
    pub fn render(
        &mut self,
        layout: &Layout,
        style: MapStyle,
        theme: &Theme,
        palette: &Palette,
        pixels: &mut [u32],
    ) {
        let width = layout.width().max(0) as usize;
        let height = layout.height().max(0) as usize;
        if width == 0 || height == 0 || pixels.len() < width * height {
            return;
        }
        self.sort_into_bands(layout, height);

        let bg = channels(theme.bg);
        let ink = channels(theme.ink);
        let mut headers = [0; 4];
        for (level, h) in headers.iter_mut().enumerate() {
            *h = pack(mix(bg, ink, HEADER_MIX + HEADER_MIX_STEP * level as f32));
        }
        let paint = Paint {
            style,
            bg: rgba(theme.bg),
            palette: palette.0.map(channels),
            headers,
            surface: rgba(theme.surface),
            line: rgba(theme.line),
        };
        let rects = layout.rects();
        let cushion = layout.cushion();
        let band_start = &self.band_start;
        let band_items = &self.band_items;
        let paint = &paint;

        pixels[..width * height]
            .par_chunks_mut(width * BAND)
            .enumerate()
            .for_each(|(b, chunk)| {
                chunk.fill(paint.bg);
                let rows = chunk.len() / width;
                let mut band = Band {
                    pixels: chunk,
                    width,
                    y0: (b * BAND) as i32,
                    y1: (b * BAND + rows) as i32,
                };
                let items = &band_items[band_start[b] as usize..band_start[b + 1] as usize];
                for &i in items {
                    let i = i as usize;
                    draw(&mut band, paint, &rects[i], &cushion[i]);
                }
            });
    }

    /// Lists, for every band, the rectangles crossing it, in layout order.
    fn sort_into_bands(&mut self, layout: &Layout, height: usize) {
        let bands = height.div_ceil(BAND);
        self.band_start.clear();
        self.band_start.resize(bands + 1, 0);
        let span = |r: &Rect| {
            let first = r.y0.max(0) as usize / BAND;
            let last = ((r.y1 - 1).max(0) as usize / BAND).min(bands - 1);
            first..=last
        };
        for r in layout.rects() {
            for b in span(&r.rect) {
                self.band_start[b + 1] += 1;
            }
        }
        for b in 0..bands {
            self.band_start[b + 1] += self.band_start[b];
        }
        self.cursor.clear();
        self.cursor.extend_from_slice(&self.band_start[..bands]);
        self.band_items.clear();
        self.band_items.resize(self.band_start[bands] as usize, 0);
        for (i, r) in layout.rects().iter().enumerate() {
            for b in span(&r.rect) {
                self.band_items[self.cursor[b] as usize] = i as u32;
                self.cursor[b] += 1;
            }
        }
    }
}

/// Draws one rectangle, clipped to the band.
fn draw(band: &mut Band, paint: &Paint, item: &MapRect, cushion: &[f32; 4]) {
    let r = item.rect;
    let x0 = r.x0.max(0);
    let x1 = r.x1.min(band.width as i32);
    let y0 = r.y0.max(band.y0);
    let y1 = r.y1.min(band.y1);
    if x0 >= x1 || y0 >= y1 {
        return;
    }
    let clip = Rect::new(x0, y0, x1, y1);
    match item.kind {
        RectKind::Frame => {
            let level = usize::from(item.depth.clamp(1, 4)) - 1;
            fill(band, clip, paint.headers[level]);
            return;
        }
        RectKind::Free => {
            free_space(band, &r, clip, paint);
            return;
        }
        RectKind::File | RectKind::Piece | RectKind::Remainder => {}
    }
    let c = paint.palette[usize::from(item.category) % CATEGORIES];
    let flat = pack(c);
    match paint.style {
        MapStyle::Vignette => vignette(band, &r, clip, c),
        MapStyle::DiagonalLight => diagonal(band, &r, clip, c),
        MapStyle::Mosaic => {
            let e = pack(mix(c, BLACK, MOSAIC_EDGE));
            edges(band, &r, clip, flat, [Some(e), Some(e), Some(e), Some(e)]);
        }
        MapStyle::Structure => {
            let e = pack(mix(c, BLACK, STRUCTURE_EDGE));
            edges(band, &r, clip, flat, [None, None, Some(e), Some(e)]);
        }
        MapStyle::Flat | MapStyle::Folders => {
            edges(
                band,
                &r,
                clip,
                flat,
                [None, None, Some(paint.bg), Some(paint.bg)],
            );
        }
        MapStyle::Cushion => cushion_shade(band, &r, clip, c, cushion),
        MapStyle::Relief => {
            let light = pack(mix(c, WHITE, RELIEF_EDGE));
            let dark = pack(mix(c, BLACK, RELIEF_EDGE));
            edges(
                band,
                &r,
                clip,
                flat,
                [Some(light), Some(light), Some(dark), Some(dark)],
            );
        }
    }
}

fn fill(band: &mut Band, clip: Rect, colour: u32) {
    for y in clip.y0..clip.y1 {
        band.row(y, clip.x0, clip.x1).fill(colour);
    }
}

/// Flat colour with optional 1 px edges `[top, left, right, bottom]`; bottom and right win over
/// top and left at the corners.
fn edges(band: &mut Band, r: &Rect, clip: Rect, colour: u32, edge: [Option<u32>; 4]) {
    let [top, left, right, bottom] = edge;
    let left = left.filter(|_| r.x0 == clip.x0);
    let right = right.filter(|_| r.x1 == clip.x1);
    for y in clip.y0..clip.y1 {
        let row = band.row(y, clip.x0, clip.x1);
        if let (Some(b), true) = (bottom, y == r.y1 - 1) {
            row.fill(b);
            continue;
        }
        match (top, y == r.y0) {
            (Some(t), true) => row.fill(t),
            (Some(_), false) | (None, true) | (None, false) => {
                row.fill(colour);
                if let (Some(l), Some(px)) = (left, row.first_mut()) {
                    *px = l;
                }
            }
        }
        if let (Some(rt), Some(px)) = (right, row.last_mut()) {
            *px = rt;
        }
    }
}

/// Radial gradient: CSS `radial-gradient(farthest-corner at 50% 42%, c 35%, c·0.68 100%)`. The
/// ellipse has the proportions of the closest sides and passes through the farthest corner.
fn vignette(band: &mut Band, r: &Rect, clip: Rect, c: Rgb) {
    let w = r.width() as f32;
    let h = r.height() as f32;
    let cx = r.x0 as f32 + 0.5 * w;
    let cy = r.y0 as f32 + VIGNETTE_CENTRE_Y * h;
    let inv_rx = 2.0 / w;
    let inv_ry = 1.0 / (VIGNETTE_CENTRE_Y * h);
    // Farthest corner (a bottom one) in those units.
    let far_y = (1.0 - VIGNETTE_CENTRE_Y) / VIGNETTE_CENTRE_Y;
    let inv_far = 1.0 / (1.0 + far_y * far_y).sqrt();
    let slope = VIGNETTE_DARK / (1.0 - VIGNETTE_FULL);
    for y in clip.y0..clip.y1 {
        let dy = (y as f32 + 0.5 - cy) * inv_ry;
        let dy2 = dy * dy;
        let row = band.row(y, clip.x0, clip.x1);
        let mut dx = (clip.x0 as f32 + 0.5 - cx) * inv_rx;
        for px in row.iter_mut() {
            let t = (dx * dx + dy2).sqrt() * inv_far;
            let k = ((t - VIGNETTE_FULL) * slope).clamp(0.0, VIGNETTE_DARK);
            *px = shade(c, 1.0 - k, 0.0);
            dx += inv_rx;
        }
    }
}

/// Linear gradient at 135°: `c` 10 % toward white at the top left, `c` at 45 %, 22 % toward black
/// at the bottom right. Along 135° the position is `(dx + dy) / (w + h)`.
fn diagonal(band: &mut Band, r: &Rect, clip: Rect, c: Rgb) {
    let inv = 1.0 / (r.width() + r.height()) as f32;
    let light = DIAGONAL_LIGHT / DIAGONAL_FULL;
    let dark = DIAGONAL_DARK / (1.0 - DIAGONAL_FULL);
    for y in clip.y0..clip.y1 {
        let row = band.row(y, clip.x0, clip.x1);
        let mut p = ((clip.x0 - r.x0) as f32 + (y - r.y0) as f32 + 1.0) * inv;
        for px in row.iter_mut() {
            let a = ((DIAGONAL_FULL - p) * light).max(0.0);
            let k = ((p - DIAGONAL_FULL) * dark).max(0.0);
            *px = shade(c, 1.0 - a - k, a);
            p += inv;
        }
    }
}

/// van Wijk & van de Wetering cushion: intensity `ambient + diffuse · cos(normal, light)`,
/// scaled by [`CUSHION_BRIGHTNESS`]; above 1 the colour goes toward white.
fn cushion_shade(band: &mut Band, r: &Rect, clip: Rect, c: Rgb, n: &[f32; 4]) {
    let [nx0, ny0, dnx, dny] = *n;
    let [lx, ly, lz] = CUSHION_LIGHT;
    for y in clip.y0..clip.y1 {
        let ny = ny0 + dny * (y - r.y0) as f32;
        let ny_light = ny * ly + lz;
        let ny2 = ny * ny + 1.0;
        let row = band.row(y, clip.x0, clip.x1);
        let mut nx = nx0 + dnx * (clip.x0 - r.x0) as f32;
        for px in row.iter_mut() {
            let cos = (nx * lx + ny_light) / (nx * nx + ny2).sqrt();
            let v = (CUSHION_AMBIENT + CUSHION_DIFFUSE * cos.max(0.0)) * CUSHION_BRIGHTNESS;
            let a = (v - 1.0).max(0.0);
            *px = shade(c, v.min(1.0) - a, a);
            nx += dnx;
        }
    }
}

/// `surface` with a 1 px `line` stripe at 135° and a 1 px `line` edge.
fn free_space(band: &mut Band, r: &Rect, clip: Rect, paint: &Paint) {
    for y in clip.y0..clip.y1 {
        let row = band.row(y, clip.x0, clip.x1);
        if y == r.y0 || y == r.y1 - 1 {
            row.fill(paint.line);
            continue;
        }
        for (k, px) in row.iter_mut().enumerate() {
            let x = clip.x0 + k as i32;
            *px = if x == r.x0 || x == r.x1 - 1 || (x + y).rem_euclid(FREE_STRIPE) == 0 {
                paint.line
            } else {
                paint.surface
            };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Palette, Renderer, Theme, rgba};
    use crate::layout::Layout;
    use crate::source::SimpleTree;
    use crate::style::MapStyle;

    const THEME: Theme = Theme {
        bg: 0x0f1113,
        ink: 0xe8eaec,
        surface: 0x161719,
        line: 0x282a2d,
    };

    #[test]
    fn pixel_format_is_rgba_in_memory() {
        assert_eq!(rgba(0x112233).to_le_bytes(), [0x11, 0x22, 0x33, 0xff]);
    }

    #[test]
    fn every_style_covers_the_map() {
        let mut t = SimpleTree::new();
        for d in 0..4 {
            let dir = t.add_dir(SimpleTree::ROOT, 5);
            for f in 0..20u64 {
                t.add_file(dir, 100 + f * 13 + d, (f % 12) as u8, 5);
            }
        }
        t.finish();
        let palette = Palette([0x808080; 12]);
        let mut layout = Layout::new();
        let mut renderer = Renderer::new();
        let mut pixels = vec![0u32; 320 * 200];
        for style in MapStyle::ALL {
            pixels.fill(0);
            layout.compute(&t, SimpleTree::ROOT, 320, 200, &style.layout_options());
            renderer.render(&layout, style, &THEME, &palette, &mut pixels);
            assert!(pixels.iter().all(|&p| p >> 24 == 0xff), "{style:?}");
            // Some pixels carry the category colour family (not only the background).
            let bg = rgba(THEME.bg);
            assert!(pixels.iter().filter(|&&p| p != bg).count() > 320 * 200 / 2);
        }
    }
}
