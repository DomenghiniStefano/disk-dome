//! The SearchField off screen (`docs/design-system.md` § 5.4), on its test window `KitSearchHarness`: the text
//! after the leading icon, or at the padding without it; typing applies
//! after the rest, Enter at once; an error's reason shows under the field while it has the focus; Esc clears, a
//! second Esc leaves; a click on the icon puts the focus back; the regex toggle applies; a disabled field takes no
//! typing.
//!
//! `DISKDOME_KIT_SHOTS=<folder>` also saves the error's tooltip under the field, for a look.

#![cfg(feature = "kit")]

use std::ops::Range;
use std::path::PathBuf;
use std::time::Duration;

use diskdome_ui::design_system;
use diskdome_ui::generated::KitSearchHarness;
use diskdome_ui::headless::Headless;
use slint::ComponentHandle;
use slint::platform::Key;

const WIDTH: u32 = 960;
const HEIGHT: u32 = 600;
/// Past `Motion.search-rest` (150 ms), with a margin for a busy CI machine.
const AFTER_REST: Duration = Duration::from_millis(400);
/// Over the field's text, away from the icon.
const TEXT_X: f32 = 60.0;
/// Over the `search` icon: `field-pad-x` (6) plus half the icon (16).
const ICON_X: f32 = 6.0 + 8.0;
/// Half a control-sm toggle (24), plus the field's right inset (`segment-inset`, 2): the regex toggle's centre from
/// the field's right edge.
const REGEX_FROM_RIGHT: f32 = 2.0 + 12.0;
/// Inside the field, vertically.
const ROW_Y: f32 = 14.0;
/// `field-pad-x`, the leading icon (`Size.icon`, normal density) and the gap after it (`space-4`) (§ 5.4).
const PAD_X: usize = 6;
const ICON: usize = 16;
const ICON_GAP: usize = 4;
/// `control`, the field's height (§ 3.5), and the rows the text is read in, inside it.
const BOX: f32 = 28.0;
const INK_INSET: f32 = 4.0;
/// Inside the box, in its start padding: the field's ground.
const GROUND_X: usize = 3;
/// Where the text is looked for: the field's start, short of its end.
const READ_W: usize = 160;

/// The first and last inked column of the field at `top`, within `columns`.
fn ink(headless: &Headless, top: f32, columns: Range<usize>) -> Option<(usize, usize)> {
    let ground = headless.pixel(GROUND_X, (top + BOX / 2.0) as usize);
    headless.ink_columns(
        columns,
        (top + INK_INSET) as usize..(top + BOX - INK_INSET) as usize,
        ground,
    )
}

fn save_shot(headless: &Headless, name: &str) {
    let Some(folder) = std::env::var_os("DISKDOME_KIT_SHOTS") else {
        return;
    };
    let path = PathBuf::from(folder).join(format!("search-{name}.bmp"));
    std::fs::write(&path, headless.bmp()).expect("the shots folder is writable");
}

/// Two turns of the loop: a timer or a change handler is seen in the next one.
fn settle(headless: &mut Headless) {
    headless.frame();
    headless.frame();
}

/// One test: the headless platform is installed once per thread.
#[test]
fn search_typing_errors_escape_and_toggles() {
    let mut headless = Headless::install(WIDTH, HEIGHT).expect("headless platform");
    let harness = KitSearchHarness::new().expect("the harness compiles");
    design_system::install(&harness);
    harness.show().expect("the harness shows off screen");
    settle(&mut headless);
    let y = harness.get_field_y() + ROW_Y;

    // Without an icon the text starts at the padding (plus its first glyph's side bearing); with the default
    // `search` icon, the icon sits there and the same text moves by the icon and its gap.
    let (bare_start, _) =
        ink(&headless, harness.get_bare_y(), 0..READ_W).expect("the text is drawn");
    assert!(
        (PAD_X..PAD_X + 3).contains(&bare_start),
        "no icon: the text starts at the padding, not at {bare_start}"
    );
    let iconed_top = harness.get_iconed_y();
    let (icon_start, icon_end) =
        ink(&headless, iconed_top, 0..PAD_X + ICON).expect("the icon is drawn");
    assert!(
        icon_start >= PAD_X && icon_end < PAD_X + ICON,
        "the icon in its slot"
    );
    let (text_start, _) =
        ink(&headless, iconed_top, PAD_X + ICON..READ_W).expect("the text is drawn");
    assert_eq!(
        text_start,
        bare_start + ICON + ICON_GAP,
        "the text follows the icon"
    );

    // Typing applies once it rests.
    headless.click(TEXT_X, y);
    assert!(
        harness.get_field_focus(),
        "a click puts the focus in the text"
    );
    headless.type_text("ab");
    settle(&mut headless);
    assert_eq!(harness.get_requests(), 0, "nothing applies while typing");
    std::thread::sleep(AFTER_REST);
    settle(&mut headless);
    assert_eq!(harness.get_requests(), 1, "the rest applies the search");
    assert_eq!(harness.get_last_request(), "ab");

    // Enter applies at once, and the rest it cut short adds nothing.
    headless.type_text("c");
    headless.press_key(Key::Return);
    assert_eq!(harness.get_requests(), 2, "Enter applies at once");
    assert_eq!(harness.get_last_request(), "abc");
    std::thread::sleep(AFTER_REST);
    settle(&mut headless);
    assert_eq!(harness.get_requests(), 2, "no second search after Enter");

    // An error's reason shows at once while the field has the focus.
    harness.set_error_text("Not a valid regular expression".into());
    settle(&mut headless);
    assert!(
        harness.get_error_shown(),
        "the reason shows under the field"
    );
    headless.request_redraw();
    headless.frame();
    save_shot(&headless, "error");
    harness.set_error_text("".into());
    settle(&mut headless);
    assert!(!harness.get_error_shown(), "no error, no tooltip");

    // Esc clears and applies; a second Esc leaves the field.
    headless.press_key(Key::Escape);
    assert_eq!(harness.get_field_text(), "");
    assert_eq!(harness.get_requests(), 3);
    assert_eq!(harness.get_last_request(), "");
    assert!(harness.get_field_focus(), "the first Esc keeps the focus");
    headless.press_key(Key::Escape);
    assert!(
        !harness.get_field_focus(),
        "the second Esc leaves the field"
    );

    // A click on the icon puts the focus back in the text.
    headless.click(ICON_X, y);
    assert!(harness.get_field_focus(), "the icon focuses the text");

    // The regex toggle flips and applies.
    headless.click(harness.get_field_width() - REGEX_FROM_RIGHT, y);
    assert!(harness.get_field_regex());
    assert_eq!(harness.get_requests(), 4, "a toggle applies the search");

    // A disabled field takes no typing.
    headless.click(TEXT_X, harness.get_off_y() + ROW_Y);
    headless.type_text("z");
    settle(&mut headless);
    assert_eq!(harness.get_off_text(), "kept");
}
