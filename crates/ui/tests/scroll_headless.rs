//! The ScrollBar off screen (`docs/design-system.md` § 5.24), on its test window `KitScrollHarness`: the arrows step
//! and stop at the ends; a press on the track pages towards the pointer and, held, repeats until the thumb reaches it;
//! Shift+click and `jump` put the thumb under the pointer; the thumb drags; the wheel over a bar scrolls along its
//! axis; a change from the screen is not reported as the user's. The default bar hides at rest, shows when the content
//! moves and fades after the delay; `on-hover` arrows appear under the pointer; a disabled bar does nothing; an
//! `always` bar over content that fits shows its track and no thumb.

#![cfg(feature = "kit")]

use std::time::Duration;

use diskdome_ui::design_system;
use diskdome_ui::generated::KitScrollHarness;
use diskdome_ui::headless::Headless;
use slint::ComponentHandle;
use slint::platform::Key;

const WIDTH: u32 = 960;
const HEIGHT: u32 = 600;

// The harness's geometry, from the tokens it is built with (normal density).
/// `space-48`: the bars' first column and row.
const MARGIN: f32 = 48.0;
/// The lower row: `space-48` × 6.
const LOWER: f32 = 288.0;
/// A vertical bar's length, `dialog-sm` / 2; its content is five times as long.
const LENGTH: f32 = 200.0;
/// The horizontal bar's length, `dialog-sm` × 3 / 4; its content is three times as long.
const WIDE: f32 = 300.0;
/// `scrollbar`: the bar's thickness, and the side of each arrow's square.
const BAR: f32 = 10.0;
/// `scroll-step`: one `row`.
const STEP: f32 = 22.0;

// The bar with arrows (§ 5.24): its track is the length less two arrows, its thumb the track × visible / content.
const TRACK: f32 = LENGTH - 2.0 * BAR;
const THUMB: f32 = TRACK * LENGTH / (LENGTH * 5.0);
const TRAVEL: f32 = TRACK - THUMB;
const RANGE: f32 = LENGTH * 4.0;

/// Past `repeat-delay` (400 ms) and enough `repeat-interval`s (50 ms) for any page to reach the pointer.
const HOLD: Duration = Duration::from_millis(1500);
/// Past `Motion.base` (140 ms), the fade, with a margin for a busy CI machine.
const AFTER_FADE: Duration = Duration::from_millis(400);
/// Past `scrollbar-linger` (1000 ms) and the fade.
const AFTER_LINGER: Duration = Duration::from_millis(1600);

/// Two turns of the loop: a timer or a change handler is seen in the next one.
fn settle(headless: &mut Headless) {
    headless.frame();
    headless.frame();
}

/// Lets time pass in small turns, so a repeating timer fires at its pace rather than once.
fn wait(headless: &mut Headless, time: Duration) {
    let turn = Duration::from_millis(25);
    for _ in 0..time.as_millis() / turn.as_millis() {
        std::thread::sleep(turn);
        headless.frame();
    }
    settle(headless);
}

/// The window's point at `along` from the top of the bar with arrows, on its centre line.
fn on_arrows_bar(along: f32) -> (f32, f32) {
    (MARGIN + BAR / 2.0, MARGIN + along)
}

/// The offset that puts the thumb's start at `at` along the track (past the first arrow).
fn offset_for_thumb_at(at: f32) -> f32 {
    -RANGE * (at - BAR) / TRAVEL
}

