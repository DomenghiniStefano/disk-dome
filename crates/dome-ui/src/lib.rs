//! Dome UI, the Dome apps' design system (`docs/design-system.md`). Its Slint half is `ui/`, a Slint library an app
//! imports as `@dome-ui` (`slint-project.json`, wired in the app's `build.rs`); this is its Rust half: what the
//! components ask Rust for, as pure functions, and the macro that wires them to an app's generated globals.

pub mod breadcrumb;
pub mod menu;
pub mod radio;
pub mod roving;
pub mod segmented;
pub mod select;

/// Writes `pub fn install_menu_nav(window)` in the calling module: it wires a window's `MenuNav` global (`§ 5.16`) to
/// [`menu`]. The app names its own generated types, since each app compiles the Slint half itself:
///
/// ```ignore
/// use crate::generated::{MenuEntry, MenuEntryKind, MenuNav};
/// dome_ui::menu_nav!(MenuNav, MenuEntry, MenuEntryKind);
/// ```
///
/// Globals are per window, so every window that shows a menu calls it once.
#[macro_export]
macro_rules! menu_nav {
    ($nav:ident, $entry:ident, $kind:ident) => {
        /// Wires the window's `MenuNav` global to `dome_ui::menu`.
        pub fn install_menu_nav<'a, T>(window: &'a T)
        where
            T: slint::ComponentHandle,
            $nav<'a>: slint::Global<'a, T>,
        {
            let nav = window.global::<$nav>();
            nav.on_step(|entries, from, by, wrap| {
                $crate::menu::step(&dome_ui_entries(&entries), from, by, wrap)
            });
            nav.on_edge(|entries, last| $crate::menu::edge(&dome_ui_entries(&entries), last));
            nav.on_typeahead(|entries, from, typed| {
                $crate::menu::typeahead(&dome_ui_entries(&entries), from, &typed)
            });
            nav.on_printable(|text| $crate::menu::printable(&text));
            nav.on_shown(|entries, index| $crate::menu::shown(&dome_ui_entries(&entries), index));
            nav.on_toggled(|entries, id| {
                let after = $crate::menu::toggled(&dome_ui_entries(&entries), &id);
                let rows: Vec<$entry> = slint::Model::iter(&entries)
                    .zip(after)
                    .map(|(entry, plain)| $entry {
                        checked: plain.checked,
                        ..entry
                    })
                    .collect();
                slint::ModelRc::new(slint::VecModel::from(rows))
            });
        }

        /// The app's entries as the plain ones `dome_ui::menu` reads.
        fn dome_ui_entries(entries: &slint::ModelRc<$entry>) -> Vec<$crate::menu::MenuEntry> {
            slint::Model::iter(entries)
                .map(|entry| $crate::menu::MenuEntry {
                    kind: match entry.kind {
                        $kind::Item => $crate::menu::MenuEntryKind::Item,
                        $kind::Checkbox => $crate::menu::MenuEntryKind::Checkbox,
                        $kind::Radio => $crate::menu::MenuEntryKind::Radio,
                        $kind::Separator => $crate::menu::MenuEntryKind::Separator,
                        $kind::Section => $crate::menu::MenuEntryKind::Section,
                    },
                    id: entry.id.to_string(),
                    label: entry.label.to_string(),
                    disabled: entry.disabled,
                    checked: entry.checked,
                })
                .collect()
        }
    };
}

/// Writes `pub fn install_segmented_nav(window)` in the calling module: it wires a window's `SegmentedNav` global (§ 5.6)
/// to [`segmented`], as [`menu_nav!`] does for menus:
///
/// ```ignore
/// use crate::generated::{SegmentedNav, SegmentedItem};
/// dome_ui::segmented_nav!(SegmentedNav, SegmentedItem);
/// ```
#[macro_export]
macro_rules! segmented_nav {
    ($nav:ident, $item:ident) => {
        $crate::roving_nav!(install_segmented_nav, $nav, $item);
    };
}

