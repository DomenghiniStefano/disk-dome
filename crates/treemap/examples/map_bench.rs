//! Headless benchmark of the map (Trello cards 4.T4 and 4.T10, `docs/design-system.md` § 12
//! tests 4 and 10): layout, every style's redraw and the name placement at 1920 × 1080 on a
//! generated tree of about 50 000 visible rectangles. Opens no window.
//!
//! ```text
//! cargo run -p diskdome-treemap --release --example map_bench -- [--files N] [--sigma S] [--runs N] [--out DIR]
//! ```
//!
//! With `--out`, one BMP per style (and the names of «Many» and «All» outlined over Vignette, and
//! the free-space block) is written there.

use std::path::{Path, PathBuf};
use std::time::Instant;

use diskdome_treemap::{
    LabelMetrics, Layout, MapStyle, NamePlacer, NamesMode, Palette, Rect, RectKind, Renderer,
    SimpleTree, Theme, rgba,
};

const WIDTH: u32 = 1920;
const HEIGHT: u32 = 1080;
const WARM_UP: usize = 3;
/// 600 000 files with a heavy tail (σ 2.6) give about 50 000 rectangles at 1920 × 1080.
const DEFAULT_FILES: usize = 600_000;
const DEFAULT_SIGMA: f64 = 2.6;
/// A sparser map where every mode reaches its cap.
const SPARSE_FILES: usize = 3_000;

/// Category base colours, `docs/design-system.md` § 3.2.
const PALETTE: Palette = Palette([
    0xd9704f, 0xe68ebb, 0xa275c9, 0x5a86ce, 0x5f53ae, 0x4caccb, 0x197a67, 0x5cb572, 0x788a37,
    0xd1a255, 0x7d6e63, 0x55585d,
]);

/// Dark theme, § 3.2.
const THEME: Theme = Theme {
    bg: 0x0f1113,
    ink: 0xe8eaec,
    surface: 0x161719,
    line: 0x282a2d,
};

/// SplitMix64: deterministic, no dependency.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }

    /// Uniform in [0, 1).
    fn unit(&mut self) -> f64 {
        (self.next() >> 11) as f64 / (1u64 << 53) as f64
    }

    fn range(&mut self, lo: u64, hi: u64) -> u64 {
        lo + self.next() % (hi - lo)
    }

    /// Standard normal (Box–Muller).
    fn normal(&mut self) -> f64 {
        let u = self.unit().max(1e-12);
        let v = self.unit();
        (-2.0 * u.ln()).sqrt() * (std::f64::consts::TAU * v).cos()
    }
}

/// Folders four levels deep (10 at the top, 2–7 under each), `files` files spread over them,
/// log-normal sizes (median 64 KiB, spread `sigma`), categories skewed like a real drive.
fn generate(files: usize, sigma: f64, seed: u64) -> SimpleTree {
    let mut rng = Rng(seed);
    let mut tree = SimpleTree::new();
    let mut dirs = vec![SimpleTree::ROOT];
    let mut level = vec![SimpleTree::ROOT];
    for depth in 0..4 {
        let mut next = Vec::new();
        for &parent in &level {
            let n = if depth == 0 { 10 } else { rng.range(2, 8) };
            for _ in 0..n {
                let d = tree.add_dir(parent, rng.range(4, 20) as u16);
                next.push(d);
            }
        }
        dirs.extend_from_slice(&next);
        level = next;
    }
    // Some folders are much fuller than others.
    let weights: Vec<f64> = dirs.iter().map(|_| rng.unit().powi(3) + 0.02).collect();
    let total: f64 = weights.iter().sum();
    let mut cumulative = Vec::with_capacity(weights.len());
    let mut acc = 0.0;
    for w in &weights {
        acc += w / total;
        cumulative.push(acc);
    }
    // Weighted categories: other, documents, code, images, build output, data…
    const CATEGORY_WEIGHTS: [u64; 12] = [4, 8, 2, 10, 2, 8, 8, 14, 10, 4, 1, 29];
    let category_total: u64 = CATEGORY_WEIGHTS.iter().sum();
    for _ in 0..files {
        let pick = rng.unit();
        let slot = cumulative
            .partition_point(|&c| c < pick)
            .min(dirs.len() - 1);
        let size = (65_536.0 * (sigma * rng.normal()).exp()) as u64 + 1;
        let mut c = rng.range(0, category_total);
        let mut category = 0u8;
        for (k, &w) in CATEGORY_WEIGHTS.iter().enumerate() {
            if c < w {
                category = k as u8;
                break;
            }
            c -= w;
        }
        tree.add_file(dirs[slot], size, category, rng.range(3, 40) as u16);
    }
    tree.finish();
    tree
}

