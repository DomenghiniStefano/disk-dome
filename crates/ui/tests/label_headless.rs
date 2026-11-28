//! The Label off screen (`docs/design-system.md` § 5.1, § 3.3), on its test window `KitLabelHarness`: a text that
//! fits is not cut and one in a narrow box is, on one line; a sentence wraps between words and a path anywhere,
//! neither cut; `max-lines` holds a text to that many lines of the font, cut, with nothing drawn under them. A
//! `SelectableLabel` is selected by a drag and by Ctrl+A, cleared by a click, and a right click on it reaches the
//! context menu's area and keeps the selection for «Copy».

#![cfg(feature = "kit")]

use diskdome_ui::design_system;
use diskdome_ui::generated::KitLabelHarness;
use diskdome_ui::headless::Headless;
use slint::ComponentHandle;
use slint::platform::Key;

const WIDTH: u32 = 960;
const HEIGHT: u32 = 600;
/// `Type.line(body)`: a single-line label is exactly that tall (§ 3.3).
const BODY_LINE: f32 = 18.0;

/// Two turns of the loop: a change handler is seen in the next one.
fn settle(headless: &mut Headless) {
    headless.frame();
    headless.frame();
}

/// One test: the headless platform is installed once per thread.
#[test]
fn label_cuts_wraps_and_holds_to_lines() {
    let mut headless = Headless::install(WIDTH, HEIGHT).expect("headless platform");
    let harness = KitLabelHarness::new().expect("the harness compiles");
    design_system::install(&harness);
    harness.show().expect("the harness shows off screen");
    settle(&mut headless);

    // One line: a short text fits, a long one in the narrow box is cut, and both are one body line tall.
    assert!(!harness.get_short_cut(), "a text that fits is not cut");
    assert!(harness.get_long_cut(), "a long text in a narrow box is cut");
    assert_eq!(harness.get_single_height(), BODY_LINE);

    // Wrapped between words: several lines, nothing cut.
    assert!(!harness.get_sentence_cut(), "a wrapped sentence is not cut");
    assert!(
        harness.get_sentence_height() > 3.0 * BODY_LINE,
        "it takes several lines: {}",
        harness.get_sentence_height()
    );
    // A path has no spaces: it wraps anywhere, nothing cut.
    assert!(
        !harness.get_path_cut(),
        "a path wrapped anywhere is not cut"
    );
    assert!(
        harness.get_path_height() > BODY_LINE,
        "it takes more than one line: {}",
        harness.get_path_height()
    );

    // Held to two and to three lines: each is that many lines of the font (2 : 3), shorter than the whole text,
    // and cut.
    let two = harness.get_two_height();
    let three = harness.get_three_height();
    assert!(
        (two * 3.0 - three * 2.0).abs() < 0.01,
        "two and three lines step by the same line: {two} and {three}"
    );
    assert!(
        three < harness.get_sentence_height(),
        "held shorter than the whole text"
    );
    assert!(harness.get_two_cut(), "a text held to two lines is cut");

    // Nothing is drawn under the second line: the band of one more line under it is the window's ground.
    let x0 = harness.get_two_x() as usize;
    let top = (harness.get_two_y() + two).ceil() as usize + 1;
    let ground = headless.pixel(WIDTH as usize - 4, HEIGHT as usize - 4);
    let drawn = (top..top + (two / 2.0) as usize)
        .flat_map(|y| (x0..x0 + harness.get_narrow() as usize).map(move |x| (x, y)))
        .filter(|&(x, y)| headless.pixel(x, y) != ground)
        .count();
    assert_eq!(drawn, 0, "nothing drawn under the second line");

    // Selectable: nothing is selected at rest; a drag across the text selects it.
    let y = harness.get_pick_y() + BODY_LINE / 2.0;
    let (x, width) = (harness.get_pick_x(), harness.get_pick_width());
    assert!(!harness.get_pick_selected(), "nothing selected at rest");
    headless.press(x + 1.0, y);
    headless.hover(x + width - 1.0, y);
    headless.release(x + width - 1.0, y);
    settle(&mut headless);
    assert!(harness.get_pick_selected(), "a drag selects the text");

    // A right click on it still reaches the area that asks for a context menu, and keeps the selection for «Copy».
    headless.right_click(x + 4.0, y);
    settle(&mut headless);
    assert_eq!(
        harness.get_menu_requests(),
        1,
        "the right click asks for the menu"
    );
    assert!(
        harness.get_pick_selected(),
        "the selection stays for the menu"
    );

    // A click elsewhere in the text clears it; Ctrl+A selects it all again.
    headless.click(x + 4.0, y);
    settle(&mut headless);
    assert!(!harness.get_pick_selected(), "a click clears the selection");
    headless.press_text_with(Key::Control, "a");
    settle(&mut headless);
    assert!(harness.get_pick_selected(), "Ctrl+A selects the whole text");
}
