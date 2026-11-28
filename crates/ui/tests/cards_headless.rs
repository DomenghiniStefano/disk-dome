//! EmptyState, Card and OptionCard off screen (`docs/design-system.md` § 5.9, § 5.22, § 5.23), on their test window
//! `KitCardsHarness`: option cards of the spec's height and gap, chosen by click and arrows (skipping the disabled
//! one, going round the ends, Enter doing nothing), the chosen one's `accent` edge, the disabled one saying why; two
//! cards side by side sharing the row equally and ending together; an empty state's action centred in the whole
//! area; a card's header and content at the spec's sizes, its action working, and cards of a row growing to the
//! tallest.

#![cfg(feature = "kit")]

use std::time::Duration;

use diskdome_ui::design_system;
use diskdome_ui::generated::KitCardsHarness;
use diskdome_ui::headless::Headless;
use slint::ComponentHandle;
use slint::platform::Key;

const WIDTH: u32 = 960;
const HEIGHT: u32 = 800;
/// An option card on one line (§ 5.9): `card-pad` 10, the title's `body-strong` line 18, the gap to the description
/// `option-card-text-gap` 2 (pending design), the description's `caption` line 16, `card-pad` 10: 56 tall.
const OPTION: f32 = 10.0 + 18.0 + 2.0 + 16.0 + 10.0;
/// Between two option cards: `option-card-gap` (pending design, 8).
const OPTION_GAP: f32 = 8.0;
/// The side-by-side group is `dialog-md` (520) wide: two cards that ask `option-card-w` (240) each share the rest
/// equally, so each is (520 − 8) / 2 = 256 wide, the second starting at 264.
const PAIR_WIDTH: f32 = 520.0;
const PAIR_CARD: f32 = (PAIR_WIDTH - OPTION_GAP) / 2.0;
/// The empty state (§ 5.22): `control-lg` × 7 = 224 tall here, content = icon 32 + title 26 + description 18 + the
/// large button 32, three `empty-gap`s of 8 and the extra 8 above the button (`section-gap` 16 in all) = 140,
/// centred: the button's centre is (224 − 140) / 2 + 140 − 16 = 166 below the top. The labels wrap, so they step
/// by the font's own line: a font a pixel off moves the button by half of it, far inside its 32 px.
const EMPTY_BUTTON_Y: f32 = (224.0 - 140.0) / 2.0 + 140.0 - 16.0;
/// Where a button aligned at the empty state's start would be: past its `section-gap` inset (16), inside the button's
/// 64 px minimum.
const EMPTY_START_X: f32 = 16.0 + 32.0;
/// A card (§ 5.23): `card-pad` 10, the header as tall as its small action button (`control-sm` 24), `card-gap` 8
/// (pending design), the content (`control-lg` 32), `card-pad` 10 = 84; it is `side-panel` (300) wide.
const CARD: f32 = 10.0 + 24.0 + 8.0 + 32.0 + 10.0;
const CARD_WIDTH: f32 = 300.0;
/// The taller card of the row: 10 + `control-lg` × 2 (64) + 10, with no header.
const TALL: f32 = 10.0 + 64.0 + 10.0;
/// The dark theme's colours (§ 3.2): `line` (a card's edge), `accent` (the chosen card's edge), `surface` (a card's
/// fill at rest).
const LINE: [u8; 4] = [0x28, 0x2a, 0x2d, 0xff];
const ACCENT: [u8; 4] = [0x37, 0xc2, 0xbb, 0xff];
const SURFACE: [u8; 4] = [0x16, 0x17, 0x19, 0xff];
/// Past `Motion.fast` (90 ms), so colours have arrived.
const AFTER_FAST: Duration = Duration::from_millis(300);
/// Past `Motion.tooltip-delay` (500 ms), with a margin for a busy CI machine.
const AFTER_DELAY: Duration = Duration::from_millis(700);
/// A point far from every block, where the pointer rests to leave no hover.
const AWAY: (f32, f32) = (900.0, 790.0);

/// Two turns of the loop: a timer or a change handler is seen in the next one.
fn settle(headless: &mut Headless) {
    headless.frame();
    headless.frame();
}

/// Moves the pointer away and lets the colours arrive, so a pixel shows the rest look.
fn rest(headless: &mut Headless) {
    headless.hover(AWAY.0, AWAY.1);
    settle(headless);
    headless.wait(AFTER_FAST);
}

