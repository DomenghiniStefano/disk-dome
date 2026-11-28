//! The Tabs off screen (`docs/design-system.md` § 5.7), on its test window `KitTabsHarness`: the bar is `tabbar` tall
//! and the current tab carries the 2 px `accent` underline at the bar's bottom; ← → skip a disabled tab and go round
//! the ends, Home and End reach them, ↑ ↓ are not the tabs'; with `manual` activation the arrows move the focus and
//! Enter chooses; a click on a disabled tab chooses nothing; long labels in a narrow box give way and say their full
//! text in the tooltip; tabs that do not fit slide to keep the focused one whole in view and list every tab in the
//! «⋯» menu.

#![cfg(feature = "kit")]

use std::time::Duration;

use diskdome_ui::design_system;
use diskdome_ui::generated::KitTabsHarness;
use diskdome_ui::headless::Headless;
use slint::ComponentHandle;
use slint::platform::Key;

const WIDTH: u32 = 960;
const HEIGHT: u32 = 600;
/// `Size.tabbar` at normal density (§ 3.5).
const TABBAR: f32 = 28.0;
/// The underline's height (§ 5.7).
const INDICATOR: f32 = 2.0;
/// `Size.tab-pad-x`: the underline starts after it.
const PAD_X: f32 = 10.0;
/// Inside the first tab's underline, which runs from `PAD_X` to the tab's width less `PAD_X`.
const FIRST_TAB: f32 = PAD_X + 2.0;
/// The «⋯» button: a small icon-only button is square, `Size.control-sm` (24 at normal density), and stands
/// `Size.tool-gap` (4) after the tabs.
const MORE: f32 = 24.0;
const MORE_GAP: f32 = 4.0;
/// Past `Motion.tooltip-delay` (500 ms), with a margin for a busy CI machine.
const AFTER_DELAY: Duration = Duration::from_millis(700);

/// Two turns of the loop: a timer or a change handler is seen in the next one.
fn settle(headless: &mut Headless) {
    headless.frame();
    headless.frame();
}

/// The colour as the frame holds it (opaque, so premultiplied is plain).
fn rgba(colour: slint::Color) -> [u8; 4] {
    [colour.red(), colour.green(), colour.blue(), colour.alpha()]
}

