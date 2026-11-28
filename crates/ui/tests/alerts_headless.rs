//! InfoBar and Toast off screen (`docs/design-system.md` § 5.19, § 5.20, § 3.11), on their test window
//! `KitAlertsHarness`: an info bar one line tall that grows when its text wraps and says its action and its `x`; a
//! toast whose time follows the spec, placed in the corner above the bar under it, closed by its `x`, Esc and its
//! actions, replaced by the next one, going by itself by the clock, stopping under the pointer and with the focus in
//! it, and waiting hidden while a popup is open.
//!
//! `DISKDOME_KIT_SHOTS=<folder>` also saves the window with a toast, for a look.

#![cfg(feature = "kit")]

use std::path::PathBuf;
use std::time::Duration;

use diskdome_ui::design_system;
use diskdome_ui::generated::{KitAlertsHarness, Theme};
use diskdome_ui::headless::Headless;
use slint::ComponentHandle;
use slint::platform::Key;

const WIDTH: u32 = 960;
const HEIGHT: u32 = 600;

/// An info bar with one line of text (§ 5.19: 32 at normal density).
const INFO_BAR: f32 = 32.0;
/// The `body` step's line (§ 3.3): a wrapped info bar is at least half a line taller than one line.
const BODY_LINE: f32 = 18.0;
/// The status bar under the toast's area (`statusbar`, § 3.5, normal density).
const STATUS_BAR: f32 = 22.0;
/// The toast's width and its distance from the right edge and from the bar under it (§ 5.20, § 3.11).
const TOAST_W: f32 = 360.0;
const NOTICE_MARGIN: f32 = 16.0;
/// Inside the card: its 1 px edge (§ 3.8), `toast-pad-x` 10 and `toast-pad-y` 6 (pending design), and the first
/// line as tall as the buttons on it (`control-sm`, 24 at normal density).
const EDGE: f32 = 1.0;
const PAD_X: f32 = 10.0;
const PAD_Y: f32 = 6.0;
const LINE_BOX: f32 = 24.0;
/// The close button (§ 5.20: 20 px), and the gap before it (`label-gap`, 6).
const CLOSE: f32 = 20.0;
const GAP: f32 = 6.0;
/// The row of actions under the text: `toast-gap` 2 (pending design) above it, `control-sm` tall; each action's word
/// lines up with the text, which starts after the 16 px icon and the gap (`toast-action-pad-x` 4 of padding in front).
const ROW_GAP: f32 = 2.0;
const ROW_START: f32 = 16.0 + GAP - 4.0;
/// A card with one line of text: its edges, its padding and the first line.
const ONE_LINE_CARD: f32 = 2.0 * EDGE + 2.0 * PAD_Y + LINE_BOX;

/// A notice of a few words: 2.5 s + 14 × 60 ms = 3.34 s, under the minimum, so it stays 4 s (§ 5.20).
const SHORT: &str = "Copied 3 items";
const MIN_TIME: Duration = Duration::from_millis(4000);
/// How far from the end of a notice's time the test looks: well inside it, and well past it, for a busy CI machine.
const MARGIN: Duration = Duration::from_millis(600);
/// Longer than `Motion.base` (140 ms), the card's fade and rise.
const FADE: Duration = Duration::from_millis(300);
/// A point of the toast's area away from the card.
const NOWHERE_X: f32 = 100.0;

fn save_shot(headless: &Headless, name: &str) {
    let Some(folder) = std::env::var_os("DISKDOME_KIT_SHOTS") else {
        return;
    };
    let path = PathBuf::from(folder).join(format!("alerts-{name}.bmp"));
    std::fs::write(&path, headless.bmp()).expect("the shots folder is writable");
}

/// Two turns of the loop: a timer or a change handler is seen in the next one.
fn settle(headless: &mut Headless) {
    headless.frame();
    headless.frame();
}

fn show(harness: &KitAlertsHarness, text: &str, action: &str, rows: i32, sticky: bool) {
    harness.invoke_show_notice(text.into(), action.into(), rows, sticky);
}

/// The time a notice gets, by the toast's own clock.
fn time_of(harness: &KitAlertsHarness, text: &str, action: &str, sticky: bool) -> i64 {
    show(harness, text, action, 0, sticky);
    harness.get_toast_time()
}

