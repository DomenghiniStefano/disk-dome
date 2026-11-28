//! ProgressBar and UsageBar off screen (`docs/design-system.md` § 5.21), on their test window `KitProgressHarness`: a
//! 2 px determinate scan line that fills `value` of `maximum`, eases to a new value (at once with reduced motion),
//! keeps inside its track and says its value; an indeterminate one whose 30 % segment slides across the track over
//! the frames (by pixels) and stays still over the start with reduced motion; usage bars 6 px and 4 px tall in both
//! densities, drawn at once, `bar-fill` on `bar-track`, and `warning` only above 90 %.

#![cfg(feature = "kit")]

use std::time::Duration;

use diskdome_ui::design_system;
use diskdome_ui::generated::{KitProgressHarness, ProgressBarMode, ProgressBarState, Size};
use diskdome_ui::headless::Headless;
use slint::ComponentHandle;

const WIDTH: u32 = 960;
const HEIGHT: u32 = 600;
/// Where the harness puts the bars: x = `space-32`, one `space-32` below the other, `dialog-sm` (400) wide.
const INSET: f32 = 32.0;
const BAR: f32 = 400.0;
/// § 5.21: the scan line is 2 px; usage bars 6 px on the drive cards, 4 px in the picker.
const LINE: f32 = 2.0;
const USAGE: f32 = 6.0;
const USAGE_SMALL: f32 = 4.0;
/// § 5.21: the indeterminate segment covers 30 % of the track: 0.30 × 400 = 120.
const SEGMENT: f32 = 120.0;
/// The dark theme's colours (§ 3.2): `bar-fill`, `bar-track`, `warning`, `accent`.
const BAR_FILL: [u8; 4] = [0x6e, 0x70, 0x72, 0xff];
const BAR_TRACK: [u8; 4] = [0x2b, 0x2c, 0x2e, 0xff];
const WARNING: [u8; 4] = [0xf6, 0xb8, 0x4d, 0xff];
const ACCENT: [u8; 4] = [0x37, 0xc2, 0xbb, 0xff];
/// Past `Motion.base` (140 ms), the fill's ease, with a margin for a busy CI machine.
const AFTER_EASE: Duration = Duration::from_millis(400);
/// Between two looks at the sliding segment: it crosses 400 + 120 px in `slow` × 6 = 1200 ms, so ~43 px.
const STEP: Duration = Duration::from_millis(100);
const LOOKS: usize = 12;

/// Two turns of the loop: a timer or a change handler is seen in the next one.
fn settle(headless: &mut Headless) {
    headless.frame();
    headless.frame();
}

/// Which columns of a bar's track are lit (not the window's ground), along one row of pixels.
fn lit(headless: &Headless, y: f32, ground: [u8; 4]) -> Vec<bool> {
    (0..BAR as usize)
        .map(|column| headless.pixel(INSET as usize + column, y as usize) != ground)
        .collect()
}

