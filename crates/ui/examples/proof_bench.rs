//! Headless measurements of the table proof (Trello card 3c). Nothing is shown on screen: the window is
//! drawn by Slint's software renderer into a buffer.
//!
//! ```text
//! cargo run -p diskdome-ui --release --example proof_bench -- --rows 14000000 --view table
//! ```
//!
//! Options: `--rows N` (default 14,000,000), `--view table|list`, `--width W --height H` (default 960×600),
//! `--dump DIR` writes BMP screenshots of key frames, `--hold SECONDS` waits before exiting so the process
//! memory can be read from outside.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

use diskdome_ui::proof::data::{FakeTree, SortColumn};
use diskdome_ui::proof::headless::Headless;
use diskdome_ui::proof::model::{ProofRows, RowStats};
use diskdome_ui::proof::{ProofWindow, View};
use slint::platform::Key;

struct Options {
    rows: u32,
    view: View,
    width: u32,
    height: u32,
    dump: Option<PathBuf>,
    hold: u64,
}

fn options() -> Options {
    let mut options = Options {
        rows: 14_000_000,
        view: View::Table,
        width: 960,
        height: 600,
        dump: None,
        hold: 0,
    };
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        let value = args.next().unwrap_or_default();
        match arg.as_str() {
            "--rows" => options.rows = value.parse().unwrap_or(options.rows),
            "--view" => options.view = View::parse(&value).unwrap_or(options.view),
            "--width" => options.width = value.parse().unwrap_or(options.width),
            "--height" => options.height = value.parse().unwrap_or(options.height),
            "--dump" => options.dump = Some(PathBuf::from(value)),
            "--hold" => options.hold = value.parse().unwrap_or(0),
            other => eprintln!("ignored argument {other}"),
        }
    }
    options
}

/// Frame times and model calls of one scenario.
struct Scenario {
    name: &'static str,
    frames: Vec<Duration>,
    calls: Vec<u64>,
}

impl Scenario {
    fn new(name: &'static str) -> Self {
        Self {
            name,
            frames: Vec::new(),
            calls: Vec::new(),
        }
    }

    fn report(&self) {
        if self.frames.is_empty() {
            println!("{:<28} no frame drawn", self.name);
            return;
        }
        let mut sorted = self.frames.clone();
        sorted.sort();
        let pick = |q: f64| {
            sorted[((sorted.len() - 1) as f64 * q).round() as usize].as_secs_f64() * 1000.0
        };
        let calls: u64 = self.calls.iter().sum();
        let over_16 = sorted
            .iter()
            .filter(|frame| frame.as_secs_f64() > 1.0 / 60.0)
            .count();
        println!(
            "{:<28} frames {:>4}  p50 {:>6.2} ms  p95 {:>6.2} ms  p99 {:>6.2} ms  max {:>6.2} ms  >16.7ms {:>3}  row_data/frame {:>5.1}",
            self.name,
            sorted.len(),
            pick(0.5),
            pick(0.95),
            pick(0.99),
            pick(1.0),
            over_16,
            calls as f64 / sorted.len() as f64
        );
    }
}

struct Bench {
    headless: Headless,
    window: ProofWindow,
    stats: std::rc::Rc<RowStats>,
    dump: Option<PathBuf>,
}

impl Bench {
    /// Draws one frame, recording its time and the model calls it caused.
    fn frame(&mut self, scenario: &mut Scenario) {
        self.headless.request_redraw();
        if let Some(time) = self.headless.frame() {
            scenario.frames.push(time);
            scenario.calls.push(self.stats.take().0);
        }
    }

    fn scroll_to(&mut self, offset: f32, scenario: &mut Scenario) {
        self.window.set_scroll(offset);
        self.frame(scenario);
    }

    fn max_scroll(&self) -> f32 {
        (self.window.content_height() - self.window.visible_height()).max(0.0)
    }

    fn row_height(&self, rows: u32) -> f32 {
        self.window.content_height() / rows.max(1) as f32
    }

    fn dump(&self, name: &str) {
        if let Some(dir) = &self.dump {
            let path = dir.join(format!("{name}.bmp"));
            if let Err(error) = std::fs::write(&path, self.headless.bmp()) {
                eprintln!("cannot write {}: {error}", path.display());
            }
        }
    }
}

