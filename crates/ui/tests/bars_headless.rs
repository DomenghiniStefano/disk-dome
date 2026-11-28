//! The Splitter, ToolBar, PanelHeader and StatusBar off screen (`docs/design-system.md` § 5.24, § 5.25), on their test
//! window `KitBarsHarness`: a splitter dragged past both panes' limits and back, moved by the keyboard, restored by
//! Enter and a double click, and kept inside a shrinking total; a toolbar whose last items fold into «⋯» as it
//! narrows, a separator never ending the row, a folded toggle flipped and a folded button chosen from the menu by
//! keyboard; a panel header whose options menu always shows; and a status bar whose `first` texts shorten before the
//! `later` one, the rest whole.

#![cfg(feature = "kit")]

use std::time::Duration;

use diskdome_ui::design_system;
use diskdome_ui::generated::KitBarsHarness;
use diskdome_ui::headless::Headless;
use slint::ComponentHandle;
use slint::platform::Key;

const WIDTH: u32 = 960;
const HEIGHT: u32 = 600;

/// The splitter's box: `dialog-lg` wide, the side panel `side-panel` (300) wide at first.
const TOTAL: f32 = 680.0;
const START: f32 = 300.0;
/// The line (`border-hairline`).
const LINE: f32 = 1.0;
/// The first pane's limits: at least `table-min` (120), and at least the total less the line and the side panel's
/// `side-panel-max` (480): 199; at most the total less the line and `side-panel-min` (220): 459.
const LOW: f32 = TOTAL - LINE - 480.0;
const HIGH: f32 = TOTAL - LINE - 220.0;
/// Double click and Enter: the side panel back to `side-panel` (300).
const DEFAULT: f32 = TOTAL - 300.0 - LINE;
/// `splitter-step`.
const STEP: f32 = 16.0;
/// Half of the splitter's height, `treemap-min` (120).
const SPLIT_MID: f32 = 60.0;

/// `toolbar`, `tabbar` and `statusbar` at normal density (§ 3.5).
const TOOLBAR: f32 = 36.0;
const TABBAR: f32 = 28.0;
const STATUSBAR: f32 = 22.0;
/// `panel-pad`, `tool-gap`, `group-gap` (§ 3.4, § 3.5).
const PAD: f32 = 8.0;
const GAP: f32 = 4.0;
const GROUP: f32 = 8.0;
/// A short text button is `button-min-w` wide (§ 5.2); «⋯» is a square `control` (28), `control-sm` (24) in a
/// panel header.
const BUTTON: f32 = 64.0;
const MORE: f32 = 28.0;
const MORE_SM: f32 = 24.0;
/// A separator: `group-gap` and the line, with no `tool-gap` around it.
const SEPARATOR: f32 = GROUP + LINE;
/// Alpha and Beta with the «⋯» after them: 8 + 64 + (4 + 64) + 4 + 28 + 8 (the gap to the children) = 188; with the
/// separator (9) too it would be 197. So 190 shows two items.
const BAR_TWO: f32 = 2.0 * PAD + BUTTON + GAP + BUTTON + GAP + MORE + GROUP + 2.0;
/// Narrower than the first item with the «⋯»: nothing shows but the «⋯».
const BAR_NONE: f32 = 60.0;
/// A status bar separator: `statusbar-gap` (4) on each side of the line.
const SEPARATOR_STATUS: f32 = 2.0 * 4.0 + LINE;
/// The panel header is `side-panel` (300) wide; its «⋯» sits at its end.
const HEADER: f32 = 300.0;
/// `statusbar-item-min-w`: how short a status text gets.
const ITEM_MIN: f32 = 48.0;
/// Layout lengths are floats.
const EPSILON: f32 = 0.5;
/// Past `Motion.tooltip-delay` (500 ms), with a margin for a busy CI machine.
const AFTER_DELAY: Duration = Duration::from_millis(700);
/// Longer than `Motion.base` (140 ms), the menu's fade.
const FADE: Duration = Duration::from_millis(250);

