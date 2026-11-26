//! Dome UI, the Dome apps' design system (`docs/design-system.md`). Its Slint half is `ui/`, a Slint library an app
//! imports as `@dome-ui` (`slint-project.json`, wired in the app's `build.rs`); this is its Rust half: what the
//! components ask Rust for, as pure functions, and the macro that wires them to an app's generated globals.

pub mod menu;

/// Writes `pub fn install(window)` in the calling module: it wires a window's `MenuNav` global (`§ 5.16`) to
/// [`menu`]. The app names its own generated types, since each app compiles the Slint half itself:
///
/// ```ignore
/// use crate::generated::{MenuEntry, MenuEntryKind, MenuNav};
/// dome_ui::menu_nav!(MenuNav, MenuEntry, MenuEntryKind);
/// ```
///
/// Globals are per window, so every window that shows a menu calls `install` once.
#[macro_export]
macro_rules! menu_nav {
    ($nav:ident, $entry:ident, $kind:ident) => {
        /// Wires the window's `MenuNav` global to `dome_ui::menu`.
        pub fn install<'a, T>(window: &'a T)
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
