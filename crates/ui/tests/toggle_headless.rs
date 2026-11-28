//! The ToggleButton off screen (`docs/design-system.md` § 5.3), on its test window `KitToggleHarness`: the three
//! sizes are square with an icon only, a text toggle hugs its label and an icon widens it, the outline variant draws
//! its edge, a long label gives way to its box and says its full text in the tooltip, and a click, Space and Enter
//! flip it and say so through `toggled`.

#![cfg(feature = "kit")]

use std::time::Duration;

use diskdome_ui::design_system;
use diskdome_ui::generated::KitToggleHarness;
use diskdome_ui::headless::Headless;
use slint::ComponentHandle;
use slint::platform::Key;

const WIDTH: u32 = 960;
const HEIGHT: u32 = 600;
/// `Size.control-sm`, `Size.control`, `Size.control-lg` at normal density (§ 3.4).
const SMALL: f32 = 24.0;
const NORMAL: f32 = 28.0;
const LARGE: f32 = 32.0;
/// A leading icon adds the icon (16 at normal density) and `label-gap` (6), and takes the 2 px the padding loses on
/// the side with an icon (`button-pad-x` 10 → `button-pad-x-icon` 8).
const ICON_LEAD: f32 = 16.0 + 6.0 - 2.0;
/// Past `Motion.tooltip-delay` (500 ms), with a margin for a busy CI machine.
const AFTER_DELAY: Duration = Duration::from_millis(700);

/// Two turns of the loop: a timer or a change handler is seen in the next one.
fn settle(headless: &mut Headless) {
    headless.frame();
    headless.frame();
}

/// One test: the headless platform is installed once per thread.
#[test]
fn toggle_sizes_content_edge_overflow_and_flips() {
    let mut headless = Headless::install(WIDTH, HEIGHT).expect("headless platform");
    let harness = KitToggleHarness::new().expect("the harness compiles");
    design_system::install(&harness);
    harness.show().expect("the harness shows off screen");
    settle(&mut headless);

    // Sizes: icon only, square.
    for (name, width, height, side) in [
        (
            "small",
            harness.get_small_width(),
            harness.get_small_height(),
            SMALL,
        ),
        (
            "normal",
            harness.get_normal_width(),
            harness.get_normal_height(),
            NORMAL,
        ),
        (
            "large",
            harness.get_large_width(),
            harness.get_large_height(),
            LARGE,
        ),
    ] {
        assert_eq!(
            (width, height),
            (side, side),
            "{name}: an icon-only toggle is square"
        );
    }

    // Content: a text toggle hugs its label, wider than square; an icon widens it by the icon and its gap.
    assert!(
        harness.get_text_width() > NORMAL,
        "a text toggle hugs its label"
    );
    assert_eq!(
        harness.get_icon_text_width() - harness.get_text_width(),
        ICON_LEAD,
        "a leading icon"
    );

    // Outline: its edge differs from the ground inside it, which has no fill at rest; a ghost toggle has no edge.
    let y = (harness.get_outline_y() + NORMAL / 2.0) as usize;
    let edge = harness.get_outline_x() as usize;
    assert_ne!(
        headless.pixel(edge, y),
        headless.pixel(edge + 4, y),
        "the outline toggle draws its edge"
    );
    let ghost = harness.get_text_x() as usize;
    let ghost_y = (harness.get_text_y() + NORMAL / 2.0) as usize;
    assert_eq!(
        headless.pixel(ghost, ghost_y),
        headless.pixel(ghost + 4, ghost_y),
        "a ghost toggle has no edge"
    );

    // Overflow: the long label gives way to its box, and its full text is in the tooltip.
    assert_eq!(
        harness.get_long_width(),
        harness.get_narrow_box(),
        "a long label gives way to its box"
    );
    headless.hover(10.0, harness.get_long_y() + NORMAL / 2.0);
    settle(&mut headless);
    headless.wait(AFTER_DELAY);
    settle(&mut headless);
    assert!(
        harness.get_long_tooltip(),
        "a cut label says its full text in the tooltip"
    );

    // Flips: a click turns it on, Space off, Enter on again; `toggled` says each new state.
    let flip_y = harness.get_flip_y() + NORMAL / 2.0;
    headless.click(10.0, flip_y);
    settle(&mut headless);
    assert!(harness.get_flip_checked(), "a click turns it on");
    assert_eq!(harness.get_flips(), 1, "a click says `toggled`");
    assert!(
        harness.get_last_toggled(),
        "`toggled` carries the new state"
    );

    headless.press_key(Key::Space);
    settle(&mut headless);
    assert!(!harness.get_flip_checked(), "Space turns it off");
    assert!(
        !harness.get_last_toggled(),
        "`toggled` carries the new state"
    );

    headless.press_key(Key::Return);
    settle(&mut headless);
    assert!(harness.get_flip_checked(), "Enter turns it on");
    assert_eq!(harness.get_flips(), 3, "every flip says `toggled` once");
}