/// One test: the headless platform is installed once per thread.
#[test]
fn option_cards_choose_cards_hold_content_and_empty_states_centre() {
    let mut headless = Headless::install(WIDTH, HEIGHT).expect("headless platform");
    let harness = KitCardsHarness::new().expect("the harness compiles");
    design_system::install(&harness);
    harness.show().expect("the harness shows off screen");
    settle(&mut headless);
    let ground = headless.pixel(AWAY.0 as usize, AWAY.1 as usize);

    // Option cards: three of the spec's height, `option-card-gap` apart; nothing chosen, the keyboard stop is the
    // first enabled card.
    assert_eq!(
        harness.get_option_height(),
        3.0 * OPTION + 2.0 * OPTION_GAP,
        "three one-line option cards and their gaps"
    );
    let top = |index: f32| harness.get_option_y() + index * (OPTION + OPTION_GAP);
    let mid = |index: f32| top(index) + OPTION / 2.0;
    assert_eq!(harness.get_option_index(), -1);
    assert_eq!(harness.get_option_focus(), 0);
    rest(&mut headless);
    assert_eq!(
        headless.pixel(0, mid(0.0) as usize),
        LINE,
        "a card's edge at rest"
    );
    assert_eq!(
        headless.pixel(
            (harness.get_option_width() - 4.0) as usize,
            (top(0.0) + 4.0) as usize
        ),
        SURFACE,
        "a card's fill at rest"
    );

    // A click anywhere on a card chooses it; its edge turns `accent`.
    headless.click(harness.get_option_width() - 20.0, mid(2.0));
    settle(&mut headless);
    assert_eq!(harness.get_option_index(), 2, "a click chooses a card");
    assert_eq!(harness.get_option_picks(), 1);
    rest(&mut headless);
    assert_eq!(
        headless.pixel(0, mid(2.0) as usize),
        ACCENT,
        "the chosen card's edge"
    );
    assert_eq!(headless.pixel(0, mid(0.0) as usize), LINE);

    // ↓ goes round the end; ↓ again skips the disabled card; ↑ comes back; ← goes round the other way.
    headless.press_key(Key::DownArrow);
    settle(&mut headless);
    assert_eq!(
        harness.get_option_index(),
        0,
        "the arrows go round the ends"
    );
    headless.press_key(Key::DownArrow);
    settle(&mut headless);
    assert_eq!(
        harness.get_option_index(),
        2,
        "the arrows skip a disabled card"
    );
    headless.press_key(Key::UpArrow);
    settle(&mut headless);
    assert_eq!(harness.get_option_index(), 0);
    headless.press_key(Key::LeftArrow);
    settle(&mut headless);
    assert_eq!(harness.get_option_index(), 2);
    assert_eq!(
        harness.get_option_picks(),
        5,
        "every new choice says `selected`"
    );
    // Enter does nothing (a radio's, as Radix); Space on the chosen card changes nothing.
    headless.press_key(Key::Return);
    headless.press_key(Key::Space);
    settle(&mut headless);
    assert_eq!(harness.get_option_index(), 2);
    assert_eq!(harness.get_option_picks(), 5);

    // The disabled card is not chosen by a click, and says why on hover.
    headless.click(harness.get_option_width() - 20.0, mid(1.0));
    settle(&mut headless);
    assert_eq!(
        harness.get_option_index(),
        2,
        "a disabled card is not chosen"
    );
    headless.hover(harness.get_option_width() - 20.0, mid(1.0));
    settle(&mut headless);
    headless.wait(AFTER_DELAY);
    settle(&mut headless);
    assert_eq!(harness.get_option_tooltip(), 1, "a disabled card says why");
    rest(&mut headless);

    // Side by side: the two cards share the row equally and end together, the short one grown to the tall one.
    assert_eq!(harness.get_pair_width(), PAIR_WIDTH);
    assert!(
        harness.get_pair_height() > OPTION,
        "the long description wraps"
    );
    let pair_mid = (harness.get_pair_y() + harness.get_pair_height() / 2.0) as usize;
    assert_eq!(
        headless.pixel(0, pair_mid),
        LINE,
        "the first card's left edge"
    );
    assert_eq!(
        headless.pixel((PAIR_CARD - 1.0) as usize, pair_mid),
        LINE,
        "the first card's right edge"
    );
    assert_eq!(
        headless.pixel((PAIR_CARD + OPTION_GAP / 2.0) as usize, pair_mid),
        ground,
        "the gap between the cards"
    );
    assert_eq!(
        headless.pixel((PAIR_CARD + OPTION_GAP) as usize, pair_mid),
        LINE,
        "the second card's left edge"
    );
    assert_eq!(
        headless.pixel((PAIR_WIDTH - 1.0) as usize, pair_mid),
        LINE,
        "the second card's right edge"
    );
    // Just above the row's bottom, far under the short card's one-line content: still inside its fill.
    let pair_bottom = harness.get_pair_y() + harness.get_pair_height() - 4.0;
    let second_x = (PAIR_CARD + OPTION_GAP + PAIR_CARD / 2.0) as usize;
    assert_eq!(
        headless.pixel(second_x, pair_bottom as usize),
        SURFACE,
        "the short card ends with the tall one"
    );

    // The empty state's action sits in the middle of the whole width, not at its start.
    let button_y = harness.get_empty_y() + EMPTY_BUTTON_Y;
    headless.click(EMPTY_START_X, button_y);
    settle(&mut headless);
    assert_eq!(
        harness.get_empty_clicks(),
        0,
        "nothing at the start of the row"
    );
    headless.click(WIDTH as f32 / 2.0, button_y);
    settle(&mut headless);
    assert_eq!(
        harness.get_empty_clicks(),
        1,
        "the action is centred in the area"
    );

    // The card: header and content at the spec's sizes, `line` edge, `surface` fill, the action at the header's end.
    assert_eq!(harness.get_card_width(), CARD_WIDTH);
    assert_eq!(
        harness.get_card_height(),
        CARD,
        "pad, header, gap, content, pad"
    );
    rest(&mut headless);
    let card_y = harness.get_card_y();
    assert_eq!(
        headless.pixel(0, (card_y + CARD / 2.0) as usize),
        LINE,
        "the card's edge"
    );
    assert_eq!(
        headless.pixel((CARD_WIDTH / 2.0) as usize, (card_y + 4.0) as usize),
        SURFACE,
        "the card's fill"
    );
    // The action is a small button ending `card-pad` before the card's right edge, on the header's line.
    headless.click(CARD_WIDTH - 10.0 - 8.0, card_y + 10.0 + 12.0);
    settle(&mut headless);
    assert_eq!(harness.get_card_clicks(), 1, "the header's action works");

    // Cards of a row end together: the shorter content's card grows to the taller one's height.
    assert_eq!(harness.get_tall_height(), TALL);
    assert_eq!(
        harness.get_short_height(),
        TALL,
        "a card grows to the row's tallest"
    );
}
