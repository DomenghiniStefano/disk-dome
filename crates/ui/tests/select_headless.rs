//! The Select off screen (`docs/design-system.md` § 5.5), on its test window `KitSelectHarness`: its heights and
//! widths, the text in the centre, the leading icon, opening by pointer and keyboard on the chosen item, choosing, disabled items, the arrows stopping at the
//! ends, Esc giving the focus back, the type-ahead on the closed select, and a disabled select.
//!
//! `DISKDOME_KIT_SHOTS=<folder>` also saves the select open and the long list open on its chosen item, for a look.

#![cfg(feature = "kit")]

use std::ops::Range;
use std::path::PathBuf;
use std::time::Duration;

use diskdome_ui::design_system;
use diskdome_ui::generated::{KitSelectHarness, KitSelectProbe};
use diskdome_ui::headless::Headless;
use slint::ComponentHandle;
use slint::platform::Key;

const WIDTH: u32 = 960;
const HEIGHT: u32 = 600;
/// `Size.control` at normal density: the select is a normal `secondary` button (§ 5.2, § 3.5).
const CONTROL: f32 = 28.0;
/// `Size.control-sm` and `Size.control-lg` at normal density: the small and large buttons' heights (§ 5.2, § 3.5).
const CONTROL_SM: f32 = 24.0;
const CONTROL_LG: f32 = 32.0;
/// `button-pad-x`, on a side with text only, and `button-pad-x-icon`, on a side with an icon (§ 3.4, § 5.2).
const PAD_X: usize = 10;
const PAD_X_ICON: usize = 8;
/// `Size.icon` at normal density, and `label-gap` between an icon and the text (§ 5.2).
const ICON: usize = 16;
const GAP: usize = 6;
/// Inside the button, in its start padding: its fill, away from any ink.
const GROUND_X: usize = 3;
/// The rows the text and the icons are read in: the button's height less its top and bottom few pixels.
const INK_INSET: f32 = 4.0;
/// How far the centre of the text's ink may sit from the centre of its room: glyphs' side bearings differ by a
/// pixel or so.
const CENTRE_TOLERANCE: f32 = 1.5;
/// `Size.menu-item` at normal density.
const ROW: f32 = 24.0;
/// From the select's bottom edge to the first row's top: `popup-gap` 4, the frame's 1 px edge, `menu-pad` 4.
const FIRST_ROW: f32 = 9.0;
/// A point inside the list's rows, away from the frame's edge.
const ROW_X: f32 = 40.0;
/// Longer than `Motion.typeahead-reset`, 1 s, so the next letter starts a new search.
const TYPEAHEAD_RESET: Duration = Duration::from_millis(1300);

// The harness's entries: 0 Size, 1 Name, 2 Last modified, 3 —, 4 Allocated (disabled), 5 Type.
const NAME: i32 = 1;
const LAST_MODIFIED: i32 = 2;
const TYPE: i32 = 5;

fn save_shot(headless: &Headless, name: &str) {
    let Some(folder) = std::env::var_os("DISKDOME_KIT_SHOTS") else {
        return;
    };
    let path = PathBuf::from(folder).join(format!("select-{name}.bmp"));
    std::fs::write(&path, headless.bmp()).expect("the shots folder is writable");
}

/// The first and last inked column of the select at `top`, `height` tall, within `columns` of it.
fn ink(
    headless: &Headless,
    top: f32,
    height: f32,
    columns: Range<usize>,
) -> Option<(usize, usize)> {
    let middle = (top + height / 2.0) as usize;
    let ground = headless.pixel(GROUND_X, middle);
    headless.ink_columns(
        columns,
        (top + INK_INSET) as usize..(top + height - INK_INSET) as usize,
        ground,
    )
}

fn selections(harness: &KitSelectHarness) -> i32 {
    harness.global::<KitSelectProbe>().get_selections()
}