/// Median and 95th percentile of `runs` timed calls, in milliseconds.
fn time(runs: usize, mut f: impl FnMut()) -> (f64, f64) {
    for _ in 0..WARM_UP {
        f();
    }
    let mut ms: Vec<f64> = (0..runs)
        .map(|_| {
            let start = Instant::now();
            f();
            start.elapsed().as_secs_f64() * 1000.0
        })
        .collect();
    ms.sort_by(f64::total_cmp);
    let median = ms[ms.len() / 2];
    let p95 = ms[((ms.len() * 95).div_ceil(100)).min(ms.len()) - 1];
    (median, p95)
}

/// 24-bit BMP, bottom-up rows; `pixels` are RGBA in memory.
fn write_bmp(path: &Path, pixels: &[u32], width: u32, height: u32) -> std::io::Result<()> {
    let row = (width * 3).div_ceil(4) * 4;
    let data = row * height;
    let mut out = Vec::with_capacity(54 + data as usize);
    out.extend_from_slice(b"BM");
    out.extend_from_slice(&(54 + data).to_le_bytes());
    out.extend_from_slice(&0u32.to_le_bytes());
    out.extend_from_slice(&54u32.to_le_bytes());
    out.extend_from_slice(&40u32.to_le_bytes());
    out.extend_from_slice(&(width as i32).to_le_bytes());
    out.extend_from_slice(&(height as i32).to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes());
    out.extend_from_slice(&24u16.to_le_bytes());
    out.extend_from_slice(&0u32.to_le_bytes());
    out.extend_from_slice(&data.to_le_bytes());
    out.extend_from_slice(&2835u32.to_le_bytes());
    out.extend_from_slice(&2835u32.to_le_bytes());
    out.extend_from_slice(&0u32.to_le_bytes());
    out.extend_from_slice(&0u32.to_le_bytes());
    for y in (0..height as usize).rev() {
        let line = &pixels[y * width as usize..(y + 1) * width as usize];
        for &p in line {
            let [r, g, b, _] = p.to_le_bytes();
            out.extend_from_slice(&[b, g, r]);
        }
        out.extend(std::iter::repeat_n(0, (row - width * 3) as usize));
    }
    std::fs::write(path, out)
}

/// 1 px outline, for showing where names go.
fn outline(pixels: &mut [u32], width: usize, r: Rect, colour: u32) {
    for x in r.x0..r.x1 {
        pixels[r.y0 as usize * width + x as usize] = colour;
        pixels[(r.y1 - 1) as usize * width + x as usize] = colour;
    }
    for y in r.y0..r.y1 {
        pixels[y as usize * width + r.x0 as usize] = colour;
        pixels[y as usize * width + (r.x1 - 1) as usize] = colour;
    }
}

struct Args {
    files: usize,
    sigma: f64,
    runs: usize,
    out: Option<PathBuf>,
}

fn args() -> Args {
    let mut parsed = Args {
        files: DEFAULT_FILES,
        sigma: DEFAULT_SIGMA,
        runs: 20,
        out: None,
    };
    let mut it = std::env::args().skip(1);
    while let Some(a) = it.next() {
        match a.as_str() {
            "--files" => {
                parsed.files = it
                    .next()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(DEFAULT_FILES)
            }
            "--sigma" => {
                parsed.sigma = it
                    .next()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(DEFAULT_SIGMA)
            }
            "--runs" => parsed.runs = it.next().and_then(|v| v.parse().ok()).unwrap_or(20),
            "--out" => parsed.out = it.next().map(PathBuf::from),
            other => eprintln!("ignored argument: {other}"),
        }
    }
    parsed.runs = parsed.runs.max(1);
    parsed
}

