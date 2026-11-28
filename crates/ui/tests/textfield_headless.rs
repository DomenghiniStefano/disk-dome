//! The TextField off screen (`docs/design-system.md` § 5.4), on its test window `KitTextFieldHarness`: the label
//! above a `control`-tall box and the help under it; typing reports each edit and Enter; the edge turns `focus` with
//! the focus and `danger` with an error, whose message adds a line under the field; a click on the label focuses the
//! text; Tab out turns the keyboard flag on and the next field shows its ring; a disabled or read-only field takes
//! no typing; the text starts at the padding, or after the leading icon.
//!
//! `DISKDOME_KIT_SHOTS=<folder>` also saves the window with the error and the ring, for a look.

#![cfg(feature = "kit")]

use std::ops::Range;
use std::path::PathBuf;

use diskdome_ui::design_system;
use diskdome_ui::generated::{InputMode, KitTextFieldHarness};
use diskdome_ui::headless::Headless;
use slint::ComponentHandle;
use slint::platform::Key;

const WIDTH: u32 = 960;
const HEIGHT: u32 = 600;

// From the spec, in the dark theme and the normal density (the window's defaults).
/// `body` line (§ 3.3): the label's height.
const LABEL: f32 = 18.0;
/// `field-gap` (Pending design, 4): between the label, the box and each message.
const GAP: f32 = 4.0;
/// `control` (§ 3.5): the box's height.
const BOX: f32 = 28.0;
/// `caption` line (§ 3.3): one line of help or error.
const MESSAGE: f32 = 16.0;
/// `field-w` (Pending design, 240): the field's width in a row that does not stretch it.
const FIELD_W: f32 = 240.0;
/// `line`, `focus` and `danger` in the dark theme (§ 3.2, § 3.7), as the edge pixel reads them (opaque).
const LINE: [u8; 4] = [0x28, 0x2a, 0x2d, 0xff];
const FOCUS: [u8; 4] = [0x37, 0xc2, 0xbb, 0xff];
const DANGER: [u8; 4] = [0xf6, 0x6d, 0x67, 0xff];
/// The window's ground, `bg` in the dark theme.
const BG: [u8; 4] = [0x0f, 0x11, 0x13, 0xff];
/// The ring is 2 px wide, 1 px outside the edge (§ 3.7): its middle is 2 px left of the box.
const RING_OUT: f32 = 2.0;
/// Over the field's text, inside the box.
const TEXT_X: f32 = 60.0;
/// `field-pad-x`, the leading icon (`Size.icon`, normal density) and `label-gap` after it (§ 5.4).
const PAD_X: usize = 6;
const ICON: usize = 16;
const ICON_GAP: usize = 6;
/// The rows the text is read in, inside the box.
const INK_INSET: f32 = 4.0;
/// Inside the box, in its start padding: the field's ground.
const GROUND_X: usize = 3;
/// How far from the field's start the text is looked for, short of its end.
const READ_W: usize = 160;

/// The first and last inked column of the box at `left`, `top`, within `columns` of it (counted from `left`).
fn ink(
    headless: &Headless,
    left: usize,
    top: f32,
    columns: Range<usize>,
) -> Option<(usize, usize)> {
    let ground = headless.pixel(left + GROUND_X, (top + BOX / 2.0) as usize);
    headless
        .ink_columns(
            left + columns.start..left + columns.end,
            (top + INK_INSET) as usize..(top + BOX - INK_INSET) as usize,
            ground,
        )
        .map(|(first, last)| (first - left, last - left))
}

fn save_shot(headless: &Headless, name: &str) {
    let Some(folder) = std::env::var_os("DISKDOME_KIT_SHOTS") else {
        return;
    };
    let path = PathBuf::from(folder).join(format!("textfield-{name}.bmp"));
    std::fs::write(&path, headless.bmp()).expect("the shots folder is writable");
}

/// Two turns of the loop, and a fresh frame: a change handler is seen in the next one.
fn settle(headless: &mut Headless) {
    headless.frame();
    headless.request_redraw();
    headless.frame();
}

