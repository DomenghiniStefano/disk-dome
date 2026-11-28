//! The menu's walk over its entries (`docs/design-system.md` § 5.16): what Slint cannot compute itself, since its
//! functions have no loops and its strings no prefix test. Pure functions over plain entries; `menu_nav!` wires them
//! to an app's `MenuNav` global.

/// What a menu entry is, as the Slint `MenuEntryKind`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MenuEntryKind {
    #[default]
    Item,
    Checkbox,
    Radio,
    Separator,
    Section,
}

/// The fields of a Slint `MenuEntry` the walk reads.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MenuEntry {
    pub kind: MenuEntryKind,
    pub id: String,
    pub label: String,
    pub disabled: bool,
    pub checked: bool,
}

/// Whether the keyboard stops on an entry: every item, enabled or not (Windows highlights a disabled item, so its
/// reason can be read); never a separator or a section title.
fn is_item(entry: &MenuEntry) -> bool {
    match entry.kind {
        MenuEntryKind::Item | MenuEntryKind::Checkbox | MenuEntryKind::Radio => true,
        MenuEntryKind::Separator | MenuEntryKind::Section => false,
    }
}

/// The indices of the items, in order.
fn items(entries: &[MenuEntry]) -> Vec<i32> {
    entries
        .iter()
        .enumerate()
        .filter(|(_, entry)| is_item(entry))
        .filter_map(|(index, _)| i32::try_from(index).ok())
        .collect()
}

/// The first or the last item; -1 with none.
pub fn edge(entries: &[MenuEntry], last: bool) -> i32 {
    let items = items(entries);
    let found = if last { items.last() } else { items.first() };
    found.copied().unwrap_or(-1)
}

/// The item `by` entries away from `from` (±1 for an arrow, ± a page), on the nearest item in that direction. At the
/// end, `wrap` goes round to the other end; otherwise it stops on the last item reached. With nothing highlighted
/// (`from` < 0), going down starts on the first item and going up on the last.
pub fn step(entries: &[MenuEntry], from: i32, by: i32, wrap: bool) -> i32 {
    let items = items(entries);
    let (Some(&first), Some(&last)) = (items.first(), items.last()) else {
        return -1;
    };
    if from < 0 || by == 0 {
        return if by < 0 { last } else { first };
    }
    let target = from.saturating_add(by);
    if by > 0 {
        let next = items.iter().copied().find(|&index| index >= target);
        next.unwrap_or(if wrap && from >= last { first } else { last })
    } else {
        let previous = items.iter().rev().copied().find(|&index| index <= target);
        previous.unwrap_or(if wrap && from <= first { last } else { first })
    }
}

/// The next item whose label starts with what was typed, ignoring case, going round from `from`. More letters
/// narrow the search and keep the current item while it still matches; one letter typed again («s», «s») moves to
/// the next item starting with it. -1 if none matches.
pub fn typeahead(entries: &[MenuEntry], from: i32, typed: &str) -> i32 {
    typeahead_among(entries, from, typed, |_| true)
}

/// [`typeahead`] over the items `stops` also accepts (the closed Select skips disabled items, § 5.5).
pub(crate) fn typeahead_among(
    entries: &[MenuEntry],
    from: i32,
    typed: &str,
    stops: impl Fn(&MenuEntry) -> bool,
) -> i32 {
    let typed = typed.to_lowercase();
    let mut letters = typed.chars();
    let Some(first) = letters.next() else {
        return -1;
    };
    let repeated = letters.clone().next().is_some() && letters.all(|c| c == first);
    let search: String = if repeated {
        first.to_string()
    } else {
        typed.clone()
    };
    // A new search, or one letter repeated, starts after the current item; a longer search starts on it.
    let starts_after = repeated || typed.chars().count() == 1;
    let items: Vec<i32> = items(entries)
        .into_iter()
        .filter(|&index| {
            usize::try_from(index)
                .ok()
                .and_then(|i| entries.get(i))
                .is_some_and(&stops)
        })
        .collect();
    let start = items
        .iter()
        .position(|&index| {
            if starts_after {
                index > from
            } else {
                index >= from
            }
        })
        .unwrap_or(0);
    items
        .iter()
        .cycle()
        .skip(start)
        .take(items.len())
        .copied()
        .find(|&index| {
            usize::try_from(index)
                .ok()
                .and_then(|i| entries.get(i))
                .is_some_and(|entry| entry.label.to_lowercase().starts_with(&search))
        })
        .unwrap_or(-1)
}

