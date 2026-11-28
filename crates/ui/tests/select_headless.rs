//! The Select off screen (`docs/design-system.md` § 5.5), on its test window `KitSelectHarness`: its height and
//! widths, opening by pointer and keyboard on the chosen item, choosing, disabled items, the arrows stopping at the
//! ends, Esc giving the focus back, the type-ahead on the closed select, and a disabled select.
//!
//! `DISKDOME_KIT_SHOTS=<folder>` also saves the select open and the long list open on its chosen item, for a look.

#![cfg(feature = "kit")]

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