/// Writes `pub fn install_tab_nav(window)` in the calling module: it wires a window's `TabNav` global (§ 5.7) to
/// [`roving`], and builds the overflow menu's entries (one radio item per tab, the current one checked, its index as
/// the entry's `id`) with the app's own `MenuEntry`:
///
/// ```ignore
/// use crate::generated::{MenuEntry, MenuEntryKind, TabItem, TabNav};
/// dome_ui::tab_nav!(TabNav, TabItem, MenuEntry, MenuEntryKind);
/// ```
#[macro_export]
macro_rules! tab_nav {
    ($nav:ident, $item:ident, $entry:ident, $kind:ident) => {
        $crate::roving_nav!(install_tab_nav_walk, $nav, $item);

        /// Wires the window's `TabNav` global: the walk over the tabs and the overflow menu.
        pub fn install_tab_nav<'a, T>(window: &'a T)
        where
            T: slint::ComponentHandle,
            $nav<'a>: slint::Global<'a, T>,
        {
            install_tab_nav_walk(window);
            window.global::<$nav>().on_entries(|items, current| {
                let entries: Vec<$entry> = slint::Model::iter(&items)
                    .enumerate()
                    .map(|(index, item)| $entry {
                        id: index.to_string().into(),
                        kind: $kind::Radio,
                        label: item.text,
                        disabled: item.disabled,
                        // A tab's tooltip says why it is disabled; an enabled one's adds nothing to the menu.
                        reason: if item.disabled {
                            item.tooltip
                        } else {
                            slint::SharedString::default()
                        },
                        checked: i32::try_from(index).is_ok_and(|index| index == current),
                        ..Default::default()
                    })
                    .collect();
                slint::ModelRc::new(slint::VecModel::from(entries))
            });
        }
    };
}

/// Writes `pub fn $install(window)`: it wires the `step` and `edge` callbacks of a row with one Tab stop (a
/// `SegmentedNav`, a `TabNav`) to [`roving`]. Shared by [`segmented_nav!`] and [`tab_nav!`]; not for apps.
#[doc(hidden)]
#[macro_export]
macro_rules! roving_nav {
    ($install:ident, $nav:ident, $item:ident) => {
        /// Wires the window's walk over its items to `dome_ui::roving`.
        pub fn $install<'a, T>(window: &'a T)
        where
            T: slint::ComponentHandle,
            $nav<'a>: slint::Global<'a, T>,
        {
            /// Which items are disabled, as `dome_ui::roving` reads them.
            fn disabled(items: &slint::ModelRc<$item>) -> Vec<bool> {
                slint::Model::iter(items)
                    .map(|item| item.disabled)
                    .collect()
            }

            let nav = window.global::<$nav>();
            nav.on_step(|items, from, forward, wrap| {
                $crate::roving::step(&disabled(&items), from, forward, wrap)
            });
            nav.on_edge(|items, last| $crate::roving::edge(&disabled(&items), last));
        }
    };
}

/// Writes `pub fn install_radio_nav(window)` in the calling module: it wires a window's `RadioNav` global (§ 5.8) to
/// [`roving`], the walk the segmented control and the tabs share:
///
/// ```ignore
/// use crate::generated::{RadioNav, RadioItem};
/// dome_ui::radio_nav!(RadioNav, RadioItem);
/// ```
#[macro_export]
macro_rules! radio_nav {
    ($nav:ident, $item:ident) => {
        $crate::roving_nav!(install_radio_nav, $nav, $item);
    };
}

