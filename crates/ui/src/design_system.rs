//! Wires Dome UI's Rust half (`crates/dome-ui`) to this app's generated Slint code: `install` on every window.

use crate::generated::{
    BreadcrumbFit, BreadcrumbItem, MenuEntry, MenuEntryKind, MenuNav, SegmentedItem, SegmentedNav,
};

dome_ui::menu_nav!(MenuNav, MenuEntry, MenuEntryKind);
dome_ui::segmented_nav!(SegmentedNav, SegmentedItem);
dome_ui::breadcrumb_fit!(BreadcrumbFit, BreadcrumbItem, MenuEntry);

/// Wires every Dome UI global of a window: call it once per window.
pub fn install<'a, T>(window: &'a T)
where
    T: slint::ComponentHandle,
    MenuNav<'a>: slint::Global<'a, T>,
    SegmentedNav<'a>: slint::Global<'a, T>,
    BreadcrumbFit<'a>: slint::Global<'a, T>,
{
    install_menu_nav(window);
    install_segmented_nav(window);
    install_breadcrumb_fit(window);
}