/// One test: the headless platform is installed once per thread.
#[test]
fn textfield_typing_errors_focus_and_disabled() {
    let mut headless = Headless::install(WIDTH, HEIGHT).expect("headless platform");
    let harness = KitTextFieldHarness::new().expect("the harness compiles");
    design_system::install(&harness);
    harness.show().expect("the harness shows off screen");
    settle(&mut headless);

    // The label above, the box under it, the help under the box.
    assert_eq!(
        harness.get_field_box_y(),
        LABEL + GAP,
        "the box sits under its label"
    );
    assert_eq!(
        harness.get_field_box_height(),
        BOX,
        "the box is `control` tall"
    );
    assert_eq!(harness.get_field_width(), FIELD_W);
    assert_eq!(
        harness.get_field_height(),
        LABEL + GAP + BOX + GAP + MESSAGE,
        "label, box and one line of help"
    );

    let left = harness.get_inset();

    // Without an icon the text starts at the padding (plus its first glyph's side bearing); with one, the icon sits
    // there and the same text moves by the icon and its gap.
    let x0 = left as usize;
    let (plain_start, _) =
        ink(&headless, x0, harness.get_plain_y(), 0..READ_W).expect("the text is drawn");
    assert!(
        (PAD_X..PAD_X + 3).contains(&plain_start),
        "no icon: the text starts at the padding, not at {plain_start}"
    );
    let iconed_top = harness.get_iconed_y();
    let (icon_start, icon_end) =
        ink(&headless, x0, iconed_top, 0..PAD_X + ICON).expect("the icon is drawn");
    assert!(
        icon_start >= PAD_X && icon_end < PAD_X + ICON,
        "the icon in its slot"
    );
    let (text_start, _) =
        ink(&headless, x0, iconed_top, PAD_X + ICON..READ_W).expect("the text is drawn");
    assert_eq!(
        text_start,
        plain_start + ICON + ICON_GAP,
        "the text follows the icon"
    );
    let box_middle = harness.get_field_y() + harness.get_field_box_y() + BOX / 2.0;
    let edge = |headless: &Headless| headless.pixel(left as usize, box_middle as usize);
    let ring = |headless: &Headless, y: f32| headless.pixel((left - RING_OUT) as usize, y as usize);
    assert_eq!(edge(&headless), LINE, "the edge at rest is `line`");

    // A click focuses the text: the edge turns `focus`, with no ring (the pointer brought the focus).
    headless.click(TEXT_X, box_middle);
    settle(&mut headless);
    assert!(
        harness.get_field_focus(),
        "a click puts the focus in the text"
    );
    assert_eq!(edge(&headless), FOCUS, "a focused edge is `focus`");
    assert_eq!(ring(&headless, box_middle), BG, "no ring after a click");

    // Each edit is reported with the text so far; Enter reports the whole text.
    headless.type_text("ab");
    settle(&mut headless);
    assert_eq!(harness.get_field_text(), "ab");
    assert_eq!(harness.get_edits(), 2, "one edit per character");
    assert_eq!(harness.get_last_edit(), "ab");
    headless.press_key(Key::Return);
    assert_eq!(harness.get_accepts(), 1, "Enter accepts");
    assert_eq!(harness.get_last_accept(), "ab");

    // An error turns the edge `danger`, over the focus, and adds its line under the help.
    harness.set_error_text("Too long".into());
    settle(&mut headless);
    assert_eq!(edge(&headless), DANGER, "an error's edge is `danger`");
    assert_eq!(
        harness.get_field_height(),
        LABEL + GAP + BOX + GAP + MESSAGE + GAP + MESSAGE,
        "the error adds a line under the help"
    );
    save_shot(&headless, "error");
    harness.set_error_text("".into());
    settle(&mut headless);
    assert_eq!(edge(&headless), FOCUS, "no error, the focus edge again");

    // Tab out turns the keyboard flag on: the next field shows its ring.
    headless.press_key(Key::Tab);
    settle(&mut headless);
    assert!(harness.get_other_focus(), "Tab moves to the next field");
    assert!(
        harness.global::<InputMode>().get_keyboard(),
        "Tab out of a field is keyboard navigation"
    );
    let other_middle = harness.get_other_y() + BOX / 2.0;
    assert_eq!(
        ring(&headless, other_middle),
        FOCUS,
        "the keyboard ring shows"
    );
    save_shot(&headless, "ring");

    // A click on the label puts the focus back in its text, and the ring goes with the pointer.
    headless.click(TEXT_X, harness.get_field_y() + LABEL / 2.0);
    settle(&mut headless);
    assert!(harness.get_field_focus(), "the label focuses its field");
    assert!(!harness.global::<InputMode>().get_keyboard());
    assert_eq!(
        ring(&headless, box_middle),
        BG,
        "no ring after the label's click"
    );

    // A disabled field takes no typing; a read-only one neither.
    headless.click(TEXT_X, harness.get_off_y() + BOX / 2.0);
    headless.type_text("z");
    settle(&mut headless);
    assert_eq!(harness.get_off_text(), "kept");
    headless.click(TEXT_X, harness.get_fixed_y() + BOX / 2.0);
    headless.type_text("z");
    settle(&mut headless);
    assert_eq!(harness.get_fixed_text(), "fixed");
}
