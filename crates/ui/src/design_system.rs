//! Wires Dome UI's Rust half (`crates/dome-ui`) to this app's generated Slint code: `install` on every window.

use crate::generated::{MenuEntry, MenuEntryKind, MenuNav};

dome_ui::menu_nav!(MenuNav, MenuEntry, MenuEntryKind);
