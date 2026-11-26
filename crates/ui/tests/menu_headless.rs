//! The Menu off screen (`docs/design-system.md` § 5.16, § 3.11), on its test window `KitMenuHarness`: opening and
//! choosing with the pointer and the keyboard, the type-ahead, submenus, disabled items, the focus coming back to
//! the button, the context menu, and where popups go near the window's edges (`Placement`).
//!
//! `DISKDOME_KIT_SHOTS=<folder>` also saves the menus open near both corners, for a look.

#![cfg(feature = "kit")]

use std::path::PathBuf;

use diskdome_ui::design_system;
use diskdome_ui::generated::{
    KitMenuHarness, KitMenuProbe, Placement, PopupAlign, PopupAnchor, PopupSide, Viewport,
};
use diskdome_ui::headless::Headless;
use slint::ComponentHandle;
use slint::platform::Key;

const WIDTH: u32 = 960;
const HEIGHT: u32 = 600;
/// `Size.menu-item` at normal density.
const ROW: f32 = 24.0;
/// From the button's bottom edge to the first row's top: `popup-gap` 4, the frame's 1 px edge, `menu-pad` 4.
const FIRST_ROW: f32 = 9.0;
/// A point inside the menu's rows, away from the frame's edge.
const ROW_X: f32 = 40.0;
/// The same window size `DomeWindow` tells the popups.
const MARGIN: f32 = 8.0;
const GAP: f32 = 4.0;
/// Longer than `Motion.base`, 140 ms, with a margin.
const FADE: std::time::Duration = std::time::Duration::from_millis(250);

fn save_shot(headless: &Headless, name: &str) {
    let Some(folder) = std::env::var_os("DISKDOME_KIT_SHOTS") else {
        return;
    };
    let path = PathBuf::from(folder).join(format!("menu-{name}.bmp"));
    std::fs::write(&path, headless.bmp()).expect("the shots folder is writable");
}

fn chosen(harness: &KitMenuHarness) -> String {
    harness.global::<KitMenuProbe>().get_chosen().to_string()
}

fn anchor(x: f32, y: f32, width: f32, height: f32) -> PopupAnchor {
    PopupAnchor {
        x,
        y,
        width,
        height,
    }
}

/// A pixel as red, green, blue, without the alpha.
fn rgb(headless: &Headless, x: f32, y: f32) -> [u8; 3] {
    let [r, g, b, _] = headless.pixel(x as usize, y as usize);
    [r, g, b]
}

/// One test: the headless platform is installed once per thread.
#[test]
fn menu_works_by_pointer_and_keyboard() {
    let mut headless = Headless::install(WIDTH, HEIGHT).expect("headless platform");
    let harness = KitMenuHarness::new().expect("the harness compiles");
    design_system::install(&harness);
    harness.show().expect("the harness shows off screen");
    headless.frame();
    let button = harness.get_button_height();
    let row = |index: f32| button + FIRST_ROW + ROW * index + ROW / 2.0;

    // An empty menu keeps one row for its text: a row, the frame's 1 px edges and its 4 px padding.
    assert_eq!(harness.get_empty_height(), ROW + 2.0 * (1.0 + 4.0));

    // Pointer: the button opens the menu, a click on an item chooses it and closes the menu. It fades in (§ 3.10):
    // the first frame is not yet the menu's final colour, the frame after the fade is.
    let inside = (ROW_X, row(2.0));
    let before = rgb(&headless, inside.0, inside.1);
    headless.click(ROW_X, button / 2.0);
    headless.frame();
    let first = rgb(&headless, inside.0, inside.1);
    std::thread::sleep(FADE);
    headless.frame();
    let last = rgb(&headless, inside.0, inside.1);
    assert!(
        first != last && first != before,
        "the menu fades in: {before:?} → {first:?} → {last:?}"
    );
    assert!(harness.get_corner_open(), "a click opens the menu");
    save_shot(&headless, "corner");
    headless.click(ROW_X, row(1.0));
    assert_eq!(chosen(&harness), "cut");
    assert!(!harness.get_corner_open(), "choosing closes the menu");
    assert_eq!(harness.global::<KitMenuProbe>().get_closed(), 1);

    // A disabled item does nothing and the menu stays open; Esc closes it.
    headless.click(ROW_X, button / 2.0);
    headless.frame();
    headless.click(ROW_X, row(4.0) + FIRST_ROW);
    assert!(
        harness.get_corner_open(),
        "a disabled item keeps the menu open"
    );
    headless.press_key(Key::Escape);
    assert!(!harness.get_corner_open(), "Esc closes the menu");

    // Keyboard: the focus came back to the button, so ↓ opens on the first item and Enter chooses.
    headless.press_key(Key::DownArrow);
    assert!(harness.get_corner_open(), "↓ on the button opens the menu");
    headless.press_key(Key::DownArrow);
    headless.press_key(Key::DownArrow);
    headless.press_key(Key::Return);
    assert_eq!(chosen(&harness), "paste", "↓ twice from Copy reaches Paste");

    // ↑ opens on the last item, Delete; it is disabled, so Enter does nothing. ↑ goes to Sort, ↑ again skips the
    // separator to Paste.
    headless.press_key(Key::UpArrow);
    assert!(harness.get_corner_open());
    headless.press_key(Key::Return);
    assert!(
        harness.get_corner_open(),
        "Enter on a disabled item does nothing"
    );
    headless.press_key(Key::UpArrow);
    headless.press_key(Key::UpArrow);
    headless.press_key(Key::Return);
    assert_eq!(chosen(&harness), "paste", "↑ skips the separator");

    // Type-ahead. ↓ opens on Copy; «c» moves to the next item starting with c, Cut.
    headless.press_key(Key::DownArrow);
    headless.type_text("c");
    headless.press_key(Key::Return);
    assert_eq!(chosen(&harness), "cut");
    // «c» again moves on once more and goes round: Cut, then Copy.
    headless.press_key(Key::DownArrow);
    headless.type_text("cc");
    headless.press_key(Key::Return);
    assert_eq!(chosen(&harness), "copy");

    // Submenu: → opens it on its first item, ← closes only it, Enter in it chooses and closes everything.
    headless.press_key(Key::DownArrow);
    headless.type_text("s");
    headless.press_key(Key::RightArrow);
    headless.frame();
    save_shot(&headless, "submenu");
    headless.press_key(Key::LeftArrow);
    assert!(harness.get_corner_open(), "← closes the submenu only");
    headless.press_key(Key::RightArrow);
    headless.press_key(Key::DownArrow);
    headless.press_key(Key::Return);
    assert_eq!(chosen(&harness), "by-name");
    assert!(
        !harness.get_corner_open(),
        "choosing in a submenu closes the menu"
    );

    // Tab closes the menu too.
    headless.press_key(Key::DownArrow);
    headless.press_key(Key::Tab);
    assert!(!harness.get_corner_open(), "Tab closes the menu");

    // The far corner: the menu opens above the button and inside the window.
    let far = (
        harness.get_far_x() + 4.0,
        harness.get_far_y() + button / 2.0,
    );
    headless.click(far.0, far.1);
    headless.frame();
    assert!(harness.get_far_open());
    save_shot(&headless, "far-corner");
    headless.press_key(Key::Escape);

    // The context menu: a right click opens it at the pointer.
    let area_y = harness.get_area_y() + ROW;
    headless.right_click(ROW_X, area_y);
    headless.frame();
    assert!(
        harness.get_area_open(),
        "a right click opens the context menu"
    );
    headless.press_key(Key::DownArrow);
    headless.press_key(Key::Return);
    assert_eq!(chosen(&harness), "copy");

    // With the focus inside the area, the Menu key and Shift+F10 open it too.
    harness.invoke_focus_area();
    headless.press_key(Key::Menu);
    assert!(
        harness.get_area_open(),
        "the Menu key opens the context menu"
    );
    headless.press_key(Key::Escape);
    assert!(!harness.get_area_open());
    harness.invoke_focus_area();
    headless.press_key_with(Key::Shift, Key::F10);
    assert!(harness.get_area_open(), "Shift+F10 opens the context menu");
    headless.press_key(Key::Escape);
}

