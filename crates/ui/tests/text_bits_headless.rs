//! Badge, Kbd, Chip and Breadcrumb off screen (`docs/design-system.md` § 5.13–5.15, § 5.26), on their test window
//! `KitTextBitsHarness`: a badge's height and icon, square one-letter key caps and a shortcut's gaps, a filter chip
//! that opens with a click or Enter and goes with its `x`, Delete or Backspace, a toggle chip that flips, a long chip
//! that gives way and says its text, and a path that folds in the middle or from the start as it narrows, cuts its
//! last name, and goes to a segment by pointer and through the «…» menu by keyboard.

#![cfg(feature = "kit")]

use std::time::Duration;

use diskdome_ui::design_system;
use diskdome_ui::generated::KitTextBitsHarness;
use diskdome_ui::headless::Headless;
use slint::ComponentHandle;
use slint::platform::Key;

const WIDTH: u32 = 960;
const HEIGHT: u32 = 600;
/// `Size.control-sm` at normal density: a chip and a breadcrumb segment (§ 5.13, § 5.15).
const CONTROL_SM: f32 = 24.0;
/// `badge-h` (§ 5.14).
const BADGE: f32 = 16.0;
/// A badge's leading icon adds `icon-sm` (12) and `badge-gap` (2).
const BADGE_ICON_LEAD: f32 = 12.0 + 2.0;
/// `kbd-h`: the caption's 16 px line and two 1 px edges (§ 3.3, § 5.26); a one-letter key is as wide.
const KEY: f32 = 16.0 + 2.0 * 1.0;
/// `kbd-gap` between the caps of a shortcut.
const KEY_GAP: f32 = 4.0;
/// The `x` adds `label-gap` (6), its 16 px hit area and the inset that centres it on the chip's end
/// ((24 − 16) / 2 = 4), in place of the right `chip-pad-x` (8).
const REMOVE_LEAD: f32 = 6.0 + 16.0 + 4.0 - 8.0;
/// The `x`'s centre from the chip's right edge: the inset and half the hit area.
const REMOVE_CENTRE: f32 = 4.0 + 8.0;
/// A swatch adds `chip-swatch` (8) and `label-gap` (6).
const SWATCH_LEAD: f32 = 8.0 + 6.0;
/// Six segments: the whole path at the window's width, folded at a narrower one, and the last name alone and cut.
const SEGMENTS: i32 = 6;
const NARROW: f32 = 220.0;
const TINY: f32 = 60.0;
/// Past `Motion.tooltip-delay` (500 ms), with a margin for a busy CI machine.
const AFTER_DELAY: Duration = Duration::from_millis(700);
/// Longer than `Motion.base` (140 ms), the menu's fade.
const FADE: Duration = Duration::from_millis(250);

/// Two turns of the loop: a timer or a change handler is seen in the next one.
fn settle(headless: &mut Headless) {
    headless.frame();
    headless.frame();
}

