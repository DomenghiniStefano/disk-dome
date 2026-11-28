//! Wires Dome UI's Rust half (`crates/dome-ui`) to this app's generated Slint code: `install` on every window.

use crate::generated::{
    BreadcrumbFit, BreadcrumbItem, MenuEntry, MenuEntryKind, MenuNav, OptionCardItem,
    OptionCardNav, RadioItem, RadioNav, SegmentedItem, SegmentedNav, SelectNav, TabItem, TabNav,
};

dome_ui::menu_nav!(MenuNav, MenuEntry, MenuEntryKind);
dome_ui::segmented_nav!(SegmentedNav, SegmentedItem);
dome_ui::radio_nav!(RadioNav, RadioItem);
dome_ui::select_nav!(SelectNav, MenuEntry, MenuEntryKind);
dome_ui::tab_nav!(TabNav, TabItem, MenuEntry, MenuEntryKind);
dome_ui::breadcrumb_fit!(BreadcrumbFit, BreadcrumbItem, MenuEntry);
dome_ui::option_card_nav!(OptionCardNav, OptionCardItem);

/// Wires every Dome UI global of a window: call it once per window.
pub fn install<'a, T>(window: &'a T)
where
    T: slint::ComponentHandle,
    MenuNav<'a>: slint::Global<'a, T>,
    SegmentedNav<'a>: slint::Global<'a, T>,
    RadioNav<'a>: slint::Global<'a, T>,
    SelectNav<'a>: slint::Global<'a, T>,
    TabNav<'a>: slint::Global<'a, T>,
    BreadcrumbFit<'a>: slint::Global<'a, T>,
    OptionCardNav<'a>: slint::Global<'a, T>,
{
    install_menu_nav(window);
    install_segmented_nav(window);
    install_radio_nav(window);
    install_select_nav(window);
    install_tab_nav(window);
    install_breadcrumb_fit(window);
    install_option_card_nav(window);
}