/// Writes `pub fn install_select_nav(window)` in the calling module: it wires a window's `SelectNav` global (§ 5.5) to
/// [`select`]. It reads the entries through the helper [`menu_nav!`] writes, so it goes in the same module, after it:
///
/// ```ignore
/// use crate::generated::{MenuEntry, MenuEntryKind, MenuNav, SelectNav};
/// dome_ui::menu_nav!(MenuNav, MenuEntry, MenuEntryKind);
/// dome_ui::select_nav!(SelectNav, MenuEntry, MenuEntryKind);
/// ```
#[macro_export]
macro_rules! select_nav {
    ($nav:ident, $entry:ident, $kind:ident) => {
        /// Wires the window's `SelectNav` global to `dome_ui::select`.
        pub fn install_select_nav<'a, T>(window: &'a T)
        where
            T: slint::ComponentHandle,
            $nav<'a>: slint::Global<'a, T>,
        {
            let nav = window.global::<$nav>();
            nav.on_marked(|entries, chosen| {
                let after = $crate::select::marked(&dome_ui_entries(&entries), chosen);
                let rows: Vec<$entry> = slint::Model::iter(&entries)
                    .zip(after)
                    .map(|(entry, plain)| $entry {
                        kind: match plain.kind {
                            $crate::menu::MenuEntryKind::Item => $kind::Item,
                            $crate::menu::MenuEntryKind::Checkbox => $kind::Checkbox,
                            $crate::menu::MenuEntryKind::Radio => $kind::Radio,
                            $crate::menu::MenuEntryKind::Separator => $kind::Separator,
                            $crate::menu::MenuEntryKind::Section => $kind::Section,
                        },
                        checked: plain.checked,
                        ..entry
                    })
                    .collect();
                slint::ModelRc::new(slint::VecModel::from(rows))
            });
            nav.on_typeahead(|entries, from, typed| {
                $crate::select::typeahead(&dome_ui_entries(&entries), from, &typed)
            });
        }
    };
}

/// Writes `pub fn install_breadcrumb_fit(window)` in the calling module: it wires a window's `BreadcrumbFit` global
/// (§ 5.13) to [`breadcrumb`], as [`menu_nav!`] does for menus. The folded segments become the entries of the «…»
/// menu, so the app names its `MenuEntry` too:
///
/// ```ignore
/// use crate::generated::{BreadcrumbFit, BreadcrumbItem, MenuEntry};
/// dome_ui::breadcrumb_fit!(BreadcrumbFit, BreadcrumbItem, MenuEntry);
/// ```
#[macro_export]
macro_rules! breadcrumb_fit {
    ($fit:ident, $item:ident, $entry:ident) => {
        /// Wires the window's `BreadcrumbFit` global to `dome_ui::breadcrumb`.
        pub fn install_breadcrumb_fit<'a, T>(window: &'a T)
        where
            T: slint::ComponentHandle,
            $fit<'a>: slint::Global<'a, T>,
        {
            let fit = window.global::<$fit>();
            fit.on_with_width(|widths, index, width| {
                let widths = $crate::breadcrumb::with_width(&dome_ui_widths(&widths), index, width);
                slint::ModelRc::new(slint::VecModel::from(widths))
            });
            fit.on_total(|widths, count| {
                $crate::breadcrumb::total(&dome_ui_widths(&widths), dome_ui_count(count))
            });
            fit.on_first_shown(|widths, count, available, ellipsis, keep_first| {
                $crate::breadcrumb::first_shown(
                    &dome_ui_widths(&widths),
                    dome_ui_count(count),
                    available,
                    ellipsis,
                    keep_first,
                )
            });
            fit.on_hidden(|items, first_shown, keep_first| {
                let count = slint::Model::row_count(&items);
                let range = $crate::breadcrumb::hidden(first_shown, count, keep_first);
                let entries: Vec<$entry> = slint::Model::iter(&items)
                    .enumerate()
                    .filter(|(index, _)| range.contains(index))
                    .map(|(index, item): (usize, $item)| $entry {
                        id: index.to_string().into(),
                        label: item.text,
                        icon: item.icon,
                        ..Default::default()
                    })
                    .collect();
                slint::ModelRc::new(slint::VecModel::from(entries))
            });
        }

        /// The widths Slint measured, as plain numbers.
        fn dome_ui_widths(widths: &slint::ModelRc<f32>) -> Vec<f32> {
            slint::Model::iter(widths).collect()
        }

        /// A count from Slint, never negative.
        fn dome_ui_count(count: i32) -> usize {
            usize::try_from(count).unwrap_or(0)
        }
    };
}
