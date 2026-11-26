//! The Tooltip off screen (`docs/design-system.md` § 5.17), on its test window `KitTooltipHarness`: the rest delay,
//! the skip delay from one tooltip to the next, keyboard focus showing it at once, Esc and a press putting it away,
//! and a disabled button saying why.
//!
//! `DISKDOME_KIT_SHOTS=<folder>` also saves the tooltips near both corners, for a look.

#![cfg(feature = "kit")]

use std::path::PathBuf;
use std::time::Duration;

use diskdome_ui::design_system;
use diskdome_ui::generated::KitTooltipHarness;
use diskdome_ui::headless::Headless;
use slint::ComponentHandle;
use slint::platform::Key;

const WIDTH: u32 = 960;
const HEIGHT: u32 = 600;
/// Past `Motion.tooltip-delay` (500 ms), with a margin for a busy CI machine.
const AFTER_DELAY: Duration = Duration::from_millis(700);
/// Well inside the delay: nothing may show yet.
const BEFORE_DELAY: Duration = Duration::from_millis(150);
/// Past `Motion.tooltip-skip` (300 ms): the next tooltip waits again.
const COOLED: Duration = Duration::from_millis(500);
/// Inside a button, away from its edges.
const INSET: f32 = 6.0;
/// A point over the background, where nothing reacts.
const NOWHERE: (f32, f32) = (480.0, 300.0);

fn save_shot(headless: &Headless, name: &str) {
    let Some(folder) = std::env::var_os("DISKDOME_KIT_SHOTS") else {
        return;
    };
    let path = PathBuf::from(folder).join(format!("tooltip-{name}.bmp"));
    std::fs::write(&path, headless.bmp()).expect("the shots folder is writable");
}

/// Waits, then lets the timers run and draws.
fn after(headless: &mut Headless, wait: Duration) {
    std::thread::sleep(wait);
    settle(headless);
}

/// Two turns of the loop, as a running event loop would give: a timer or a change handler opens the popup in one,
/// its `is-open` is seen in the next.
fn settle(headless: &mut Headless) {
    headless.frame();
    headless.frame();
}

/// One test: the headless platform is installed once per thread.
#[test]
fn tooltips_open_after_the_delay_and_close_as_they_should() {
    let mut headless = Headless::install(WIDTH, HEIGHT).expect("headless platform");
    let harness = KitTooltipHarness::new().expect("the harness compiles");
    design_system::install(&harness);
    harness.show().expect("the harness shows off screen");
    headless.frame();
    let middle = harness.get_button_height() / 2.0;
    let second = harness.get_second_x() + INSET;

    // The pointer rests on the icon button: nothing at first, its label after the delay.
    headless.hover(INSET, middle);
    after(&mut headless, BEFORE_DELAY);
    assert!(!harness.get_first_shown(), "no tooltip before the delay");
    after(&mut headless, AFTER_DELAY);
    assert!(
        harness.get_first_shown(),
        "the tooltip shows after the delay"
    );
    save_shot(&headless, "icon-button");

    // Straight onto the next button: the first closes and the second opens without waiting.
    headless.hover(second, middle);
    settle(&mut headless);
    assert!(!harness.get_first_shown(), "leaving closes the tooltip");
    assert!(
        harness.get_second_shown(),
        "the next tooltip skips the delay"
    );

    // A press puts it away while the pointer stays.
    headless.click(second, middle);
    settle(&mut headless);
    assert!(!harness.get_second_shown(), "a press closes the tooltip");

    // Away long enough, the delay applies again.
    headless.hover(NOWHERE.0, NOWHERE.1);
    after(&mut headless, COOLED);
    headless.hover(second, middle);
    after(&mut headless, BEFORE_DELAY);
    assert!(
        !harness.get_second_shown(),
        "after a pause the delay applies again"
    );
    headless.hover(NOWHERE.0, NOWHERE.1);
    after(&mut headless, COOLED);

    // Keyboard focus shows it at once; Esc puts it away.
    headless.press_key(Key::Tab);
    settle(&mut headless);
    let focused_first = harness.get_first_shown() || harness.get_second_shown();
    assert!(focused_first, "keyboard focus shows the tooltip at once");
    headless.press_key(Key::Escape);
    settle(&mut headless);
    assert!(
        !harness.get_first_shown() && !harness.get_second_shown(),
        "Esc puts the tooltip away"
    );

    // A disabled button in the far corner says why, above itself and inside the window.
    headless.click(NOWHERE.0, NOWHERE.1);
    after(&mut headless, COOLED);
    headless.hover(harness.get_far_x() + INSET, harness.get_far_y() + middle);
    after(&mut headless, AFTER_DELAY);
    assert!(
        harness.get_far_shown(),
        "a disabled button's tooltip shows its reason"
    );
    save_shot(&headless, "disabled-far-corner");
}
