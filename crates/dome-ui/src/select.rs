//! The Select's answers (`docs/design-system.md` § 5.5) that Slint cannot compute: its list with the chosen item
//! marked, and the type-ahead on the closed control. Pure functions over the menu's plain entries; `select_nav!`
//! wires them to an app's `SelectNav` global.

use crate::menu::{MenuEntry, MenuEntryKind, typeahead_among};

/// The entries as the Select's list draws them: every item becomes a check box item, checked only at `chosen`, so
/// the chosen one carries the `check` and screen readers hear which one it is. Separators and titles stay as they
/// are; with `chosen` -1 (nothing chosen, the placeholder shows) no item is checked.
pub fn marked(entries: &[MenuEntry], chosen: i32) -> Vec<MenuEntry> {
    entries
        .iter()
        .enumerate()
        .map(|(index, entry)| match entry.kind {
            MenuEntryKind::Item | MenuEntryKind::Checkbox | MenuEntryKind::Radio => MenuEntry {
                kind: MenuEntryKind::Checkbox,
                checked: i32::try_from(index).is_ok_and(|index| index == chosen),
                ..entry.clone()
            },
            MenuEntryKind::Separator | MenuEntryKind::Section => entry.clone(),
        })
        .collect()
}

/// The item the closed Select moves to as the user types, as the menu's type-ahead (`menu::typeahead`) but over the
/// enabled items only: a disabled item can be highlighted in the open list, never chosen. -1 if none matches.
pub fn typeahead(entries: &[MenuEntry], from: i32, typed: &str) -> i32 {
    typeahead_among(entries, from, typed, |entry| !entry.disabled)
}

#[cfg(test)]
mod tests {
    use super::{MenuEntry, MenuEntryKind, marked, typeahead};

    fn entry(kind: MenuEntryKind, label: &str, disabled: bool) -> MenuEntry {
        MenuEntry {
            kind,
            label: label.into(),
            disabled,
            ..MenuEntry::default()
        }
    }

    /// 0 Size, 1 Name, 2 —, 3 «Dates» (title), 4 Modified, 5 Allocated (disabled), 6 Accessed
    fn select() -> Vec<MenuEntry> {
        vec![
            entry(MenuEntryKind::Item, "Size", false),
            entry(MenuEntryKind::Item, "Name", false),
            entry(MenuEntryKind::Separator, "", false),
            entry(MenuEntryKind::Section, "Dates", false),
            entry(MenuEntryKind::Item, "Modified", false),
            entry(MenuEntryKind::Item, "Allocated", true),
            entry(MenuEntryKind::Item, "Accessed", false),
        ]
    }

    #[test]
    fn only_the_chosen_item_is_checked() {
        let shown = marked(&select(), 4);
        let checked: Vec<bool> = shown.iter().map(|e| e.checked).collect();
        assert_eq!(checked, [false, false, false, false, true, false, false]);
        let kinds: Vec<MenuEntryKind> = shown.iter().map(|e| e.kind).collect();
        assert_eq!(
            kinds,
            [
                MenuEntryKind::Checkbox,
                MenuEntryKind::Checkbox,
                MenuEntryKind::Separator,
                MenuEntryKind::Section,
                MenuEntryKind::Checkbox,
                MenuEntryKind::Checkbox,
                MenuEntryKind::Checkbox,
            ],
            "items carry the check slot; separators and titles stay"
        );
        assert_eq!(shown[5].label, "Allocated");
        assert!(shown[5].disabled, "the rest of an entry is kept");
    }

    #[test]
    fn nothing_chosen_checks_nothing() {
        assert!(marked(&select(), -1).iter().all(|e| !e.checked));
        assert!(marked(&select(), 99).iter().all(|e| !e.checked));
    }

    #[test]
    fn typeahead_skips_disabled_items_and_titles() {
        let s = select();
        assert_eq!(typeahead(&s, 0, "a"), 6, "Allocated is disabled: Accessed");
        assert_eq!(typeahead(&s, -1, "d"), -1, "a title is never chosen");
        assert_eq!(typeahead(&s, 4, "s"), 0, "goes round");
        assert_eq!(typeahead(&s, 1, "n"), 1, "the only match stays");
        assert_eq!(typeahead(&s, 0, "mo"), 4);
        assert_eq!(typeahead(&s, 0, "x"), -1);
    }
}
