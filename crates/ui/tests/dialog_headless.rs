//! The Dialog off screen (`docs/design-system.md` § 5.18, § 6.4), on its test window `KitDialogHarness`: it opens
//! centred at its size over a scrim that stops the pointer, the focus goes to the default action, Tab and Shift+Tab
//! go round inside it, Enter from the check box chooses the default action, Esc the cancel one, a disabled default
//! leaves Enter without effect and the focus on the first control, the focus goes back to the opening button when it
//! closes, and a body taller than the window scrolls inside a dialog kept `dialog-margin` from the edges.

#![cfg(feature = "kit")]

use std::time::Duration;

use diskdome_ui::design_system;
use diskdome_ui::generated::KitDialogHarness;
use diskdome_ui::headless::Headless;
use slint::ComponentHandle;
use slint::platform::Key;

const WIDTH: u32 = 960;
const HEIGHT: u32 = 600;
/// `Size.dialog-md` and `Size.dialog-sm` (§ 3.5).
const MEDIUM: f32 = 520.0;
const SMALL: f32 = 400.0;
/// `Size.dialog-margin` (pending design, 16).
const MARGIN: f32 = 16.0;
/// Centred: (960 − 520) / 2.
const MEDIUM_X: f32 = 220.0;
/// A body taller than the window: 600 − 2 × 16.
const TALL_HEIGHT: f32 = 568.0;
/// Past the opening (`Motion.slow`, 200 ms) and the closing (`Motion.fast`, 90 ms) fades, with a margin for a busy CI
/// machine.
const AFTER_FADE: Duration = Duration::from_millis(500);
/// A point of the window's ground away from every control: the top edge, mid-width.
const GROUND: (usize, usize) = (480, 4);
/// The default action and the cancel one (the harness's `actions`); -1 would be a close without an action.
const DEFAULT: i32 = 0;
const CANCEL: i32 = 1;

/// Two turns of the loop: a timer or a change handler is seen in the next one.
fn settle(headless: &mut Headless) {
    headless.frame();
    headless.frame();
}

fn key(headless: &mut Headless, key: Key) {
    headless.press_key(key);
    settle(headless);
}

fn back_tab(headless: &mut Headless) {
    headless.press_key_with(Key::Shift, Key::Tab);
    settle(headless);
}

/// Opens the dialog with Space on the focused opening button, and lets it fade in.
fn reopen_by_keyboard(headless: &mut Headless, harness: &KitDialogHarness) {
    key(headless, Key::Space);
    headless.wait(AFTER_FADE);
    settle(headless);
    assert!(
        harness.get_open(),
        "the focus came back to the opening button, and Space on it opens the dialog again"
    );
}

