//! Wires Dome UI's Rust half (`crates/dome-ui`) to this app's generated Slint code: `install` on every window.

use crate::generated::{
    MenuEntry, MenuEntryKind, MenuNav, SegmentedItem, SegmentedNav, SelectNav,
};

dome_ui::menu_nav!(MenuNav, MenuEntry, MenuEntryKind);
dome_ui::segmented_nav!(SegmentedNav, SegmentedItem);
dome_ui::select_nav!(SelectNav, MenuEntry, MenuEntryKind);

/// Wires every Dome UI global of a window: call it once per window.
pub fn install<'a, T>(window: &'a T)
where
    T: slint::ComponentHandle,
    MenuNav<'a>: slint::Global<'a, T>,
    SegmentedNav<'a>: slint::Global<'a, T>,
    SelectNav<'a>: slint::Global<'a, T>,
{
    install_menu_nav(window);
    install_segmented_nav(window);
    install_select_nav(window);
}
