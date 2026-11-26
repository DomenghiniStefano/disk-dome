// No console window behind the app in release builds on Windows.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use diskdome_ui::generated::MainWindow;
use slint::ComponentHandle;

fn main() -> Result<(), slint::PlatformError> {
    // Technical proof of step 3: `diskdome --proof-rows 14000000 [--proof-view table|list]`, `spikes` feature.
    #[cfg(feature = "spikes")]
    if let Some(options) = diskdome_ui::proof::ProofOptions::from_args(std::env::args()) {
        return diskdome_ui::proof::run(options);
    }
    // Title bar spike of card 4.T1: `diskdome --spike-titlebar`, `spikes` feature.
    #[cfg(feature = "spikes")]
    if std::env::args().any(|a| a == diskdome_ui::spike::FLAG) {
        return diskdome_ui::spike::run();
    }
    // The design system's Kit window (card 4.2, § 9): `diskdome --kit`, with the `kit` feature only.
    #[cfg(feature = "kit")]
    if std::env::args().any(|a| a == diskdome_ui::kit::FLAG) {
        return diskdome_ui::kit::run();
    }
    let window = MainWindow::new()?;
    diskdome_ui::design_system::install(&window);
    window.run()
}