/// One test: the headless platform is installed once per thread.
#[test]
fn badge_kbd_chip_and_breadcrumb() {
    let mut headless = Headless::install(WIDTH, HEIGHT).expect("headless platform");
    let harness = KitTextBitsHarness::new().expect("the harness compiles");
    design_system::install(&harness);
    harness.show().expect("the harness shows off screen");
    settle(&mut headless);

    // Badge: its height, and a leading icon.
    assert_eq!(
        harness.get_badge_height(),
        BADGE,
        "a badge is `badge-h` tall"
    );
    assert_eq!(
        harness.get_badge_icon_width() - harness.get_badge_width(),
        BADGE_ICON_LEAD,
        "a badge's icon"
    );

    // Kbd: one letter is square, a long name widens the cap, a shortcut's caps sit `kbd-gap` apart.
    assert_eq!(
        (harness.get_key_width(), harness.get_key_height()),
        (KEY, KEY),
        "a one-letter key cap is square"
    );
    assert!(
        harness.get_long_key_width() > KEY,
        "a long key name widens its cap"
    );
    assert_eq!(
        harness.get_group_width(),
        harness.get_ctrl_width() + harness.get_shift_width() + KEY + 2.0 * KEY_GAP,
        "a shortcut's caps"
    );

    // Chip: the `x` and the swatch, then what each part does.
    assert_eq!(
        harness.get_filter_height(),
        CONTROL_SM,
        "a chip is `control-sm` tall"
    );
    assert_eq!(
        harness.get_filter_width() - harness.get_toggle_width(),
        REMOVE_LEAD,
        "a filter chip's `x`; a toggle chip has none"
    );
    assert_eq!(
        harness.get_swatch_width() - harness.get_toggle_width(),
        SWATCH_LEAD,
        "a toggle chip's swatch"
    );
    let chip_y = harness.get_chips_y() + CONTROL_SM / 2.0;
    headless.click(harness.get_filter_x() + 4.0, chip_y);
    settle(&mut headless);
    assert_eq!(harness.get_clicks(), 1, "a click on a filter chip opens it");
    headless.click(
        harness.get_filter_x() + harness.get_filter_width() - REMOVE_CENTRE,
        chip_y,
    );
    settle(&mut headless);
    assert_eq!(harness.get_removals(), 1, "the `x` removes the chip");
    assert_eq!(harness.get_clicks(), 1, "the `x` does not open the filter");
    // The click gave the chip the focus; the `x` did not take it.
    headless.press_key(Key::Delete);
    settle(&mut headless);
    assert_eq!(harness.get_removals(), 2, "Delete removes a focused chip");
    headless.press_key(Key::Backspace);
    settle(&mut headless);
    assert_eq!(
        harness.get_removals(),
        3,
        "Backspace removes a focused chip"
    );
    headless.press_key(Key::Return);
    settle(&mut headless);
    assert_eq!(harness.get_clicks(), 2, "Enter opens a focused filter chip");

    headless.click(harness.get_toggle_x() + 4.0, chip_y);
    settle(&mut headless);
    assert!(
        harness.get_toggle_checked(),
        "a click turns a toggle chip on"
    );
    assert!(
        harness.get_last_toggled(),
        "`toggled` carries the new state"
    );
    headless.press_key(Key::Space);
    settle(&mut headless);
    assert!(!harness.get_toggle_checked(), "Space turns it off");
    headless.press_key(Key::Delete);
    settle(&mut headless);
    assert_eq!(harness.get_flips(), 2, "every flip says `toggled` once");
    assert_eq!(harness.get_removals(), 3, "a toggle chip is not removable");

    // A long chip gives way to its box, cuts its text and says it whole in the tooltip.
    assert_eq!(
        harness.get_long_width(),
        harness.get_narrow_box(),
        "a long chip gives way to its box"
    );
    assert!(harness.get_long_cut(), "its text is cut");
    headless.hover(4.0, harness.get_long_y() + CONTROL_SM / 2.0);
    settle(&mut headless);
    headless.wait(AFTER_DELAY);
    settle(&mut headless);
    assert!(
        harness.get_long_tooltip(),
        "a cut chip says its text in the tooltip"
    );

    // Breadcrumb: the whole path fits the window's width.
    assert_eq!(
        harness.get_crumb_height(),
        CONTROL_SM,
        "a breadcrumb is `control-sm` tall"
    );
    assert_eq!(harness.get_middle_first(), 0, "the whole path shows");
    assert!(!harness.get_middle_collapsed(), "nothing is folded");
    assert!(!harness.get_start_collapsed(), "nothing is folded");

    // Narrower, it folds: in the middle after the first segment, or from the start; the last one stays.
    harness.set_crumb_width(NARROW);
    settle(&mut headless);
    let first = harness.get_middle_first();
    assert!(harness.get_middle_collapsed(), "a narrow path folds");
    assert!(
        (2..SEGMENTS).contains(&first),
        "the middle folds after the first segment, the last stays: {first}"
    );
    assert!(harness.get_start_collapsed(), "a narrow path folds");
    assert!(
        (1..SEGMENTS).contains(&harness.get_start_first()),
        "the start folds, the last stays"
    );
    assert!(
        harness.get_start_first() <= first,
        "without the first segment, as many or more of the last fit"
    );
    assert!(!harness.get_middle_last_cut(), "the last name fits");

    // A click on the first segment goes there.
    headless.click(4.0, CONTROL_SM / 2.0);
    settle(&mut headless);
    assert_eq!(
        (harness.get_crumb_index(), harness.get_crumb_count()),
        (0, 1),
        "a click on a segment goes there"
    );

    // The keyboard reaches the «…» after the first segment; ↓ opens its menu on the first folded segment, the
    // second of the path, and Enter goes there.
    headless.press_key(Key::Tab);
    settle(&mut headless);
    headless.press_key(Key::DownArrow);
    settle(&mut headless);
    headless.wait(FADE);
    settle(&mut headless);
    headless.press_key(Key::Return);
    settle(&mut headless);
    assert_eq!(
        (harness.get_crumb_index(), harness.get_crumb_count()),
        (1, 2),
        "the «…» menu goes to a folded segment"
    );

    // Tiny: only the last segment stays after the fold, and its name is cut.
    harness.set_crumb_width(TINY);
    settle(&mut headless);
    assert_eq!(
        harness.get_middle_first(),
        SEGMENTS - 1,
        "only the last segment stays"
    );
    assert!(harness.get_middle_last_cut(), "the last name is cut");
}
