//! The choice controls off screen (`docs/design-system.md` § 5.8), on their test window `KitChoiceHarness`: a check
//! box's row and its `line-strong` edge, a click and Space flipping it (Enter does not), the indeterminate box ticked
//! by a click, a long label cut and said whole in the tooltip or wrapped onto more lines, a disabled box that does not
//! flip and says why; a radio group chosen by click and arrows, skipping its disabled item and going round the ends;
//! a switch flipped by click, Space and Enter with its knob moving, and a switch whose label on the left puts it at
//! the end of its row.

#![cfg(feature = "kit")]

use std::time::Duration;

use diskdome_ui::design_system;
use diskdome_ui::generated::{CheckboxState, KitChoiceHarness};
use diskdome_ui::headless::Headless;
use slint::ComponentHandle;
use slint::platform::Key;

const WIDTH: u32 = 960;
const HEIGHT: u32 = 600;
/// `Size.choice-h` (pending design: `control-sm`, 24 at normal density) and the row's centre.
const ROW: f32 = 24.0;
const MID: f32 = ROW / 2.0;
/// The box and the circle (§ 5.8: 14 px).
const BOX: f32 = 14.0;
/// The switch's track and knob (§ 5.8: 28 × 16, knob 12): off, the knob sits (16 − 12) / 2 = 2 from the left; on,
/// 28 − 12 − 2 = 14.
const SWITCH_W: f32 = 28.0;
const KNOB_OFF: f32 = 2.0;
const KNOB_ON: f32 = 14.0;
/// The dark theme's colours (§ 3.2): the box's edge, the `accent` fill, the switch's `control` track.
const LINE_STRONG: [u8; 4] = [0x66, 0x69, 0x6e, 0xff];
const ACCENT: [u8; 4] = [0x37, 0xc2, 0xbb, 0xff];
const CONTROL: [u8; 4] = [0x23, 0x25, 0x28, 0xff];
/// Past `Motion.fast` (90 ms), so colours and the knob have arrived.
const AFTER_FAST: Duration = Duration::from_millis(300);
/// Past `Motion.tooltip-delay` (500 ms), with a margin for a busy CI machine.
const AFTER_DELAY: Duration = Duration::from_millis(700);
/// A point far from every control, where the pointer rests to leave no hover.
const AWAY: (f32, f32) = (900.0, 580.0);

/// Two turns of the loop: a timer or a change handler is seen in the next one.
fn settle(headless: &mut Headless) {
    headless.frame();
    headless.frame();
}

/// Moves the pointer away and lets the colours arrive, so a pixel shows the rest look.
fn rest(headless: &mut Headless) {
    headless.hover(AWAY.0, AWAY.1);
    settle(headless);
    headless.wait(AFTER_FAST);
}

