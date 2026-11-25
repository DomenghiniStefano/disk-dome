//! Drives the Kit window off screen (Slint's software renderer, no window opened): the components of card 4.2
//! with the keyboard and the pointer, through the Kit's own theme and density switches (two SegmentedControls).
//!
//! `DISKDOME_KIT_SHOTS=<folder>` also saves the Kit as BMP in dark / normal and light / compact, for a look.

#![cfg(feature = "kit")]

use std::path::PathBuf;

use diskdome_ui::generated::{KitWindow, Nav, Size, Theme, ThemeMode};
use diskdome_ui::proof::headless::Headless;
use slint::ComponentHandle;
use slint::platform::Key;

const WIDTH: u32 = 1280;
// Tall enough to show every section of the Kit in one shot.
const HEIGHT: u32 = 1400;

/// The theme switch's «Dark» segment, in the Kit's top bar (logical pixels).
const DARK_SEGMENT: (f32, f32) = (125.0, 18.0);

fn save_shot(headless: &Headless, name: &str) {
    let Some(folder) = std::env::var_os("DISKDOME_KIT_SHOTS") else {
        return;
    };
    let path = PathBuf::from(folder).join(format!("kit-{name}.bmp"));
    std::fs::write(&path, headless.bmp()).expect("the shots folder is writable");
}

/// One test: the headless platform is installed once per thread.
#[test]
fn kit_switches_work_by_keyboard_and_pointer() {
    let mut headless = Headless::install(WIDTH, HEIGHT).expect("headless platform");
    let kit = KitWindow::new().expect("the Kit compiles");
    kit.show().expect("the Kit shows off screen");
    headless.frame();
    save_shot(&headless, "dark-normal");

    // Tab reaches the theme switch and shows the keyboard focus; the arrow moves the choice.
    headless.press_key(Key::Tab);
    assert!(
        kit.global::<Nav>().get_keyboard(),
        "Tab turns the focus ring on"
    );
    headless.press_key(Key::RightArrow);
    assert_eq!(kit.global::<Theme>().get_mode(), ThemeMode::Light);
    // The arrow stops at the last option.
    headless.press_key(Key::RightArrow);
    assert_eq!(kit.global::<Theme>().get_mode(), ThemeMode::Light);

    // The next stop is the density switch; End chooses its last option.
    headless.press_key(Key::Tab);
    headless.press_key(Key::End);
    assert!(kit.global::<Size>().get_compact());
    headless.frame();
    save_shot(&headless, "light-compact");

    // A click chooses a segment and turns the focus ring off.
    headless.click(DARK_SEGMENT.0, DARK_SEGMENT.1);
    assert_eq!(kit.global::<Theme>().get_mode(), ThemeMode::Dark);
    assert!(
        !kit.global::<Nav>().get_keyboard(),
        "a pointer press hides the focus ring"
    );
}