/// Two turns of the loop: a timer or a change handler is seen in the next one.
fn settle(headless: &mut Headless) {
    headless.frame();
    headless.frame();
}

fn near(actual: f32, expected: f32) -> bool {
    (actual - expected).abs() <= EPSILON
}

/// Drags the splitter's line from `from` to each of `to` in turn, then lets go at the last one.
fn drag(headless: &mut Headless, from: f32, to: &[f32]) {
    headless.press(from + LINE / 2.0, SPLIT_MID);
    settle(headless);
    for &x in to {
        headless.hover(x + LINE / 2.0, SPLIT_MID);
        settle(headless);
    }
    let last = to.last().copied().unwrap_or(from);
    headless.release(last + LINE / 2.0, SPLIT_MID);
    settle(headless);
}

/// One test: the headless platform is installed once per thread.
#[test]
fn splitter_and_bars() {
    let mut headless = Headless::install(WIDTH, HEIGHT).expect("headless platform");
    let harness = KitBarsHarness::new().expect("the harness compiles");
    design_system::install(&harness);
    harness.show().expect("the harness shows off screen");
    settle(&mut headless);

    // Splitter: the harness's constants are the spec's.
    assert_eq!(harness.get_split_total(), TOTAL, "the splitter's box");
    assert_eq!(
        harness.get_split_actual(),
        START,
        "the side panel at `side-panel`"
    );

    // A drag moves it with the pointer.
    headless.press(START + LINE / 2.0, SPLIT_MID);
    settle(&mut headless);
    headless.hover(START + 50.0 + LINE / 2.0, SPLIT_MID);
    settle(&mut headless);
    assert!(
        harness.get_split_dragging(),
        "the pointer holds the splitter"
    );
    assert_eq!(
        harness.get_split_actual(),
        START + 50.0,
        "it follows the pointer"
    );
    headless.release(START + 50.0 + LINE / 2.0, SPLIT_MID);
    settle(&mut headless);
    assert!(!harness.get_split_dragging(), "let go");
    assert_eq!(
        harness.get_last_resize(),
        START + 50.0,
        "`resized` says where"
    );

    // Past the side panel's minimum it stops, and coming back it follows the pointer again from where it was held.
    drag(&mut headless, START + 50.0, &[TOTAL - 20.0, START + 100.0]);
    assert_eq!(
        harness.get_split_actual(),
        START + 100.0,
        "back under the pointer after the limit"
    );
    drag(&mut headless, START + 100.0, &[TOTAL - 20.0]);
    assert_eq!(
        harness.get_split_actual(),
        HIGH,
        "the side panel keeps `side-panel-min`"
    );
    drag(&mut headless, HIGH, &[20.0]);
    assert_eq!(
        harness.get_split_actual(),
        LOW,
        "the side panel stops at `side-panel-max`"
    );

    // The keyboard: the splitter is the first Tab stop.
    let before = harness.get_resizes();
    headless.press_key(Key::Tab);
    settle(&mut headless);
    headless.press_key(Key::RightArrow);
    settle(&mut headless);
    assert_eq!(
        harness.get_split_actual(),
        LOW + STEP,
        "→ moves it by `splitter-step`"
    );
    headless.press_key(Key::End);
    settle(&mut headless);
    assert_eq!(harness.get_split_actual(), HIGH, "End: the far limit");
    headless.press_key(Key::Home);
    settle(&mut headless);
    assert_eq!(harness.get_split_actual(), LOW, "Home: the near limit");
    headless.press_key(Key::Return);
    settle(&mut headless);
    assert_eq!(
        harness.get_split_actual(),
        DEFAULT,
        "Enter restores the default"
    );
    assert_eq!(
        harness.get_resizes(),
        before + 4,
        "every move says `resized` once"
    );

    // A double click restores the default too.
    headless.press_key(Key::Home);
    settle(&mut headless);
    headless.click(LOW + LINE / 2.0, SPLIT_MID);
    headless.click(LOW + LINE / 2.0, SPLIT_MID);
    settle(&mut headless);
    assert_eq!(
        harness.get_split_actual(),
        DEFAULT,
        "a double click restores the default"
    );

    // The screen sets it without `resized`; a smaller total keeps the side panel's minimum, and the position comes
    // back when the room does.
    let before = harness.get_resizes();
    harness.set_split_position(START);
    settle(&mut headless);
    assert_eq!(
        harness.get_split_actual(),
        START,
        "the screen sets the position"
    );
    harness.set_split_total(500.0);
    settle(&mut headless);
    assert_eq!(
        harness.get_split_actual(),
        500.0 - LINE - 220.0,
        "a smaller total keeps `side-panel-min`"
    );
    assert_eq!(
        harness.get_split_position(),
        START,
        "the position itself is kept"
    );
    harness.set_split_total(TOTAL);
    settle(&mut headless);
    assert_eq!(
        harness.get_split_actual(),
        START,
        "and comes back with the room"
    );
    assert_eq!(
        harness.get_resizes(),
        before,
        "no `resized` for the screen's changes"
    );

    // ToolBar: `toolbar` tall; at the window's width every item shows.
    let bar_mid = harness.get_bar_y() + TOOLBAR / 2.0;
    assert_eq!(
        harness.get_bar_height(),
        TOOLBAR,
        "a toolbar is `toolbar` tall"
    );
    assert_eq!(harness.get_bar_shown(), 5, "every item shows");
    assert!(!harness.get_bar_overflowing(), "no «⋯»");

    // Narrower, the last ones fold; the separator does not end the row.
    harness.set_bar_width(BAR_TWO);
    settle(&mut headless);
    assert_eq!(
        harness.get_bar_shown(),
        2,
        "Alpha and Beta show, the rest fold"
    );
    assert!(harness.get_bar_overflowing(), "the «⋯» shows");

    // A click on Beta; then Tab reaches the «⋯» (the folded items are no Tab stops), ↓ opens it on Gamma, the first
    // folded item (the separator is skipped), and Enter flips the toggle in its row of the model.
    headless.click(PAD + BUTTON + GAP + BUTTON / 2.0, bar_mid);
    settle(&mut headless);
    assert_eq!(
        (harness.get_last_id().as_str(), harness.get_activations()),
        ("beta", 1),
        "a click on a button item"
    );
    headless.press_key(Key::Tab);
    settle(&mut headless);
    headless.press_key(Key::DownArrow);
    settle(&mut headless);
    headless.wait(FADE);
    settle(&mut headless);
    headless.press_key(Key::Return);
    settle(&mut headless);
    assert_eq!(
        (
            harness.get_last_id().as_str(),
            harness.get_last_checked(),
            harness.get_toggles()
        ),
        ("gamma", true, 1),
        "a folded toggle flips from the «⋯» menu"
    );
    assert!(
        harness.get_gamma_checked(),
        "its row of the model holds the state"
    );

    // The menu gave the focus back to «⋯»: ↓ opens it again, ↓ goes to Delta, Enter chooses it.
    headless.press_key(Key::DownArrow);
    settle(&mut headless);
    headless.wait(FADE);
    settle(&mut headless);
    headless.press_key(Key::DownArrow);
    settle(&mut headless);
    headless.press_key(Key::Return);
    settle(&mut headless);
    assert_eq!(
        (harness.get_last_id().as_str(), harness.get_activations()),
        ("delta", 2),
        "a folded button is chosen from the «⋯» menu"
    );

    // Wide again, every item is back, and a click on the toggle in the bar flips it off.
    harness.set_bar_width(WIDTH as f32);
    settle(&mut headless);
    assert_eq!(harness.get_bar_shown(), 5, "every item is back");
    let gamma_x = PAD + BUTTON + GAP + BUTTON + SEPARATOR;
    headless.click(gamma_x + MORE / 2.0, bar_mid);
    settle(&mut headless);
    assert_eq!(
        (harness.get_last_checked(), harness.get_toggles()),
        (false, 2),
        "a click on the toggle in the bar"
    );
    assert!(!harness.get_gamma_checked(), "its row of the model follows");

    // Too narrow for any item: the «⋯» alone.
    harness.set_bar_width(BAR_NONE);
    settle(&mut headless);
    assert_eq!(harness.get_bar_shown(), 0, "every item folds");
    assert!(harness.get_bar_overflowing(), "the «⋯» stays");

    // PanelHeader: `tabbar` tall; an options menu alone keeps the «⋯», at the header's end.
    let header_mid = harness.get_header_y() + TABBAR / 2.0;
    assert_eq!(
        harness.get_header_height(),
        TABBAR,
        "a panel header is `tabbar` tall"
    );
    assert!(
        harness.get_header_overflowing(),
        "`more-entries` keep the «⋯»"
    );
    assert_eq!(harness.get_header_shown(), 0, "no items");
    headless.click(HEADER - PAD - MORE_SM / 2.0, header_mid);
    settle(&mut headless);
    headless.wait(FADE);
    settle(&mut headless);
    headless.press_key(Key::DownArrow);
    settle(&mut headless);
    headless.press_key(Key::Return);
    settle(&mut headless);
    assert_eq!(
        harness.get_header_chosen().as_str(),
        "style",
        "`more-activated` for an option"
    );

    // StatusBar: `statusbar` tall; at the window's width every text is whole.
    let status_mid = harness.get_status_y() + STATUSBAR / 2.0;
    assert_eq!(
        harness.get_status_height(),
        STATUSBAR,
        "a status bar is `statusbar` tall"
    );
    assert!(
        !harness.get_first_cut() && !harness.get_later_cut() && !harness.get_never_cut(),
        "every text is whole"
    );
    let natural = harness.get_status_natural();
    let first = harness.get_first_width();
    let later = harness.get_later_width();
    let never = harness.get_never_width();
    assert!(
        first > ITEM_MIN + 50.0,
        "the first text is long enough to shorten: {first}"
    );

    // A little short: only the `first` text gives way.
    harness.set_status_width(natural - 50.0);
    settle(&mut headless);
    assert!(
        near(harness.get_first_width(), first - 50.0),
        "the first text gives way first"
    );
    assert!(
        near(harness.get_later_width(), later),
        "the later one stays whole"
    );
    assert!(
        harness.get_first_cut() && !harness.get_later_cut(),
        "only the first is cut"
    );

    // Shorter: the first text is at its minimum, the later one gives way, the unit stays whole.
    harness.set_status_width(natural - (first - ITEM_MIN) - 30.0);
    settle(&mut headless);
    assert!(
        near(harness.get_first_width(), ITEM_MIN),
        "the first text at `statusbar-item-min-w`"
    );
    assert!(
        near(harness.get_later_width(), later - 30.0),
        "then the later one gives way"
    );
    assert!(harness.get_later_cut(), "and is cut");
    assert!(
        near(harness.get_never_width(), never) && !harness.get_never_cut(),
        "a `never` item stays whole"
    );

    // The cut text says itself whole in its tooltip.
    headless.hover(PAD + ITEM_MIN / 2.0, status_mid);
    settle(&mut headless);
    headless.wait(AFTER_DELAY);
    settle(&mut headless);
    assert!(
        harness.get_first_tooltip(),
        "a cut status text says itself in the tooltip"
    );

    // The live state is a button.
    let live_x = PAD + ITEM_MIN + SEPARATOR_STATUS + (later - 30.0) + SEPARATOR_STATUS;
    headless.click(live_x + 10.0, status_mid);
    settle(&mut headless);
    assert_eq!(
        harness.get_live_clicks(),
        1,
        "a click on a status button item"
    );
}