/// One test: the headless platform is installed once per thread.
#[test]
fn checkbox_radio_and_switch_flip_choose_and_say_why() {
    let mut headless = Headless::install(WIDTH, HEIGHT).expect("headless platform");
    let harness = KitChoiceHarness::new().expect("the harness compiles");
    design_system::install(&harness);
    harness.show().expect("the harness shows off screen");
    settle(&mut headless);

    // The row and the box: `choice-h` tall, the box's edge in `line-strong`, nothing inside it at rest.
    assert_eq!(harness.get_check_height(), ROW, "a one-line choice row");
    let check_mid = (harness.get_check_y() + MID) as usize;
    assert_eq!(headless.pixel(0, check_mid), LINE_STRONG, "the box's edge");
    let ground = headless.pixel(AWAY.0 as usize, AWAY.1 as usize);
    assert_eq!(
        headless.pixel(1, check_mid),
        ground,
        "an unchecked box is empty"
    );

    // A click on the label (the whole row is clickable) checks it; the box fills with `accent`.
    headless.click(BOX + 20.0, harness.get_check_y() + MID);
    settle(&mut headless);
    assert_eq!(harness.get_check_state(), CheckboxState::Checked);
    assert_eq!(harness.get_check_flips(), 1, "a click says `toggled`");
    assert_eq!(harness.get_last_state(), CheckboxState::Checked);
    rest(&mut headless);
    assert_eq!(headless.pixel(1, check_mid), ACCENT, "a checked box is filled");

    // Space flips it back; Enter does not (it belongs to the dialog's default button, as WAI-ARIA and Radix).
    headless.press_key(Key::Space);
    settle(&mut headless);
    assert_eq!(harness.get_check_state(), CheckboxState::Unchecked);
    assert_eq!(harness.get_last_state(), CheckboxState::Unchecked);
    headless.press_key(Key::Return);
    settle(&mut headless);
    assert_eq!(
        harness.get_check_state(),
        CheckboxState::Unchecked,
        "Enter does not flip a check box"
    );
    assert_eq!(harness.get_check_flips(), 2);

    // A click on an indeterminate box ticks it.
    headless.click(BOX / 2.0, harness.get_mixed_y() + MID);
    settle(&mut headless);
    assert_eq!(harness.get_mixed_state(), CheckboxState::Checked);

    // A long label gives way to its box and says its full text in the tooltip; wrapped, it takes more lines.
    assert_eq!(harness.get_long_width(), harness.get_narrow_box());
    headless.hover(BOX / 2.0, harness.get_long_y() + MID);
    settle(&mut headless);
    headless.wait(AFTER_DELAY);
    settle(&mut headless);
    assert!(
        harness.get_long_tooltip(),
        "a cut label says its text in the tooltip"
    );
    assert!(
        harness.get_wrapped_height() > ROW,
        "a wrapped label grows its row"
    );

    // A disabled box does not flip, and says why on hover.
    headless.click(BOX / 2.0, harness.get_off_y() + MID);
    settle(&mut headless);
    assert_eq!(harness.get_off_state(), CheckboxState::Unchecked);
    headless.hover(BOX / 2.0, harness.get_off_y() + MID);
    settle(&mut headless);
    headless.wait(AFTER_DELAY);
    settle(&mut headless);
    assert!(harness.get_off_tooltip(), "a disabled box says why");

    // The radio group: nothing chosen, the keyboard stop is the first enabled item.
    let item = |index: f32| harness.get_radio_y() + index * ROW + MID;
    assert_eq!(harness.get_radio_index(), -1);
    assert_eq!(harness.get_radio_focus(), 0);
    headless.click(BOX / 2.0, item(2.0));
    settle(&mut headless);
    assert_eq!(harness.get_radio_index(), 2, "a click chooses an item");
    assert_eq!(harness.get_radio_picks(), 1);
    rest(&mut headless);
    assert_eq!(
        headless.pixel(2, item(2.0) as usize),
        ACCENT,
        "the chosen circle is filled"
    );
    // ↓ goes round the end; ↓ again skips the disabled item; ↑ goes back and round.
    headless.press_key(Key::DownArrow);
    settle(&mut headless);
    assert_eq!(harness.get_radio_index(), 0, "the arrows go round the ends");
    headless.press_key(Key::DownArrow);
    settle(&mut headless);
    assert_eq!(
        harness.get_radio_index(),
        2,
        "the arrows skip a disabled item"
    );
    headless.press_key(Key::UpArrow);
    settle(&mut headless);
    assert_eq!(harness.get_radio_index(), 0);
    headless.press_key(Key::LeftArrow);
    settle(&mut headless);
    assert_eq!(harness.get_radio_index(), 2);
    assert_eq!(harness.get_radio_picks(), 5, "every new choice says `selected`");
    // A click on the disabled item chooses nothing.
    headless.click(BOX / 2.0, item(1.0));
    settle(&mut headless);
    assert_eq!(harness.get_radio_index(), 2, "a disabled item is not chosen");

    // The switch: a click turns it on and moves the knob right; Space off, Enter on (a switch is a button).
    assert_eq!(harness.get_knob_x(), KNOB_OFF);
    headless.click(SWITCH_W / 2.0, harness.get_switch_y() + MID);
    settle(&mut headless);
    assert!(harness.get_switch_on(), "a click turns the switch on");
    headless.wait(AFTER_FAST);
    assert_eq!(harness.get_knob_x(), KNOB_ON, "on, the knob is on the right");
    headless.press_key(Key::Space);
    settle(&mut headless);
    assert!(!harness.get_switch_on(), "Space turns it off");
    headless.press_key(Key::Return);
    settle(&mut headless);
    assert!(harness.get_switch_on(), "Enter turns it on");
    assert_eq!(harness.get_switch_flips(), 3);

    // With the label on the left, the switch fills its row and sits at its end: its `control` track is there.
    assert_eq!(harness.get_left_width(), harness.get_wide_box());
    rest(&mut headless);
    let track_x = (harness.get_wide_box() - SWITCH_W / 2.0) as usize;
    assert_eq!(
        headless.pixel(track_x, (harness.get_left_y() + MID) as usize),
        CONTROL,
        "the switch sits at the row's end"
    );
}