/// Whether a key's text goes to the type-ahead: one character, not a control character, and not one of Slint's
/// special keys (arrows, function keys…), which it sends as private-use characters.
pub fn printable(text: &str) -> bool {
    let mut chars = text.chars();
    let (Some(c), None) = (chars.next(), chars.next()) else {
        return false;
    };
    !c.is_control() && !('\u{E000}'..='\u{F8FF}').contains(&c)
}

/// Whether an entry is drawn. A separator is not drawn at the start, after another separator, or with only
/// separators after it; everything else always is.
pub fn shown(entries: &[MenuEntry], index: i32) -> bool {
    let Ok(index) = usize::try_from(index) else {
        return false;
    };
    let Some(entry) = entries.get(index) else {
        return false;
    };
    let separator = |e: &MenuEntry| matches!(e.kind, MenuEntryKind::Separator);
    if !separator(entry) {
        return true;
    }
    let after_something = index > 0 && entries.get(index - 1).is_some_and(|e| !separator(e));
    let before_something = entries.iter().skip(index + 1).any(|e| !separator(e));
    after_something && before_something
}

/// The entries after the one with `id` was chosen: an enabled check box flips; an enabled radio item is checked and
/// the other radio items of its group are cleared. A group is the run of radio items the chosen one sits in, between
/// anything that is not a radio item (a separator, a title, a plain item). Any other entry, or an unknown `id`, leaves
/// the entries as they were.
pub fn toggled(entries: &[MenuEntry], id: &str) -> Vec<MenuEntry> {
    let chosen = entries
        .iter()
        .position(|entry| entry.id == id && !entry.disabled);
    let Some(chosen) = chosen else {
        return entries.to_vec();
    };
    match entries[chosen].kind {
        MenuEntryKind::Checkbox => entries
            .iter()
            .enumerate()
            .map(|(index, entry)| MenuEntry {
                checked: entry.checked != (index == chosen),
                ..entry.clone()
            })
            .collect(),
        MenuEntryKind::Radio => {
            let group = radio_group(entries, chosen);
            entries
                .iter()
                .enumerate()
                .map(|(index, entry)| MenuEntry {
                    checked: if group.contains(&index) {
                        index == chosen
                    } else {
                        entry.checked
                    },
                    ..entry.clone()
                })
                .collect()
        }
        MenuEntryKind::Item | MenuEntryKind::Separator | MenuEntryKind::Section => entries.to_vec(),
    }
}

/// The indices of the run of radio items around `index`.
fn radio_group(entries: &[MenuEntry], index: usize) -> std::ops::Range<usize> {
    let radio = |entry: &MenuEntry| matches!(entry.kind, MenuEntryKind::Radio);
    let start = entries[..index]
        .iter()
        .rposition(|entry| !radio(entry))
        .map_or(0, |before| before + 1);
    let end = entries[index..]
        .iter()
        .position(|entry| !radio(entry))
        .map_or(entries.len(), |after| index + after);
    start..end
}

#[cfg(test)]
mod tests {
    use super::{MenuEntry, MenuEntryKind};
    use super::{edge, printable, shown, step, toggled, typeahead};

    fn entry(kind: MenuEntryKind, label: &str) -> MenuEntry {
        MenuEntry {
            kind,
            label: label.into(),
            ..MenuEntry::default()
        }
    }

    fn item(label: &str) -> MenuEntry {
        entry(MenuEntryKind::Item, label)
    }

    fn separator() -> MenuEntry {
        entry(MenuEntryKind::Separator, "")
    }

    fn section(label: &str) -> MenuEntry {
        entry(MenuEntryKind::Section, label)
    }

    /// 0 «Sort by» (title), 1 Size, 2 Name, 3 —, 4 Open, 5 —, 6 —, 7 Delete
    fn menu() -> Vec<MenuEntry> {
        vec![
            section("Sort by"),
            item("Size"),
            item("Name"),
            separator(),
            item("Open"),
            separator(),
            separator(),
            item("Delete"),
        ]
    }

    #[test]
    fn arrows_skip_separators_and_titles() {
        let m = menu();
        assert_eq!(step(&m, 2, 1, true), 4);
        assert_eq!(step(&m, 4, 1, true), 7);
        assert_eq!(step(&m, 4, -1, true), 2);
        assert_eq!(
            step(&m, 1, -1, false),
            1,
            "no wrap: stays on the first item"
        );
    }

    #[test]
    fn arrows_go_round_only_with_wrap() {
        let m = menu();
        assert_eq!(step(&m, 7, 1, true), 1);
        assert_eq!(step(&m, 1, -1, true), 7);
        assert_eq!(step(&m, 7, 1, false), 7);
    }

