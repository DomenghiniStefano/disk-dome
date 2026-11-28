//! The toolbar's fit (`docs/design-system.md` § 5.25): how many of a `ToolBar`'s or `PanelHeader`'s items show before
//! the rest fold into its «⋯» menu, which Slint cannot work out itself since its functions have no loops. Pure
//! functions over the items' natural widths, measured by the Slint half; `tool_bar_fit!` wires them to an app's
//! `ToolBarFit` global, with [`crate::breadcrumb::with_width`] and [`crate::breadcrumb::total`] for the widths.

use std::ops::Range;

/// Lengths from Slint's layout are floats: a row this close to its width still fits.
const FIT_TOLERANCE: f32 = 0.01;

/// How many items show, from the first. All of them when they fit `available` (with the «⋯» button after them when
/// `keep_more`, for an app menu that always shows). Otherwise the most that fit with `gap` and the «⋯» button (`more`)
/// after them, or the button alone when none does. A separator never ends the row: it folds with the item after it.
///
/// Each width is an item with the gap before it (none before the first); `separators` says which items are
/// separators. Widths not yet measured count as 0.
pub fn shown_count(
    widths: &[f32],
    separators: &[bool],
    available: f32,
    gap: f32,
    more: f32,
    keep_more: bool,
) -> i32 {
    let count = separators.len();
    let width = |index: usize| widths.get(index).copied().unwrap_or(0.0);
    let prefix = |shown: usize| (0..shown).map(width).sum::<f32>();
    let fits = |row: f32| row <= available + FIT_TOLERANCE;
    let with_more = |shown: usize| {
        if shown == 0 {
            more
        } else {
            prefix(shown) + gap + more
        }
    };
    if (!keep_more && fits(prefix(count))) || (keep_more && fits(with_more(count))) {
        return to_index(count);
    }
    let mut shown = (0..count)
        .rev()
        .find(|&shown| fits(with_more(shown)))
        .unwrap_or(0);
    while shown > 0 && separators[shown - 1] {
        shown -= 1;
    }
    to_index(shown)
}

/// The folded items: from `shown_count` to the end. Empty when everything shows.
pub fn folded(shown_count: i32, count: usize) -> Range<usize> {
    let start = usize::try_from(shown_count).unwrap_or(0).min(count);
    start..count
}

/// The index of the folded item whose id is `id`, or -1 when none is (the id is one of the menu's other entries).
pub fn folded_index(ids: &[&str], shown_count: i32, id: &str) -> i32 {
    folded(shown_count, ids.len())
        .find(|&index| ids[index] == id)
        .map_or(-1, to_index)
}

/// A count or an index for Slint.
fn to_index(value: usize) -> i32 {
    i32::try_from(value).unwrap_or(i32::MAX)
}

#[cfg(test)]
mod tests {
    use super::{folded, folded_index, shown_count};

    /// Four buttons 60 wide (40 for the first, which has no gap before it), then a separator (9) and a button.
    const WIDTHS: [f32; 6] = [40.0, 60.0, 60.0, 60.0, 9.0, 40.0];
    const SEPARATORS: [bool; 6] = [false, false, false, false, true, false];
    const GAP: f32 = 4.0;
    /// The «⋯» button.
    const MORE: f32 = 28.0;

    #[test]
    fn a_row_that_fits_folds_nothing() {
        // 40 + 60 * 3 + 9 + 40 = 269.
        assert_eq!(
            shown_count(&WIDTHS, &SEPARATORS, 269.0, GAP, MORE, false),
            6
        );
        assert!(folded(6, 6).is_empty());
    }

    #[test]
    fn the_last_items_fold_and_leave_room_for_the_menu() {
        // 255: not all, nor five with the menu (229 + 4 + 28 = 261); four with it: 40 + 60 * 3 + 4 + 28 = 252.
        assert_eq!(
            shown_count(&WIDTHS, &SEPARATORS, 255.0, GAP, MORE, false),
            4
        );
        assert_eq!(folded(4, 6), 4..6);
        // 40 + 60 + 4 + 28 = 132 fits two.
        assert_eq!(
            shown_count(&WIDTHS, &SEPARATORS, 140.0, GAP, MORE, false),
            2
        );
    }

    #[test]
    fn a_separator_never_ends_the_row() {
        // Five fit with the menu (40 + 180 + 9 + 4 + 28 = 261), but the fifth is the separator: the row stops at four.
        assert_eq!(
            shown_count(&WIDTHS, &SEPARATORS, 262.0, GAP, MORE, false),
            4
        );
    }

    #[test]
    fn the_menu_alone_when_nothing_fits() {
        assert_eq!(shown_count(&WIDTHS, &SEPARATORS, 50.0, GAP, MORE, false), 0);
        assert_eq!(shown_count(&WIDTHS, &SEPARATORS, 0.0, GAP, MORE, false), 0);
        assert_eq!(folded(0, 6), 0..6);
    }

    #[test]
    fn an_app_menu_keeps_room_for_its_button() {
        // Everything with the button: 269 + 4 + 28 = 301.
        assert_eq!(shown_count(&WIDTHS, &SEPARATORS, 301.0, GAP, MORE, true), 6);
        // 269 alone would fit without the button, but the button always shows.
        assert_eq!(shown_count(&WIDTHS, &SEPARATORS, 269.0, GAP, MORE, true), 4);
        // No items: the button alone.
        assert_eq!(shown_count(&[], &[], 10.0, GAP, MORE, true), 0);
    }

    #[test]
    fn widths_not_yet_measured_count_as_nothing() {
        assert_eq!(
            shown_count(&WIDTHS[..2], &SEPARATORS, 100.0, GAP, MORE, false),
            6
        );
    }

    #[test]
    fn a_folded_item_is_found_by_its_id() {
        let ids = ["back", "scan", "sep", "panel"];
        assert_eq!(folded_index(&ids, 2, "panel"), 3);
        assert_eq!(
            folded_index(&ids, 2, "back"),
            -1,
            "a shown item is not folded"
        );
        assert_eq!(folded_index(&ids, 2, "about"), -1, "an app menu entry");
        assert_eq!(folded_index(&ids, -1, "back"), 0);
    }
}
