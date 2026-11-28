//! Drives the Kit window off screen (Slint's software renderer, no window opened): the components with the
//! keyboard and the pointer, through the Kit's own theme and density switches (two SegmentedControls), and the
//! review board's count.
//!
//! `DISKDOME_KIT_SHOTS=<folder>` also saves the Kit as BMP in dark / normal and light / compact, for a look.

#![cfg(feature = "kit")]

use std::path::PathBuf;

use diskdome_ui::generated::{InputMode, KitReview, KitSearch, Size, Theme, ThemeMode};
use diskdome_ui::headless::Headless;
use diskdome_ui::kit;
use slint::ComponentHandle;
use slint::platform::Key;

const WIDTH: u32 = 1280;
// Tall enough to show every section of the Kit, tokens included, in one shot.
const HEIGHT: u32 = 12000;

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
    let kit = kit::open().expect("the Kit compiles");
    kit.show().expect("the Kit shows off screen");
    // The review board's count in the top bar comes from Rust.
    let review = kit.global::<KitReview>();
    assert_eq!(review.get_open_count(), 0);
    assert_eq!(review.get_proposed_count(), 3);
    headless.frame();
    save_shot(&headless, "dark-normal");

    // Tab reaches the theme switch and shows the keyboard focus; the arrow moves the choice.
    headless.press_key(Key::Tab);
    assert!(
        kit.global::<InputMode>().get_keyboard(),
        "Tab turns the focus ring on"
    );
    headless.press_key(Key::RightArrow);
    assert_eq!(kit.global::<Theme>().get_mode(), ThemeMode::Light);
    // The arrow goes round the ends (`wrap`, on by default as in Radix, § 5.6), and back.
    headless.press_key(Key::RightArrow);
    assert_eq!(kit.global::<Theme>().get_mode(), ThemeMode::Dark);
    headless.press_key(Key::LeftArrow);
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
        !kit.global::<InputMode>().get_keyboard(),
        "a pointer press hides the focus ring"
    );

    // Test 7, decision 8: Segoe UI 400 and 700 have tabular digits, so tree columns and their bold totals line
    // up. Windows only: macOS has no Segoe UI.
    #[cfg(windows)]
    {
        assert!(kit.get_regular_tabular(), "Segoe UI 400 has tabular digits");
        assert!(kit.get_strong_tabular(), "Segoe UI 700 has tabular digits");
    }

    // The search keeps only the sections whose title matches; Rust answers the match.
    let search = kit.global::<KitSearch>();
    search.set_query("scroll|toggle".into());
    search.set_regex(true);
    headless.frame();
    save_shot(&headless, "search");
    assert!(search.invoke_matches(
        "ScrollBar · states".into(),
        "scroll|toggle".into(),
        false,
        true
    ));
    assert!(!search.invoke_valid("([a-z".into(), true));
}