/// One test: the headless platform is installed once per thread.
#[test]
fn select_works_by_pointer_and_keyboard() {
    let mut headless = Headless::install(WIDTH, HEIGHT).expect("headless platform");
    let harness = KitSelectHarness::new().expect("the harness compiles");
    design_system::install(&harness);
    harness.show().expect("the harness shows off screen");
    headless.frame();
    let row = |index: f32| CONTROL + FIRST_ROW + ROW * index + ROW / 2.0;

    // Size: a normal button's height. Width `longest`: as wide as a select that hugs its longest item, wider than
    // one that hugs a short item, and it does not move when the choice changes.
    assert_eq!(harness.get_sort_height(), CONTROL);
    let longest = harness.get_sort_width();
    assert_eq!(longest, harness.get_hug_longest_width());
    assert!(harness.get_hug_short_width() < longest);
    assert_eq!(harness.get_current(), -1, "nothing chosen: the placeholder");

    // Sizes: the button's heights of the same name.
    assert_eq!(harness.get_small_height(), CONTROL_SM);
    assert_eq!(harness.get_large_height(), CONTROL_LG);

    // The text at the start: it begins at the padding (plus its first glyph's side bearing).
    let plain_top = harness.get_hug_short_y();
    let (plain_start, _) =
        ink(&headless, plain_top, CONTROL, 0..WIDTH as usize / 2).expect("the text is drawn");
    assert!(
        (PAD_X..PAD_X + 3).contains(&plain_start),
        "the text starts at the padding, not at {plain_start}"
    );

    // The leading icon: its ink sits in its slot after the tighter padding, and the same text moves by the icon and
    // the gap, less the padding the icon's side loses.
    let icon_top = harness.get_iconic_y();
    let (icon_start, icon_end) =
        ink(&headless, icon_top, CONTROL, 0..PAD_X_ICON + ICON).expect("the icon is drawn");
    assert!(
        icon_start >= PAD_X_ICON && icon_end < PAD_X_ICON + ICON,
        "the icon in its slot"
    );
    let text_after = PAD_X_ICON + ICON;
    let (icon_text_start, _) = ink(&headless, icon_top, CONTROL, text_after..WIDTH as usize / 2)
        .expect("the text is drawn after the icon");
    assert_eq!(
        icon_text_start,
        plain_start + ICON + GAP - (PAD_X - PAD_X_ICON),
        "the text follows the icon"
    );

    // The text in the centre of its room, between the padding and the chevron with its gap.
    let centred_top = harness.get_centred_y();
    let room_end = harness.get_centred_width() as usize - PAD_X_ICON - ICON - GAP;
    let (first, last) =
        ink(&headless, centred_top, CONTROL, PAD_X..room_end).expect("the centred text is drawn");
    let ink_centre = (first + last) as f32 / 2.0;
    let room_centre = (PAD_X + room_end) as f32 / 2.0;
    assert!(
        (ink_centre - room_centre).abs() <= CENTRE_TOLERANCE,
        "the text's centre {ink_centre} is the room's {room_centre}"
    );
    assert!(first > plain_start, "centred, not at the start");

    // Pointer: a click opens the list, a click on an item chooses it and closes the list.
    headless.click(ROW_X, CONTROL / 2.0);
    headless.frame();
    assert!(harness.get_sort_open(), "a click opens the list");
    save_shot(&headless, "open");
    headless.click(ROW_X, row(1.0));
    assert_eq!(harness.get_current(), NAME);
    assert_eq!(selections(&harness), 1);
    assert_eq!(harness.global::<KitSelectProbe>().get_last_selected(), NAME);
    assert!(!harness.get_sort_open(), "choosing closes the list");
    headless.frame();
    assert_eq!(harness.get_sort_width(), longest, "the width never jumps");

    // Keyboard: the focus came back to the select. ↓ opens on the chosen item, Name, so ↓ then Enter is the next one.
    headless.press_key(Key::DownArrow);
    assert!(harness.get_sort_open(), "↓ opens the list");
    headless.press_key(Key::DownArrow);
    headless.press_key(Key::Return);
    assert_eq!(
        harness.get_current(),
        LAST_MODIFIED,
        "the list opens on the chosen item"
    );
    assert_eq!(selections(&harness), 2);

    // Space opens too, on Last modified. ↓ skips the separator to Allocated, disabled: Enter does nothing. ↓ reaches
    // Type, the last item, and ↓ again stays there (no wrap, as Radix).
    headless.press_key(Key::Space);
    assert!(harness.get_sort_open(), "Space opens the list");
    headless.press_key(Key::DownArrow);
    headless.press_key(Key::Return);
    assert!(
        harness.get_sort_open(),
        "Enter on a disabled item does nothing"
    );
    headless.press_key(Key::DownArrow);
    headless.press_key(Key::DownArrow);
    headless.press_key(Key::Return);
    assert_eq!(harness.get_current(), TYPE, "the arrows stop at the end");
    assert_eq!(selections(&harness), 3);

    // Enter on the chosen item closes the list without a new `selected`.
    headless.press_key(Key::Return);
    assert!(harness.get_sort_open(), "Enter opens the list");
    headless.press_key(Key::Return);
    assert!(!harness.get_sort_open());
    assert_eq!(selections(&harness), 3, "choosing the same item again");

    // Esc closes the list without choosing and gives the focus back: typing on the closed select chooses.
    headless.press_key(Key::UpArrow);
    assert!(harness.get_sort_open(), "↑ opens the list");
    headless.press_key(Key::Escape);
    assert!(!harness.get_sort_open(), "Esc closes the list");
    assert_eq!(harness.get_current(), TYPE);
    headless.type_text("n");
    assert_eq!(harness.get_current(), NAME, "typing on the closed select");
    assert!(!harness.get_sort_open(), "typing does not open the list");
    assert_eq!(selections(&harness), 4);

    // The closed type-ahead skips a disabled item: «a» finds only Allocated, so nothing changes.
    headless.wait(TYPEAHEAD_RESET);
    headless.type_text("a");
    assert_eq!(
        harness.get_current(),
        NAME,
        "a disabled item is never chosen"
    );
    headless.wait(TYPEAHEAD_RESET);
    headless.type_text("la");
    assert_eq!(
        harness.get_current(),
        LAST_MODIFIED,
        "more letters narrow the search"
    );

    // A disabled select does not open.
    headless.click(ROW_X, harness.get_disabled_y() + CONTROL / 2.0);
    headless.frame();
    assert!(!harness.get_disabled_open(), "a disabled select stays shut");

    // A long list near the bottom opens above, cut to the room, on its chosen item (looked at in the shot).
    headless.click(
        WIDTH as f32 / 2.0 + ROW_X,
        harness.get_long_y() + CONTROL / 2.0,
    );
    headless.wait(Duration::from_millis(300));
    assert!(harness.get_long_open());
    save_shot(&headless, "long");
    headless.press_key(Key::Escape);
}