fn main() -> Result<(), slint::PlatformError> {
    let options = options();
    println!(
        "rows {}  view {:?}  window {}x{}",
        options.rows, options.view, options.width, options.height
    );

    let start = Instant::now();
    let tree = Arc::new(FakeTree::generate(options.rows));
    println!(
        "generate: {:.0} ms, {:.1} MB in arrays ({:.1} bytes per row)",
        start.elapsed().as_secs_f64() * 1000.0,
        tree.heap_bytes() as f64 / 1e6,
        tree.heap_bytes() as f64 / f64::from(options.rows.max(1))
    );

    for (column, descending, label) in [
        (SortColumn::Size, false, "sort size asc"),
        (SortColumn::Size, true, "sort size desc"),
        (SortColumn::Name, false, "sort name asc"),
        (SortColumn::Percent, false, "sort % asc"),
        (SortColumn::Modified, false, "sort modified asc"),
    ] {
        let start = Instant::now();
        let order = tree.sorted_order(column, descending);
        println!(
            "{label:<18} {:>6.0} ms ({} rows)",
            start.elapsed().as_secs_f64() * 1000.0,
            order.len()
        );
    }

    let headless = Headless::install(options.width, options.height)?;
    let rows = ProofRows::new(tree.clone());
    let stats = rows.stats();
    let window = ProofWindow::new(options.view, rows.clone(), false)?;
    window.show()?;
    let mut bench = Bench {
        headless,
        window,
        stats,
        dump: options.dump.clone(),
    };

    let mut first = Scenario::new("first frame");
    bench.frame(&mut first);
    first.report();
    bench.dump("01-top");

    let row_height = bench.row_height(options.rows);
    let max_scroll = bench.max_scroll();
    println!(
        "content height {:.0} px, visible {:.0} px, row height {:.3} px, max scroll {:.0} px",
        bench.window.content_height(),
        bench.window.visible_height(),
        row_height,
        max_scroll
    );
    // Slint lengths are f32: at the bottom of a long list a scroll position cannot be finer than this.
    let ulp = f32::from_bits(max_scroll.to_bits() + 1) - max_scroll;
    println!("f32 step of the scroll position at the end of the list: {ulp} px");

    // Mouse wheel: three rows per frame from the top.
    let mut wheel = Scenario::new("wheel, 3 rows/frame");
    for step in 1..=600 {
        bench.scroll_to(step as f32 * 3.0 * row_height, &mut wheel);
    }
    wheel.report();

    // Fast flick: one page per frame.
    let mut pages = Scenario::new("page per frame");
    let page = bench.window.visible_height();
    let base = bench.window.scroll();
    for step in 1..=300 {
        bench.scroll_to(base + step as f32 * page, &mut pages);
    }
    pages.report();

    // Scrollbar drag: random jumps anywhere in the list.
    let mut jumps = Scenario::new("random jumps");
    let mut seed = 0x2545_F491_4F6C_DD1Du64;
    for _ in 0..300 {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        let fraction = (seed >> 11) as f64 / (1u64 << 53) as f64;
        bench.scroll_to((fraction * f64::from(max_scroll)) as f32, &mut jumps);
    }
    jumps.report();

    // The very end, then the wheel upwards from there.
    let mut end = Scenario::new("jump to end");
    bench.scroll_to(max_scroll, &mut end);
    end.report();
    bench.dump("02-end");
    let (_, low, high) = {
        bench.window.set_scroll(max_scroll);
        bench.headless.request_redraw();
        bench.headless.frame();
        bench.stats.take()
    };
    println!(
        "at the end: scroll {:.0} px, rows asked {low}..={high} (last row is {})",
        bench.window.scroll(),
        options.rows.saturating_sub(1)
    );
    let mut wheel_end = Scenario::new("wheel up from the end");
    for step in 1..=300 {
        bench.scroll_to(max_scroll - step as f32 * 3.0 * row_height, &mut wheel_end);
    }
    wheel_end.report();
    bench.dump("03-near-end");

    // One row per frame near the end: does the list move by one row each time?
    let mut fine = Scenario::new("1 row/frame at the end");
    let mut firsts = Vec::new();
    for step in 0..20 {
        bench
            .window
            .set_scroll(max_scroll - step as f32 * row_height);
        bench.headless.request_redraw();
        if let Some(time) = bench.headless.frame() {
            fine.frames.push(time);
            let (calls, low, _) = bench.stats.take();
            fine.calls.push(calls);
            firsts.push(if calls == 0 { None } else { Some(low) });
        }
    }
    fine.report();
    println!("  lowest row asked per step (None = instances reused, no new row asked): {firsts:?}");

    // Touchpad-like wheel events of 4 px, sent to the window as the OS would: near the top, then 1000 rows
    // before the end. Slint lengths are f32, so a step under half the spacing there may be lost.
    let (width, height) = bench.headless.size();
    let (x, y) = (width as f32 / 2.0, height as f32 / 2.0);
    for (label, start) in [
        ("near the top", 4.0 * row_height),
        ("1000 rows before the end", max_scroll - 1000.0 * row_height),
    ] {
        bench.window.set_scroll(start);
        bench.headless.frame();
        let before = bench.window.scroll();
        for _ in 0..20 {
            bench.headless.wheel(x, y, -4.0);
            bench.headless.frame();
        }
        println!(
            "20 wheel events of 4 px {label}: moved {:.0} px (80 expected)",
            bench.window.scroll() - before
        );
    }

    // Keyboard: focus the table, then Down 100 times from the top.
    bench.window.set_scroll(0.0);
    bench.window.focus_table();
    bench.headless.frame();
    let mut keys = Scenario::new("key down x100 (top)");
    for _ in 0..100 {
        bench.headless.press_key(Key::DownArrow);
        bench.frame(&mut keys);
    }
    keys.report();
    println!(
        "  current row after 100 x Down: {}",
        bench.window.current_row()
    );
    if options.view == View::List {
        let mut end_key = Scenario::new("key End");
        bench.headless.press_key(Key::End);
        bench.frame(&mut end_key);
        end_key.report();
        println!(
            "  current row after End: {}, scroll {:.0} px",
            bench.window.current_row(),
            bench.window.scroll()
        );
        bench.dump("04-key-end");
    }

    // Sorting through the window, as a header click does, then the frame that follows.
    bench.window.set_scroll(0.0);
    for (column, label) in [(1, "size"), (0, "name")] {
        let start = Instant::now();
        bench.window.request_sort(column, true);
        let sort_time = start.elapsed();
        let mut after = Scenario::new("frame after sort");
        bench.frame(&mut after);
        println!(
            "sort by {label} desc through the window: {:.0} ms",
            sort_time.as_secs_f64() * 1000.0
        );
        after.report();
        bench.dump(&format!("05-sorted-{label}"));
    }

    if options.hold > 0 {
        println!("HOLD pid={}", std::process::id());
        std::thread::sleep(Duration::from_secs(options.hold));
    }
    drop(rows);
    Ok(())
}
