//! The SegmentedControl off screen (`docs/design-system.md` § 5.6), on its test window `KitSegmentedHarness`: the
//! arrow keys, Home and End skip a disabled segment and go round the ends; a click on a disabled segment chooses
//! nothing; an icon-only segment says its text in the tooltip, at once on keyboard focus, and Esc puts it away.

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
fn segmented_keyboard_pointer_and_tooltips() {
    let mut headless = Headless::install(WIDTH, HEIGHT).expect("headless platform");
    let harness = KitSegmentedHarness::new().expect("the harness compiles");
    design_system::install(&harness);
    harness.show().expect("the harness shows off screen");
    settle(&mut headless);

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