/// One test: the headless platform is installed once per thread.
#[test]
fn tabs_keyboard_pointer_overflow_and_underline() {
    let mut headless = Headless::install(WIDTH, HEIGHT).expect("headless platform");
    let harness = KitTabsHarness::new().expect("the harness compiles");
    design_system::install(&harness);
    harness.show().expect("the harness shows off screen");
    settle(&mut headless);

    assert_eq!(
        harness.get_text_height(),
        TABBAR,
        "the bar is `tabbar` tall"
    );

    // The underline: `accent`, the bar's last 2 px, under the current tab only.
    let accent = rgba(harness.get_accent());
    let bottom = harness.get_text_y() + TABBAR - 1.0;
    let underline = |headless: &Headless| headless.pixel(FIRST_TAB as usize, bottom as usize);
    assert_eq!(
        underline(&headless),
        accent,
        "the current tab is underlined"
    );
    let above = harness.get_text_y() + TABBAR - INDICATOR - 1.0;
    assert_ne!(
        headless.pixel(FIRST_TAB as usize, above as usize),
        accent,
        "the underline is 2 px tall"
    );

    // The keyboard: Overview, Explore, Clean-up (disabled), Timeline.
    headless.click(FIRST_TAB, harness.get_text_y() + TABBAR / 2.0);
    assert_eq!(harness.get_text_current_index(), 0);
    headless.press_key(Key::RightArrow);
    assert_eq!(
        harness.get_text_current_index(),
        1,
        "→ chooses the next tab"
    );
    settle(&mut headless);
    assert_ne!(underline(&headless), accent, "the underline moves with it");
    headless.press_key(Key::RightArrow);
    assert_eq!(
        harness.get_text_current_index(),
        3,
        "→ skips the disabled tab"
    );
    headless.press_key(Key::RightArrow);
    assert_eq!(
        harness.get_text_current_index(),
        0,
        "→ on the last goes round"
    );
    headless.press_key(Key::LeftArrow);
    assert_eq!(
        harness.get_text_current_index(),
        3,
        "← on the first goes round"
    );
    headless.press_key(Key::LeftArrow);
    assert_eq!(harness.get_text_current_index(), 1, "← skips it too");
    headless.press_key(Key::Home);
    assert_eq!(harness.get_text_current_index(), 0);
    headless.press_key(Key::End);
    assert_eq!(harness.get_text_current_index(), 3);
    headless.press_key(Key::DownArrow);
    headless.press_key(Key::UpArrow);
    assert_eq!(
        harness.get_text_current_index(),
        3,
        "↑ ↓ are not the tabs' (a horizontal tab list, as Radix)"
    );
    assert_eq!(harness.get_text_selections(), 7);

    // Manual activation: Off (disabled), General (current), Appearance. A click on the disabled tab chooses nothing.
    let manual_y = harness.get_manual_y() + TABBAR / 2.0;
    headless.click(FIRST_TAB, manual_y);
    settle(&mut headless);
    assert_eq!(
        harness.get_manual_current_index(),
        1,
        "a disabled tab is not chosen"
    );
    headless.press_key(Key::Tab);
    settle(&mut headless);
    // Tab from the text tabs lands on these, on the current tab.
    assert_eq!(harness.get_manual_focus_index(), 1);
    headless.press_key(Key::RightArrow);
    assert_eq!(harness.get_manual_focus_index(), 2, "→ moves the focus");
    assert_eq!(
        harness.get_manual_current_index(),
        1,
        "and does not choose (manual)"
    );
    headless.press_key(Key::RightArrow);
    assert_eq!(
        harness.get_manual_focus_index(),
        1,
        "round the end, skipping the disabled first tab"
    );
    headless.press_key(Key::End);
    headless.press_key(Key::Return);
    assert_eq!(harness.get_manual_current_index(), 2, "Enter chooses");

    // Long labels give way to their box, and a cut one says its full text.
    assert_eq!(
        harness.get_long_width(),
        harness.get_narrow_box(),
        "long labels give way to their box"
    );
    headless.hover(FIRST_TAB, harness.get_long_y() + TABBAR / 2.0);
    settle(&mut headless);
    std::thread::sleep(AFTER_DELAY);
    settle(&mut headless);
    assert_eq!(
        harness.get_long_tooltip(),
        0,
        "a cut label says its full text"
    );

    // Tabs that do not fit: the «⋯» shows, and the last tab slides whole into view.
    let menu_y = harness.get_menu_y() + TABBAR / 2.0;
    assert_eq!(harness.get_menu_width(), harness.get_menu_box());
    assert!(harness.get_menu_preferred() > harness.get_menu_box());
    assert!(harness.get_menu_overflowing(), "the menu shows");
    assert_eq!(harness.get_menu_shift(), 0.0);
    headless.click(FIRST_TAB, menu_y);
    headless.press_key(Key::End);
    settle(&mut headless);
    assert_eq!(harness.get_menu_current_index(), 5);
    let strip = harness.get_menu_box() - MORE - MORE_GAP;
    let whole_in_view = harness.get_menu_preferred() - strip;
    assert_eq!(
        harness.get_menu_shift(),
        whole_in_view,
        "the last tab slides whole into view"
    );
    headless.press_key(Key::Home);
    settle(&mut headless);
    assert_eq!(harness.get_menu_shift(), 0.0, "and back for the first");

    // The «⋯» menu lists every tab: its last item chooses the last tab.
    headless.click(harness.get_menu_box() - MORE / 2.0, menu_y);
    settle(&mut headless);
    headless.press_key(Key::End);
    headless.press_key(Key::Return);
    settle(&mut headless);
    assert_eq!(
        harness.get_menu_current_index(),
        5,
        "the menu's last item is the last tab"
    );
    assert_eq!(harness.get_menu_shift(), whole_in_view);
}