/// One test: the headless platform is installed once per thread.
#[test]
fn dialog_opens_traps_the_focus_takes_enter_and_esc_and_gives_the_focus_back() {
    let mut headless = Headless::install(WIDTH, HEIGHT).expect("headless platform");
    let harness = KitDialogHarness::new().expect("the harness compiles");
    design_system::install(&harness);
    harness.show().expect("the harness shows off screen");
    settle(&mut headless);
    let ground = headless.pixel(GROUND.0, GROUND.1);

    // Opens: a click on the button, the dialog centred at its size.
    headless.click(10.0, harness.get_trigger_height() / 2.0);
    settle(&mut headless);
    headless.wait(AFTER_FADE);
    settle(&mut headless);
    assert!(harness.get_open(), "a click on the button opens it");
    assert_eq!(harness.get_panel_width(), MEDIUM, "medium is 520 wide");
    assert_eq!(harness.get_panel_x(), MEDIUM_X, "centred across");
    let (panel_y, panel_height) = (harness.get_panel_y(), harness.get_panel_height());
    // Centred down, on a whole pixel: the body's text may make the height fractional.
    assert!(
        (panel_y - (HEIGHT as f32 - panel_height) / 2.0).abs() <= 0.5,
        "centred down"
    );
    assert_eq!(panel_y, panel_y.round(), "on a whole pixel");

    // The scrim darkens the window around it; the surface differs from the scrim (the left padding, mid-height).
    let scrim = headless.pixel(GROUND.0, GROUND.1);
    assert_ne!(scrim, ground, "the scrim covers the window");
    let surface = headless.pixel(
        (harness.get_panel_x() + 8.0) as usize,
        (panel_y + panel_height / 2.0) as usize,
    );
    assert_ne!(surface, scrim, "the surface is drawn over the scrim");

    // The pointer never reaches what is behind, and a click on the scrim does not close it (modal).
    headless.click(
        10.0,
        harness.get_behind_y() + harness.get_trigger_height() / 2.0,
    );
    settle(&mut headless);
    assert_eq!(
        harness.get_behind_clicks(),
        0,
        "the scrim stops the pointer"
    );
    assert!(harness.get_open(), "a click on the scrim leaves it open");

    // The focus went to the default action: two Tabs pass «Cancel», go round the end and reach the check box, the
    // first control; Space ticks it. Escaped to the window, the second Tab would land on the buttons behind.
    key(&mut headless, Key::Tab);
    key(&mut headless, Key::Tab);
    key(&mut headless, Key::Space);
    assert!(
        harness.get_box_checked(),
        "Tab goes round inside the dialog, to its first control"
    );

    // Enter on the check box (which does not take it) chooses the default action, and the dialog closes.
    key(&mut headless, Key::Return);
    assert_eq!(harness.get_activations(), 1, "Enter chooses one action");
    assert_eq!(
        harness.get_last_index(),
        DEFAULT,
        "Enter from anywhere in the dialog chooses the default action"
    );
    assert!(!harness.get_open(), "an action closes the dialog");
    assert_eq!(harness.get_closings(), 1, "`closed` says it closed");
    headless.wait(AFTER_FADE);
    settle(&mut headless);
    assert_eq!(
        headless.pixel(GROUND.0, GROUND.1),
        ground,
        "the scrim fades away"
    );

    // Shift+Tab from the default action reaches the check box, then goes round the start to the last action,
    // «Cancel»; Space chooses it.
    reopen_by_keyboard(&mut headless, &harness);
    back_tab(&mut headless);
    back_tab(&mut headless);
    key(&mut headless, Key::Space);
    assert_eq!(
        harness.get_last_index(),
        CANCEL,
        "Shift+Tab goes round inside the dialog, to its last action"
    );
    assert!(!harness.get_open(), "«Cancel» closes the dialog");
    headless.wait(AFTER_FADE);
    settle(&mut headless);

    // Esc chooses the cancel action.
    reopen_by_keyboard(&mut headless, &harness);
    key(&mut headless, Key::Escape);
    assert_eq!(harness.get_activations(), 3, "Esc chooses one action");
    assert_eq!(harness.get_last_index(), CANCEL, "Esc chooses «Cancel»");
    assert!(!harness.get_open(), "Esc closes the dialog");
    headless.wait(AFTER_FADE);
    settle(&mut headless);

    // A disabled default: the focus goes to the first control, the check box, and Enter does nothing.
    harness.set_default_disabled(true);
    settle(&mut headless);
    reopen_by_keyboard(&mut headless, &harness);
    key(&mut headless, Key::Return);
    assert!(
        harness.get_open(),
        "Enter leaves it open: the default is disabled"
    );
    assert_eq!(
        harness.get_activations(),
        3,
        "a disabled default is not chosen"
    );
    key(&mut headless, Key::Space);
    assert!(
        !harness.get_box_checked(),
        "the focus went to the first control: Space unticks the check box"
    );
    key(&mut headless, Key::Escape);
    assert!(!harness.get_open(), "Esc still closes it");
    assert_eq!(harness.get_closings(), 4, "every close says `closed`");
    headless.wait(AFTER_FADE);
    settle(&mut headless);

    // A body taller than the window: the dialog keeps its margin from the edges and its body scrolls.
    harness.set_tall_open(true);
    settle(&mut headless);
    headless.wait(AFTER_FADE);
    settle(&mut headless);
    assert_eq!(harness.get_tall_panel_width(), SMALL, "small is 400 wide");
    assert_eq!(
        harness.get_tall_panel_height(),
        TALL_HEIGHT,
        "a tall dialog stays `dialog-margin` from the top and the bottom"
    );
    assert!(
        harness.get_tall_panel_height() + 2.0 * MARGIN <= HEIGHT as f32,
        "inside the window"
    );
    key(&mut headless, Key::Escape);
    assert!(!harness.get_tall_open(), "Esc closes the tall dialog too");
}
