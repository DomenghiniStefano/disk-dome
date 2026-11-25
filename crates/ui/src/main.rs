// No console window behind the app in release builds on Windows.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use diskdome_ui::generated::MainWindow;
use diskdome_ui::proof::{self, ProofOptions};
use diskdome_ui::spike;
use slint::ComponentHandle;

fn main() -> Result<(), slint::PlatformError> {
    // Technical proof of step 3: `diskdome --proof-rows 14000000 [--proof-view table|list]`.
    if let Some(options) = ProofOptions::from_args(std::env::args()) {
        return proof::run(options);
    }
    // Title bar spike of card 4.T1: `diskdome --spike-titlebar`.
    if std::env::args().any(|a| a == spike::FLAG) {
        return spike::run();
    }
    // The design system's Kit window (card 4.2, § 9): `diskdome --kit`, with the `kit` feature only.
    #[cfg(feature = "kit")]
    if std::env::args().any(|a| a == diskdome_ui::kit::FLAG) {
        return diskdome_ui::kit::run();
    }
    let window = MainWindow::new()?;
    window.run()
}