/// One test: the headless platform is installed once per thread.
#[test]
fn info_bars_and_toasts() {
    let mut headless = Headless::install(WIDTH, HEIGHT).expect("headless platform");
    let harness = KitAlertsHarness::new().expect("the harness compiles");
    design_system::install(&harness);
    harness.show().expect("the harness shows off screen");
    settle(&mut headless);

    // InfoBar: one line is `info-bar` tall; a long text wraps and grows the bar; the area starts under both.
    assert_eq!(harness.get_bar_height(), INFO_BAR);
    assert!(
        harness.get_long_bar_height() > INFO_BAR + BODY_LINE / 2.0,
        "a long text wraps and makes the bar taller ({})",
        harness.get_long_bar_height()
    );
    assert_eq!(
        harness.get_area_y(),
        harness.get_bar_height() + harness.get_long_bar_height()
    );
    assert_eq!(harness.get_status_y(), HEIGHT as f32 - STATUS_BAR);

    // The toast's time (§ 5.20): 2.5 s + 60 ms a character, at least 4 s (8 s with an action), at most 10 s;
    // a sticky one stays until closed (0).
    let ten = "x".repeat(10);
    let hundred = "x".repeat(100);
    let two_hundred = "x".repeat(200);
    // 2.5 + 0.6 = 3.1 s: the minimum.
    assert_eq!(time_of(&harness, &ten, "", false), 4000);
    // 2.5 + 6 = 8.5 s.
    assert_eq!(time_of(&harness, &hundred, "", false), 8500);
    // 2.5 + 12 = 14.5 s: the maximum.
    assert_eq!(time_of(&harness, &two_hundred, "", false), 10000);
    // With an action: at least 8 s.
    assert_eq!(time_of(&harness, &ten, "Undo", false), 8000);
    assert_eq!(time_of(&harness, &hundred, "Undo", false), 8500);
    assert_eq!(time_of(&harness, &ten, "Show details", true), 0);
    // Each notice replaced the one before; closing says so.
    assert_eq!(harness.get_replacements(), 5);
    harness.invoke_close_notice();
    assert!(!harness.get_toast_open(), "close() puts the notice away");
    assert_eq!(harness.get_dismissals(), 1);

    // A sticky notice does not go by itself.
    show(&harness, SHORT, "", 0, true);
    settle(&mut headless);
    headless.wait(FADE);
    assert!(harness.get_toast_shown(), "a sticky notice stays");
    harness.invoke_close_notice();
    assert_eq!(harness.get_dismissals(), 2);

    // The keyboard focus in the toast stops its time: Tab reaches its `x` (the toast's buttons are the first stops),
    // and the notice stays past its time; Enter on the `x` closes it.
    show(&harness, SHORT, "", 0, false);
    settle(&mut headless);
    headless.press_key(Key::Tab);
    settle(&mut headless);
    headless.wait(MIN_TIME + MARGIN);
    assert!(
        harness.get_toast_shown(),
        "the focus in the toast stops its time"
    );
    headless.press_key(Key::Return);
    settle(&mut headless);
    assert!(
        !harness.get_toast_open(),
        "Enter on the x closes the notice"
    );
    assert_eq!(harness.get_dismissals(), 3);

    // Esc from a button of the toast closes it.
    show(&harness, "Moved 3 items", "Undo", 0, false);
    settle(&mut headless);
    headless.press_key(Key::Tab);
    settle(&mut headless);
    headless.press_key(Key::Escape);
    settle(&mut headless);
    assert!(!harness.get_toast_open(), "Esc closes the notice");
    assert_eq!(harness.get_dismissals(), 4);

    // Placement (§ 3.11): 360 wide, 16 px from the right edge and 16 px above the status bar.
    show(&harness, "Moved 3 items", "Undo", 0, false);
    settle(&mut headless);
    headless.wait(FADE);
    let card_x = harness.get_card_x();
    let card_y = harness.get_card_y();
    assert_eq!(harness.get_card_width(), TOAST_W);
    assert_eq!(card_x, WIDTH as f32 - NOTICE_MARGIN - TOAST_W);
    assert_eq!(harness.get_card_height(), ONE_LINE_CARD);
    assert_eq!(
        card_y + harness.get_card_height(),
        harness.get_status_y() - NOTICE_MARGIN
    );
    // It is drawn: the level-1 fill inside its padding.
    let raised = harness.global::<Theme>().get_raised();
    let inside = headless.pixel(
        (card_x + EDGE + PAD_X / 2.0) as usize,
        (card_y + ONE_LINE_CARD / 2.0) as usize,
    );
    assert_eq!(
        inside,
        [raised.red(), raised.green(), raised.blue(), 255],
        "the card is drawn in `raised`"
    );
    save_shot(&headless, "toast");

    // A click on Undo (left of the `x` and the gap) chooses it: the action is said and the notice goes.
    let first_line = card_y + EDGE + PAD_Y + LINE_BOX / 2.0;
    let close_x = card_x + TOAST_W - EDGE - PAD_X - CLOSE / 2.0;
    let undo_end = card_x + TOAST_W - EDGE - PAD_X - CLOSE - GAP;
    headless.click(undo_end - GAP, first_line);
    settle(&mut headless);
    assert_eq!(harness.get_action_clicks(), 1);
    assert_eq!(harness.get_actions(), 1);
    assert!(!harness.get_toast_open(), "an action closes the notice");

    // The `x` by the pointer.
    show(&harness, SHORT, "", 0, false);
    settle(&mut headless);
    headless.click(close_x, first_line);
    settle(&mut headless);
    assert!(!harness.get_toast_open(), "the x closes the notice");
    assert_eq!(harness.get_dismissals(), 5);

    // The row under the text: its first action, by the pointer.
    show(&harness, "Exported C.csv", "", 2, false);
    settle(&mut headless);
    assert_eq!(
        harness.get_card_height(),
        ONE_LINE_CARD + ROW_GAP + LINE_BOX
    );
    let row_y = harness.get_card_y() + EDGE + PAD_Y + LINE_BOX + ROW_GAP + LINE_BOX / 2.0;
    headless.click(card_x + EDGE + PAD_X + ROW_START + GAP, row_y);
    settle(&mut headless);
    assert_eq!(harness.get_row_clicks(), 1);
    assert_eq!(harness.get_last_row_index(), 0);
    assert_eq!(harness.get_actions(), 2);

    // One at a time: a new notice replaces the one shown.
    show(&harness, "First", "", 0, false);
    show(&harness, "Second", "", 0, false);
    settle(&mut headless);
    assert_eq!(harness.get_replacements(), 6);
    assert_eq!(harness.get_toast_text(), "Second");
    assert!(harness.get_toast_shown());
    harness.invoke_close_notice();
    assert_eq!(harness.get_dismissals(), 6);

    // By the clock: a notice of a few words stays its 4 s, then goes by itself.
    show(&harness, SHORT, "", 0, false);
    settle(&mut headless);
    headless.wait(MIN_TIME - MARGIN);
    assert!(harness.get_toast_shown(), "the notice stays its time");
    headless.wait(MARGIN * 2);
    assert!(!harness.get_toast_open(), "the notice goes by itself");
    assert_eq!(harness.get_timeouts(), 1);

    // Under the pointer its time stops, and goes on from where it was when the pointer leaves: shown 2 s, hovered
    // past the whole time, then 2 s are left.
    let half = MIN_TIME / 2;
    let away_y = harness.get_area_y() + NOTICE_MARGIN;
    show(&harness, SHORT, "", 0, false);
    settle(&mut headless);
    headless.wait(half);
    headless.hover(
        harness.get_card_x() + TOAST_W / 2.0,
        harness.get_card_y() + harness.get_card_height() / 2.0,
    );
    headless.wait(half + MARGIN * 2);
    assert!(
        harness.get_toast_shown(),
        "the pointer over it stops its time"
    );
    headless.hover(NOWHERE_X, away_y);
    headless.wait(half - MARGIN);
    assert!(
        harness.get_toast_shown(),
        "its time goes on where it stopped"
    );
    headless.wait(MARGIN * 2);
    assert!(
        !harness.get_toast_open(),
        "the rest of its time, not the whole time again"
    );
    assert_eq!(harness.get_timeouts(), 2);

    // Held by a popup it hides and its time stands still; it comes back with its whole time (§ 5.20): shown 2 s,
    // held 1 s, then 4 s again.
    show(&harness, SHORT, "", 0, false);
    settle(&mut headless);
    headless.wait(half);
    harness.set_held(true);
    settle(&mut headless);
    assert!(
        harness.get_toast_open() && !harness.get_toast_shown(),
        "held, the notice waits hidden"
    );
    headless.wait(Duration::from_millis(1000));
    harness.set_held(false);
    settle(&mut headless);
    headless.wait(MIN_TIME - MARGIN);
    assert!(
        harness.get_toast_shown(),
        "back from under a popup, the notice has its whole time"
    );
    headless.wait(MARGIN * 2);
    assert!(!harness.get_toast_open());
    assert_eq!(harness.get_timeouts(), 3);

    // The info bar's buttons, by the pointer: the `x` at the right end, the action just before it.
    let bar_middle = INFO_BAR / 2.0;
    // The `x`: `control-sm` (24) square, as far from the right edge as from the top ((32 − 24) / 2 = 4).
    let bar_close_x = WIDTH as f32 - 4.0 - LINE_BOX / 2.0;
    headless.click(bar_close_x, bar_middle);
    settle(&mut headless);
    assert_eq!(harness.get_bar_closes(), 1);
    let bar_action_end = WIDTH as f32 - 4.0 - LINE_BOX - GAP;
    headless.click(bar_action_end - GAP, bar_middle);
    settle(&mut headless);
    assert_eq!(harness.get_bar_actions(), 1);
}
