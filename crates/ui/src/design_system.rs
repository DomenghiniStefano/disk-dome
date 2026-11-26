//! Wires Dome UI's Rust half (`crates/dome-ui`) to this app's generated Slint code: `install` on every window.

use crate::generated::{MenuEntry, MenuEntryKind, MenuNav, SegmentNav, SegmentOption};

dome_ui::menu_nav!(MenuNav, MenuEntry, MenuEntryKind);
dome_ui::segment_nav!(SegmentNav, SegmentOption);

/// Wires every Dome UI global of a window: call it once per window.
pub fn install<'a, T>(window: &'a T)
where
    T: slint::ComponentHandle,
    MenuNav<'a>: slint::Global<'a, T>,
    SegmentNav<'a>: slint::Global<'a, T>,
{
    install_menu_nav(window);
    install_segment_nav(window);
}
