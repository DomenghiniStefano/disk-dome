//! Dome UI, the Dome apps' design system (`docs/design-system.md`). Its Slint half is `ui/`, a Slint library an app
//! imports as `@dome-ui` (`slint-project.json`, wired in the app's `build.rs`); this is its Rust half: what the
//! components ask Rust for, as pure functions, and the macro that wires them to an app's generated globals.

pub mod menu;
pub mod radio;
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
        /// Wires the window's `SegmentedNav` global to `dome_ui::segmented`.
        pub fn install_segmented_nav<'a, T>(window: &'a T)
        where
            T: slint::ComponentHandle,
            $nav<'a>: slint::Global<'a, T>,
        {
            let nav = window.global::<$nav>();
            nav.on_step(|items, from, forward, wrap| {
                $crate::segmented::step(&dome_ui_disabled(&items), from, forward, wrap)
            });
            nav.on_edge(|items, last| $crate::segmented::edge(&dome_ui_disabled(&items), last));
        }

        /// Which segments are disabled, as `dome_ui::segmented` reads them.
        fn dome_ui_disabled(items: &slint::ModelRc<$item>) -> Vec<bool> {
            slint::Model::iter(items)
                .map(|item| item.disabled)
                .collect()
        }
    };
}

/// Writes `pub fn install_radio_nav(window)` in the calling module: it wires a window's `RadioNav` global (§ 5.8) to
/// [`radio`], as [`segmented_nav!`] does for segmented controls:
///
/// ```ignore
/// use crate::generated::{RadioNav, RadioItem};
/// dome_ui::radio_nav!(RadioNav, RadioItem);
/// ```
#[macro_export]
macro_rules! radio_nav {
    ($nav:ident, $item:ident) => {
        /// Wires the window's `RadioNav` global to `dome_ui::radio`.
        pub fn install_radio_nav<'a, T>(window: &'a T)
        where
            T: slint::ComponentHandle,
            $nav<'a>: slint::Global<'a, T>,
        {
            let nav = window.global::<$nav>();
            nav.on_step(|items, from, forward, wrap| {
                $crate::radio::step(&dome_ui_radio_disabled(&items), from, forward, wrap)
            });
            nav.on_edge(|items, last| $crate::radio::edge(&dome_ui_radio_disabled(&items), last));
        }

        /// Which radio items are disabled, as `dome_ui::radio` reads them.
        fn dome_ui_radio_disabled(items: &slint::ModelRc<$item>) -> Vec<bool> {
            slint::Model::iter(items)
                .map(|item| item.disabled)
                .collect()
        }
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