#[test]
fn popups_flip_and_stay_inside_the_window() {
    let _headless = Headless::install(WIDTH, HEIGHT).expect("headless platform");
    let harness = KitMenuHarness::new().expect("the harness compiles");
    let viewport = harness.global::<Viewport>();
    viewport.set_width(800.0);
    viewport.set_height(600.0);
    let place = harness.global::<Placement>();

    // Room below: the preferred side.
    let top_left = anchor(10.0, 10.0, 80.0, 28.0);
    let side = place.invoke_side(
        top_left.clone(),
        200.0,
        300.0,
        PopupSide::Bottom,
        GAP,
        MARGIN,
    );
    assert_eq!(side, PopupSide::Bottom);
    assert_eq!(
        place.invoke_y(
            top_left.clone(),
            300.0,
            side,
            PopupAlign::Start,
            GAP,
            MARGIN
        ),
        42.0
    );

    // No room below a button at the bottom: it flips up.
    let bottom_right = anchor(720.0, 572.0, 80.0, 28.0);
    let side = place.invoke_side(
        bottom_right.clone(),
        200.0,
        300.0,
        PopupSide::Bottom,
        GAP,
        MARGIN,
    );
    assert_eq!(side, PopupSide::Top);
    assert_eq!(
        place.invoke_y(
            bottom_right.clone(),
            300.0,
            side,
            PopupAlign::Start,
            GAP,
            MARGIN
        ),
        268.0
    );
    // Lined up on the button's start edge it would leave the window: moved back to the margin.
    assert_eq!(
        place.invoke_x(
            bottom_right.clone(),
            200.0,
            side,
            PopupAlign::Start,
            GAP,
            MARGIN
        ),
        592.0
    );
    // Lined up on its end edge (600) it would touch the window's edge: also kept 8 px inside.
    assert_eq!(
        place.invoke_x(
            bottom_right.clone(),
            200.0,
            side,
            PopupAlign::End,
            GAP,
            MARGIN
        ),
        592.0
    );
    assert_eq!(
        place.invoke_x(bottom_right, 60.0, side, PopupAlign::Center, GAP, MARGIN),
        730.0
    );

    // Taller than the room on either side (310 px below, 238 above): the side with more room, cut to it, so the
    // list scrolls.
    let middle = anchor(100.0, 250.0, 80.0, 28.0);
    let side = place.invoke_side(middle.clone(), 200.0, 900.0, PopupSide::Bottom, GAP, MARGIN);
    assert_eq!(side, PopupSide::Bottom);
    assert_eq!(
        place.invoke_fitted_height(middle, 900.0, side, GAP, MARGIN),
        310.0
    );

    // A submenu at the right edge opens to the left.
    let item = anchor(600.0, 100.0, 200.0, 24.0);
    let side = place.invoke_side(item.clone(), 220.0, 100.0, PopupSide::Right, -4.0, MARGIN);
    assert_eq!(side, PopupSide::Left);
    assert_eq!(
        place.invoke_x(item, 220.0, side, PopupAlign::Start, -4.0, MARGIN),
        384.0
    );
}