fn main() -> std::io::Result<()> {
    let args = args();
    if let Some(dir) = &args.out {
        std::fs::create_dir_all(dir)?;
    }
    let tree = generate(args.files, args.sigma, 0x00d1_5cd0_3e00_0001);
    let root = SimpleTree::ROOT;
    println!(
        "tree: {} entries, {} files generated (log-normal sizes, sigma {}); {WIDTH}x{HEIGHT}; {} threads; median / p95 of {} runs after {WARM_UP} warm-up",
        tree.len(),
        args.files,
        args.sigma,
        rayon::current_num_threads(),
        args.runs
    );

    let mut layout = Layout::new();
    let mut renderer = Renderer::new();
    let mut placer = NamePlacer::new();
    let mut pixels = vec![0u32; (WIDTH * HEIGHT) as usize];

    println!();
    println!(
        "| Style | Rectangles (files / pieces / remainders / frames) | Layout ms (median / p95) | Redraw ms (median / p95) |"
    );
    println!("| --- | --- | --- | --- |");
    for style in MapStyle::ALL {
        let options = style.layout_options();
        let (layout_median, layout_p95) = time(args.runs, || {
            layout.compute(&tree, root, WIDTH, HEIGHT, &options);
        });
        let count = |kind: RectKind| layout.rects().iter().filter(|r| r.kind == kind).count();
        let counts = (
            count(RectKind::File),
            count(RectKind::Piece),
            count(RectKind::Remainder),
            count(RectKind::Frame),
        );
        let (median, p95) = time(args.runs, || {
            renderer.render(&layout, style, &THEME, &PALETTE, &mut pixels);
        });
        println!(
            "| {} | {} ({} / {} / {} / {}) | {layout_median:.2} / {layout_p95:.2} | {median:.2} / {p95:.2} |",
            style.key(),
            layout.rects().len(),
            counts.0,
            counts.1,
            counts.2,
            counts.3,
        );
        if let Some(dir) = &args.out {
            write_bmp(
                &dir.join(format!("{}.bmp", style.key())),
                &pixels,
                WIDTH,
                HEIGHT,
            )?;
        }
    }

    // Names on the default layout (Vignette), and on Folders.
    println!();
    println!(
        "| Layout | Names | Placed (name+size / name / small) | Placement ms (median / p95) |"
    );
    println!("| --- | --- | --- | --- |");
    for style in [MapStyle::Vignette, MapStyle::Folders] {
        layout.compute(&tree, root, WIDTH, HEIGHT, &style.layout_options());
        for mode in NamesMode::ALL {
            let (median, p95) = time(args.runs, || {
                placer.place(&layout, &tree, mode, &LabelMetrics::DEFAULT);
            });
            let labels = placer.labels();
            let tier = |t: diskdome_treemap::Tier| labels.iter().filter(|l| l.tier == t).count();
            println!(
                "| {} | {} | {} ({} / {} / {}) | {median:.3} / {p95:.3} |",
                style.key(),
                mode.key(),
                labels.len(),
                tier(diskdome_treemap::Tier::NameSize),
                tier(diskdome_treemap::Tier::Name),
                tier(diskdome_treemap::Tier::Small),
            );
        }
    }

    // On a sparse map every mode fills its cap: the placement's worst case.
    let sparse = generate(SPARSE_FILES, 1.3, 0x5eed);
    layout.compute(
        &sparse,
        root,
        WIDTH,
        HEIGHT,
        &MapStyle::Vignette.layout_options(),
    );
    for mode in NamesMode::ALL {
        let (median, p95) = time(args.runs, || {
            placer.place(&layout, &sparse, mode, &LabelMetrics::DEFAULT);
        });
        let labels = placer.labels();
        let tier = |t: diskdome_treemap::Tier| labels.iter().filter(|l| l.tier == t).count();
        println!(
            "| sparse ({} rectangles) | {} | {} ({} / {} / {}) | {median:.3} / {p95:.3} |",
            layout.rects().len(),
            mode.key(),
            labels.len(),
            tier(diskdome_treemap::Tier::NameSize),
            tier(diskdome_treemap::Tier::Name),
            tier(diskdome_treemap::Tier::Small),
        );
    }

    if let Some(dir) = &args.out {
        layout.compute(
            &tree,
            root,
            WIDTH,
            HEIGHT,
            &MapStyle::Vignette.layout_options(),
        );
        for mode in [NamesMode::Many, NamesMode::All] {
            renderer.render(&layout, MapStyle::Vignette, &THEME, &PALETTE, &mut pixels);
            for label in placer.place(&layout, &tree, mode, &LabelMetrics::DEFAULT) {
                outline(&mut pixels, WIDTH as usize, label.rect, rgba(THEME.ink));
            }
            write_bmp(
                &dir.join(format!("names-{}.bmp", mode.key())),
                &pixels,
                WIDTH,
                HEIGHT,
            )?;
        }
        let used = diskdome_treemap::MapSource::size(&tree, root);
        let options = diskdome_treemap::LayoutOptions {
            free_space: used / 2,
            ..MapStyle::Vignette.layout_options()
        };
        layout.compute(&tree, root, WIDTH, HEIGHT, &options);
        renderer.render(&layout, MapStyle::Vignette, &THEME, &PALETTE, &mut pixels);
        write_bmp(&dir.join("free-space.bmp"), &pixels, WIDTH, HEIGHT)?;
        println!();
        println!("images written to {}", dir.display());
    }
    Ok(())
}
