//! The Button off screen (`docs/design-system.md` § 5.2), on its test window `KitButtonHarness`: the minimum width,
//! a label too long for its box (cut, with its full text in the tooltip), the full width, the trailing icon.

#![cfg(feature = "kit")]

use std::time::Duration;

use diskdome_ui::design_system;
use diskdome_ui::generated::KitButtonHarness;
use diskdome_ui::headless::Headless;
use slint::ComponentHandle;

const WIDTH: u32 = 960;
const HEIGHT: u32 = 600;
/// `Size.button-min-w`.
const MIN_WIDTH: f32 = 64.0;
/// The trailing icon adds the icon (16 at normal density), `label-gap` (6), and takes the 2 px the padding loses on
/// the side with an icon (`button-pad-x` 10 → `button-pad-x-icon` 8).
const ICON_END: f32 = 16.0 + 6.0 - 2.0;
/// Longer than `Motion.tooltip-delay`, 500 ms, with a margin.
const TOOLTIP_WAIT: Duration = Duration::from_millis(700);

/// One test: the headless platform is installed once per thread.
#[test]
fn button_widths_and_overflow() {
    let mut headless = Headless::install(WIDTH, HEIGHT).expect("headless platform");
    let harness = KitButtonHarness::new().expect("the harness compiles");
    design_system::install(&harness);
    harness.show().expect("the harness shows off screen");
    headless.frame();

    assert_eq!(
        harness.get_short_width(),
        MIN_WIDTH,
        "a short label keeps the minimum width"
    );
    assert_eq!(
        harness.get_long_width(),
        harness.get_narrow_box(),
        "a long label gives way to its box"
    );
    assert_eq!(
        harness.get_fill_width(),
        harness.get_fill_box(),
        "a full-width button takes its layout's width"
    );
    assert_eq!(
        harness.get_end_width() - harness.get_plain_width(),
        ICON_END,
        "the trailing icon"
    );

    // The cut label's full text is in the tooltip.
    let y = harness.get_long_y() + 4.0;
    headless.hover(10.0, y);
    headless.frame();
    std::thread::sleep(TOOLTIP_WAIT);
    headless.frame();
    headless.frame();
    assert!(
        harness.get_long_tooltip(),
        "a cut label says its full text in the tooltip"
    );
}
