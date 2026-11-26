//! The SegmentedControl off screen (`docs/design-system.md` § 5.6), on its test window `KitSegmentedHarness`: the
//! arrow keys, Home and End skip a disabled segment and go round the ends; a click on a disabled segment chooses
//! nothing; an icon-only segment says its text in the tooltip, at once on keyboard focus, and Esc puts it away. Long
//! labels in a narrow box give way and say their full text in the tooltip; the control hugs its segments, or fills
//! its layout's width split equally.

#![cfg(feature = "kit")]

use std::time::Duration;

use diskdome_ui::design_system;
use diskdome_ui::generated::KitSegmentedHarness;
use diskdome_ui::headless::Headless;
use slint::ComponentHandle;
use slint::platform::Key;

const WIDTH: u32 = 960;
const HEIGHT: u32 = 600;
/// `Size.segment-inset`: the frame around the segments.
const INSET: f32 = 2.0;
/// An icon-only segment is square: `Size.control` (28 at normal density) less the inset above and below.
const ICON_SEGMENT: f32 = 28.0 - 2.0 * INSET;
/// Past `Motion.tooltip-delay` (500 ms), with a margin for a busy CI machine.
const AFTER_DELAY: Duration = Duration::from_millis(700);

/// Inside a segment, off its centred label: a pixel of the segment's own fill.
const FILL_PROBE: f32 = 8.0;

/// The centre of an icon-only segment, from the control's top.
fn icon_segment(index: u8) -> f32 {
    INSET + ICON_SEGMENT * f32::from(index) + ICON_SEGMENT / 2.0
}

/// Two turns of the loop: a timer or a change handler is seen in the next one.
fn settle(headless: &mut Headless) {
    headless.frame();
    headless.frame();
}

/// One test: the headless platform is installed once per thread.
#[test]
fn segmented_widths_keyboard_pointer_and_tooltips() {
    let mut headless = Headless::install(WIDTH, HEIGHT).expect("headless platform");
    let harness = KitSegmentedHarness::new().expect("the harness compiles");
    design_system::install(&harness);
    harness.show().expect("the harness shows off screen");
    settle(&mut headless);

    // Widths: long labels give way to their box; hugging stays narrower than the panel; filling takes it all.
    assert_eq!(
        harness.get_long_width(),
        harness.get_narrow_box(),
        "long labels give way to their box"
    );
    assert!(
        harness.get_hug_width() < harness.get_panel(),
        "a hugging control keeps its own width"
    );
    assert_eq!(
        harness.get_fill_width(),
        harness.get_panel(),
        "a filling control takes its layout's width"
    );
    // Filling, the two segments are equal: the chosen first one's fill ends at the middle.
    let half = (harness.get_panel() - 2.0 * INSET) / 2.0;
    let y = harness.get_fill_y() + INSET + 2.0;
    let pixel = |x: f32| headless.pixel(x as usize, y as usize);
    let chosen = pixel(INSET + FILL_PROBE);
    assert_eq!(
        pixel(INSET + half - 2.0),
        chosen,
        "the first segment reaches the middle"
    );
    assert_ne!(pixel(INSET + half + 2.0), chosen, "and stops there");

    // A cut label says its full text in the tooltip.
    headless.hover(INSET + FILL_PROBE, harness.get_long_y() + INSET + 4.0);
    settle(&mut headless);
    std::thread::sleep(AFTER_DELAY);
    settle(&mut headless);
    assert_eq!(
        harness.get_long_tooltip(),
        0,
        "a cut label says its full text"
    );

    // The keyboard on the text segments: Table, Both (disabled), Map, Tree.
    headless.click(INSET + 4.0, harness.get_text_y() + INSET + 4.0);
    assert_eq!(harness.get_text_current(), 0);
    headless.press_key(Key::RightArrow);
    assert_eq!(
        harness.get_text_current(),
        2,
        "→ skips the disabled segment"
    );
    headless.press_key(Key::LeftArrow);
    assert_eq!(harness.get_text_current(), 0, "← skips it too");
    headless.press_key(Key::LeftArrow);
    assert_eq!(harness.get_text_current(), 3, "← on the first goes round");
    headless.press_key(Key::RightArrow);
    assert_eq!(harness.get_text_current(), 0, "→ on the last goes round");
    headless.press_key(Key::End);
    assert_eq!(harness.get_text_current(), 3);
    headless.press_key(Key::Home);
    assert_eq!(harness.get_text_current(), 0);

    // A click on the disabled icon segment chooses nothing.
    let icons_y = harness.get_icons_y() + INSET + 4.0;
    headless.click(icon_segment(1), icons_y);
    settle(&mut headless);
    assert_eq!(harness.get_icons_current(), 0);
    assert_eq!(harness.get_icons_changes(), 0);

    // Resting on an icon-only segment shows its text.
    headless.hover(icon_segment(2), icons_y);
    settle(&mut headless);
    std::thread::sleep(AFTER_DELAY);
    settle(&mut headless);
    assert_eq!(
        harness.get_icons_tooltip(),
        2,
        "an icon-only segment says its text"
    );

    // From the keyboard, the chosen segment's tooltip shows at once; Esc puts it away.
    headless.hover(icon_segment(0), harness.get_icons_y() + 200.0);
    headless.click(icon_segment(0), icons_y);
    headless.hover(icon_segment(0), harness.get_icons_y() + 200.0);
    settle(&mut headless);
    headless.press_key(Key::RightArrow);
    settle(&mut headless);
    assert_eq!(
        harness.get_icons_current(),
        2,
        "→ skips the disabled segment"
    );
    assert_eq!(harness.get_icons_changes(), 1);
    assert_eq!(
        harness.get_icons_tooltip(),
        2,
        "keyboard focus shows the tooltip"
    );
    headless.press_key(Key::Escape);
    settle(&mut headless);
    assert_eq!(harness.get_icons_tooltip(), -1, "Esc puts it away");
}