/// One test: the headless platform is installed once per thread.
#[test]
fn scroll_bar_arrows_track_drag_wheel_and_visibility() {
    let mut headless = Headless::install(WIDTH, HEIGHT).expect("headless platform");
    let harness = KitScrollHarness::new().expect("the harness compiles");
    design_system::install(&harness);
    harness.show().expect("the harness shows off screen");
    settle(&mut headless);

    // The geometry the numbers below are computed from.
    assert_eq!(harness.get_margin(), MARGIN);
    assert_eq!(harness.get_lower(), LOWER);
    assert_eq!(harness.get_vertical_length(), LENGTH);
    assert_eq!(harness.get_horizontal_length(), WIDE);
    assert_eq!(harness.get_thickness(), BAR);
    assert_eq!(harness.get_step(), STEP);

    // The arrows step by `step` and stop at the start.
    let (x, y) = on_arrows_bar(LENGTH - BAR / 2.0);
    headless.click(x, y);
    settle(&mut headless);
    assert_eq!(harness.get_arrows_value(), -STEP, "the down arrow steps");
    let (x, y) = on_arrows_bar(BAR / 2.0);
    headless.click(x, y);
    headless.click(x, y);
    settle(&mut headless);
    assert_eq!(
        harness.get_arrows_value(),
        0.0,
        "the up arrow stops at the start"
    );

    // A click on the track below the thumb pages one visible length.
    let (x, y) = on_arrows_bar(150.0);
    headless.click(x, y);
    settle(&mut headless);
    assert_eq!(
        harness.get_arrows_value(),
        -LENGTH,
        "a click on the track pages"
    );

    // Held, it pages again after the delay until the thumb has passed the pointer: from 0, pages end with the
    // thumb at 46–82, 82–118, then 118–154, which covers 120.
    harness.set_arrows_value(0.0);
    settle(&mut headless);
    let (x, y) = on_arrows_bar(120.0);
    headless.press(x, y);
    settle(&mut headless);
    assert_eq!(
        harness.get_arrows_value(),
        -LENGTH,
        "a press pages once at once"
    );
    wait(&mut headless, HOLD);
    assert_eq!(
        harness.get_arrows_value(),
        -3.0 * LENGTH,
        "held, it pages until the thumb reaches the pointer, then stops"
    );
    headless.release(x, y);
    settle(&mut headless);

    // Shift+click jumps: the thumb's centre comes under the pointer.
    harness.set_arrows_value(0.0);
    settle(&mut headless);
    let (x, y) = on_arrows_bar(100.0);
    headless.click_with(Key::Shift, x, y);
    settle(&mut headless);
    let jumped = offset_for_thumb_at(100.0 - THUMB / 2.0);
    assert_eq!(
        harness.get_arrows_value(),
        jumped,
        "Shift+click centres the thumb there"
    );

    // The thumb drags: grabbed at its centre and moved 36 px down, its centre follows the pointer there.
    headless.press(x, y);
    settle(&mut headless);
    let (x, y) = on_arrows_bar(100.0 + 36.0);
    headless.hover(x, y);
    settle(&mut headless);
    headless.release(x, y);
    settle(&mut headless);
    assert_eq!(
        harness.get_arrows_value(),
        offset_for_thumb_at(136.0 - THUMB / 2.0),
        "the thumb follows the pointer"
    );

    // The wheel over the bar scrolls the content.
    let before = harness.get_arrows_value();
    headless.wheel(x, y, -50.0);
    settle(&mut headless);
    assert_eq!(
        harness.get_arrows_value(),
        before - 50.0,
        "the wheel over the bar scrolls"
    );

    // A change from the screen is not the user's: `scrolled` stays as it was.
    let reported = harness.get_arrows_scrolled();
    assert!(reported > 0, "the user's moves were reported");
    harness.set_arrows_value(0.0);
    settle(&mut headless);
    assert_eq!(
        harness.get_arrows_scrolled(),
        reported,
        "a screen's change is not reported"
    );

    // The horizontal bar jumps (`track-click: jump`): a click at 250 of 300 brings the 100 px thumb's centre there,
    // its end at the track's end.
    headless.click(MARGIN * 2.0 + 250.0, MARGIN + BAR / 2.0);
    settle(&mut headless);
    assert_eq!(
        harness.get_wide_value(),
        -2.0 * WIDE,
        "the horizontal bar jumps to the end"
    );
    // A plain vertical wheel over a horizontal bar scrolls it.
    headless.wheel(MARGIN * 2.0 + 150.0, MARGIN + BAR / 2.0, 60.0);
    settle(&mut headless);
    assert_eq!(
        harness.get_wide_value(),
        -2.0 * WIDE + 60.0,
        "the wheel scrolls a horizontal bar"
    );

    // A disabled bar takes no click.
    headless.click(MARGIN * 3.0 + BAR / 2.0, LOWER + 150.0);
    settle(&mut headless);
    assert_eq!(
        harness.get_disabled_value(),
        0.0,
        "a disabled bar does nothing"
    );

    // Pixels: the window's ground, away from every bar.
    headless.hover(MARGIN / 2.0, (HEIGHT - 10) as f32);
    wait(&mut headless, AFTER_LINGER);
    let ground = headless.pixel(MARGIN as usize / 2, HEIGHT as usize - 10);

    // The default bar is hidden at rest; it shows when the content moves, and fades `scrollbar-linger` later.
    // Its thumb is 40 px (200 × 200 / 1000); 50 px down a 800 px range it starts at 10.
    let rest =
        |headless: &Headless| headless.pixel((MARGIN + BAR / 2.0) as usize, LOWER as usize + 30);
    assert_eq!(rest(&headless), ground, "the default bar is hidden at rest");
    harness.set_rest_value(-50.0);
    wait(&mut headless, AFTER_FADE);
    assert_ne!(rest(&headless), ground, "it shows when the content moves");
    wait(&mut headless, AFTER_LINGER);
    assert_eq!(rest(&headless), ground, "and fades after the delay");

    // `on-hover` arrows are hidden at rest and appear under the pointer.
    let arrow = |headless: &Headless| {
        headless.pixel(
            (MARGIN * 2.0 + BAR / 2.0) as usize,
            (LOWER + LENGTH - BAR / 2.0) as usize,
        )
    };
    assert_eq!(arrow(&headless), ground, "on-hover arrows hide at rest");
    headless.hover(MARGIN * 2.0 + BAR / 2.0, LOWER + LENGTH / 2.0);
    wait(&mut headless, AFTER_FADE);
    assert_ne!(arrow(&headless), ground, "and appear under the pointer");

    // An `always` bar over content that fits: its track shows, with no thumb on it.
    let track = |along: f32| {
        headless.pixel(
            (MARGIN * 4.0 + BAR / 2.0) as usize,
            (LOWER + along) as usize,
        )
    };
    assert_ne!(track(LENGTH / 2.0), ground, "an always bar shows its track");
    assert_eq!(track(LENGTH / 2.0), track(BAR * 3.0), "and no thumb on it");
}
