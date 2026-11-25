//! The Kit window (`docs/design-system.md` § 9): `cargo run -p diskdome-ui --features kit -- --kit` shows every
//! component of the design system in every variant, size and state. Only with the `kit` feature, never in a
//! release.

use slint::ComponentHandle;

use crate::generated::KitWindow;

/// The flag that opens the Kit instead of the app.
pub const FLAG: &str = "--kit";

pub fn run() -> Result<(), slint::PlatformError> {
    KitWindow::new()?.run()
}