/// One test: the headless platform is installed once per thread.
#[test]
fn progress_and_usage_bars_fill_slide_and_warn() {
    let mut headless = Headless::install(WIDTH, HEIGHT).expect("headless platform");
    let harness = KitProgressHarness::new().expect("the harness compiles");
    design_system::install(&harness);
    harness.show().expect("the harness shows off screen");
    settle(&mut headless);

    // Where the harness put the bars, from the tokens.
    assert_eq!(
        harness.get_inset(),
        INSET,
        "the harness's inset is `space-32`"
    );
    assert_eq!(
        harness.get_bar_width(),
        BAR,
        "the bars are `dialog-sm` wide"
    );
    assert_eq!(harness.get_progress_x(), INSET, "the first bar's place");
    // The window's ground, left of every bar.
    let ground = headless.pixel(INSET as usize / 2, harness.get_progress_y() as usize);

    // Determinate: 2 px, 25 of 100 fills 100 of 400 px, in `accent`, the rest of the track transparent.
    assert_eq!(harness.get_progress_height(), LINE, "the scan line is 2 px");
    assert_eq!(harness.get_progress_fill(), 100.0, "25 % of 400 px");
    assert_eq!(harness.get_progress_state(), ProgressBarState::Loading);
    assert_eq!(
        harness.get_progress_said(),
        "25",
        "a screen reader says the value"
    );
    let line_y = harness.get_progress_y() as usize;
    assert_eq!(
        headless.pixel(INSET as usize + 50, line_y),
        ACCENT,
        "the fill is `accent`"
    );
    assert_eq!(
        headless.pixel(INSET as usize + 200, line_y),
        ground,
        "the track is transparent"
    );

    // A new value eases there (`base`, § 3.10): not yet in the next frame, there after the ease.
    harness.set_progress_value(75.0);
    settle(&mut headless);
    assert!(
        harness.get_progress_fill() < 300.0,
        "the fill eases to a new value, it does not jump"
    );
    headless.wait(AFTER_EASE);
    assert_eq!(harness.get_progress_fill(), 300.0, "75 % of 400 px");

    // Outside 0…maximum it shows full or empty; another maximum; words for the screen reader.
    harness.set_progress_value(150.0);
    headless.wait(AFTER_EASE);
    assert_eq!(
        harness.get_progress_fill(),
        BAR,
        "past the maximum it is full"
    );
    assert_eq!(harness.get_progress_state(), ProgressBarState::Complete);
    assert_eq!(harness.get_progress_said(), "100", "said within the range");
    harness.set_progress_value(-10.0);
    headless.wait(AFTER_EASE);
    assert_eq!(harness.get_progress_fill(), 0.0, "below 0 it is empty");
    assert_eq!(harness.get_progress_said(), "0", "said within the range");
    harness.set_progress_maximum(200.0);
    harness.set_progress_value(50.0);
    headless.wait(AFTER_EASE);
    assert_eq!(harness.get_progress_fill(), 100.0, "50 of 200 is a quarter");
    harness.set_progress_words("1.2 of 4.2 GB".into());
    settle(&mut headless);
    assert_eq!(
        harness.get_progress_said(),
        "1.2 of 4.2 GB",
        "the screen's own words"
    );
    harness.set_progress_mode(ProgressBarMode::Indeterminate);
    settle(&mut headless);
    assert_eq!(
        harness.get_progress_state(),
        ProgressBarState::Indeterminate
    );
    assert_eq!(
        harness.get_progress_said(),
        "",
        "no value while indeterminate"
    );
    harness.set_progress_mode(ProgressBarMode::Determinate);
    settle(&mut headless);

    // Indeterminate: a segment 30 % of the track slides across it. Looked at by pixels over the frames, the lit
    // columns change at every look and are never more than the segment; once wholly inside the track, they are it.
    assert_eq!(harness.get_sweep_state(), ProgressBarState::Indeterminate);
    // 0.30 has no exact `f32`: 0.30 × 400 comes out a few millionths over 120.
    assert!(
        (harness.get_sweep_width() - SEGMENT).abs() < 0.001,
        "the segment is 30 % of the track"
    );
    let sweep_y = harness.get_sweep_y();
    let mut looks = Vec::new();
    let mut whole = 0;
    for _ in 0..LOOKS {
        headless.wait(STEP);
        let x = harness.get_sweep_x();
        let row = lit(&headless, sweep_y, ground);
        let count = row.iter().filter(|on| **on).count() as f32;
        assert!(count <= SEGMENT + 1.0, "never more than the segment is lit");
        if x >= 1.0 && x + SEGMENT <= BAR - 1.0 {
            assert!(
                (count - SEGMENT).abs() <= 1.0,
                "inside the track, the whole segment shows"
            );
            whole += 1;
        }
        looks.push(row);
    }
    assert!(
        looks.windows(2).all(|pair| pair[0] != pair[1]),
        "the segment moves at every look"
    );
    assert!(whole > 0, "the segment was seen wholly inside the track");

    // Reduced motion: the fill reaches a new value at once, and the segment stays still over the start.
    harness.set_reduced(true);
    settle(&mut headless);
    harness.set_progress_value(100.0);
    settle(&mut headless);
    assert_eq!(harness.get_progress_fill(), 200.0, "100 of 200, at once");
    headless.wait(STEP);
    assert_eq!(
        harness.get_sweep_x(),
        0.0,
        "the still segment sits at the start"
    );
    let still = lit(&headless, sweep_y, ground);
    assert!(
        still[0] && still[SEGMENT as usize - 2],
        "it covers the first 30 %"
    );
    assert!(!still[SEGMENT as usize + 2], "and nothing past it");
    headless.wait(STEP * 3);
    assert_eq!(lit(&headless, sweep_y, ground), still, "it does not move");
    harness.set_reduced(false);
    settle(&mut headless);

    // UsageBar: 6 px (normal) and 4 px (small), the same in the compact density.
    assert_eq!(harness.get_used_height(), USAGE, "a drive card's usage bar");
    assert_eq!(
        harness.get_picker_height(),
        USAGE_SMALL,
        "the drive picker's usage bar"
    );
    harness.global::<Size>().set_compact(true);
    settle(&mut headless);
    assert_eq!(harness.get_used_height(), USAGE, "the same in compact");
    assert_eq!(
        harness.get_picker_height(),
        USAGE_SMALL,
        "the same in compact"
    );
    harness.global::<Size>().set_compact(false);
    settle(&mut headless);

    // 50 of 100 fills half, `bar-fill` on `bar-track`; a new value is drawn at once (the data never moves, § 3.10).
    assert_eq!(harness.get_used_fill(), BAR / 2.0, "half the drive");
    assert_eq!(
        harness.get_picker_fill(),
        BAR / 2.0,
        "half the drive, small"
    );
    assert_eq!(
        harness.get_used_said(),
        "50",
        "a screen reader says the value"
    );
    let used_y = (harness.get_used_y() + USAGE / 2.0) as usize;
    assert_eq!(
        headless.pixel(INSET as usize + 100, used_y),
        BAR_FILL,
        "the fill"
    );
    assert_eq!(
        headless.pixel(INSET as usize + 300, used_y),
        BAR_TRACK,
        "the track"
    );
    harness.set_usage_value(25.0);
    settle(&mut headless);
    assert_eq!(harness.get_used_fill(), BAR / 4.0, "drawn at once");
    assert!(!harness.get_used_warning(), "a quarter full does not warn");

    // `warning` only above 90 %: exactly 90 stays `bar-fill`, 91 turns `warning`.
    assert!(!harness.get_edge_warning(), "exactly 90 % does not warn");
    assert!(harness.get_full_warning(), "above 90 % warns");
    let edge_y = (harness.get_edge_y() + USAGE / 2.0) as usize;
    let full_y = (harness.get_full_y() + USAGE / 2.0) as usize;
    assert_eq!(
        headless.pixel(INSET as usize + 100, edge_y),
        BAR_FILL,
        "90 %: `bar-fill`"
    );
    assert_eq!(
        headless.pixel(INSET as usize + 100, full_y),
        WARNING,
        "91 %: `warning`"
    );
}