    #[test]
    fn from_nothing_down_starts_first_and_up_starts_last() {
        let m = menu();
        assert_eq!(step(&m, -1, 1, true), 1);
        assert_eq!(step(&m, -1, -1, true), 7);
    }

    #[test]
    fn a_page_stops_at_the_end_without_going_round() {
        let m = menu();
        assert_eq!(step(&m, 1, 3, false), 4);
        assert_eq!(step(&m, 4, 10, false), 7);
        assert_eq!(step(&m, 4, -10, false), 1);
    }

    #[test]
    fn no_item_gives_none() {
        assert_eq!(step(&[], -1, 1, true), -1);
        assert_eq!(step(&[separator(), section("A")], -1, 1, true), -1);
        assert_eq!(edge(&[], false), -1);
    }

    #[test]
    fn edges_are_the_first_and_last_items() {
        let m = menu();
        assert_eq!(edge(&m, false), 1);
        assert_eq!(edge(&m, true), 7);
    }

    #[test]
    fn typeahead_finds_the_next_label_ignoring_case() {
        let m = vec![item("Copy"), item("Cut"), item("Paste"), item("copy path")];
        assert_eq!(typeahead(&m, -1, "c"), 0);
        assert_eq!(typeahead(&m, 0, "c"), 1, "one letter moves on");
        assert_eq!(typeahead(&m, 0, "cc"), 1, "the same letter again moves on");
        assert_eq!(typeahead(&m, 1, "ccc"), 3);
        assert_eq!(typeahead(&m, 3, "c"), 0, "goes round");
        assert_eq!(typeahead(&m, 0, "co"), 0, "a longer search keeps a match");
        assert_eq!(typeahead(&m, 0, "copy p"), 3);
        assert_eq!(typeahead(&m, 0, "x"), -1);
        assert_eq!(typeahead(&m, 0, ""), -1);
    }

    #[test]
    fn typeahead_skips_titles() {
        let m = vec![section("Sort"), item("Size")];
        assert_eq!(typeahead(&m, -1, "s"), 1);
    }

    #[test]
    fn control_and_special_keys_are_not_typed() {
        assert!(printable("a"));
        assert!(printable("è"));
        assert!(printable(" "));
        assert!(!printable("\u{1b}"), "Esc");
        assert!(!printable("\u{F700}"), "an arrow");
        assert!(!printable("ab"));
        assert!(!printable(""));
    }

    #[test]
    fn leading_trailing_and_repeated_separators_are_hidden() {
        let m = vec![
            separator(),
            item("A"),
            separator(),
            separator(),
            item("B"),
            separator(),
        ];
        let drawn: Vec<bool> = (0..6).map(|i| shown(&m, i)).collect();
        assert_eq!(drawn, [false, true, true, false, true, false]);
    }

    fn choice(kind: MenuEntryKind, id: &str, checked: bool) -> MenuEntry {
        MenuEntry {
            kind,
            id: id.into(),
            label: id.into(),
            checked,
            ..MenuEntry::default()
        }
    }

    fn checked(entries: &[MenuEntry]) -> Vec<bool> {
        entries.iter().map(|entry| entry.checked).collect()
    }

    #[test]
    fn a_check_box_flips_and_nothing_else_changes() {
        let m = vec![
            choice(MenuEntryKind::Checkbox, "a", false),
            choice(MenuEntryKind::Checkbox, "b", true),
        ];
        assert_eq!(checked(&toggled(&m, "a")), [true, true]);
        assert_eq!(checked(&toggled(&m, "b")), [false, false]);
    }

    #[test]
    fn a_radio_item_clears_only_its_own_group() {
        // Two groups split by a separator: «size» / «name», then «asc» / «desc».
        let m = vec![
            choice(MenuEntryKind::Radio, "size", true),
            choice(MenuEntryKind::Radio, "name", false),
            separator(),
            choice(MenuEntryKind::Radio, "asc", true),
            choice(MenuEntryKind::Radio, "desc", false),
        ];
        assert_eq!(
            checked(&toggled(&m, "name")),
            [false, true, false, true, false]
        );
        assert_eq!(
            checked(&toggled(&m, "size")),
            [true, false, false, true, false],
            "choosing the checked radio keeps it checked"
        );
    }

    #[test]
    fn disabled_plain_and_unknown_entries_change_nothing() {
        let mut locked = choice(MenuEntryKind::Checkbox, "locked", true);
        locked.disabled = true;
        let m = vec![locked, choice(MenuEntryKind::Item, "open", false)];
        assert_eq!(checked(&toggled(&m, "locked")), [true, false]);
        assert_eq!(checked(&toggled(&m, "open")), [true, false]);
        assert_eq!(checked(&toggled(&m, "nope")), [true, false]);
    }
}
