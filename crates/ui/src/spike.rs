//! The title bar spike (card 4.T1, `docs/design-system.md` § 12 test 1): `diskdome --spike-titlebar` opens a
//! window with DiskDome's own title bar and the drive tabs, for the owner to try on Windows 10 and 11.
//! Throwaway: removed once the test decides where the tabs go.

use std::rc::Rc;
use std::time::Duration;

use diskdome_platform::titlebar::{self, Event, Regions};
use raw_window_handle::HasWindowHandle;
use slint::{ComponentHandle, ModelRc, SharedString, VecModel};

use crate::generated::{SpikeTab, TitlebarSpike};

/// The flag that opens the spike instead of the app.
pub const FLAG: &str = "--spike-titlebar";

pub fn run() -> Result<(), slint::PlatformError> {
    let window = TitlebarSpike::new()?;
    let tabs = [
        ("Start", ""),
        ("C:", "Windows"),
        ("D:", "Data"),
        ("E:", "USB"),
    ]
    .map(|(letter, label)| SpikeTab {
        letter: SharedString::from(letter),
        label: SharedString::from(label),
    });
    window.set_small_min(std::env::var_os("DISKDOME_SPIKE_SMALL_MIN").is_some());
    window.set_tabs(ModelRc::from(Rc::new(VecModel::from(tabs.to_vec()))));

    let weak = window.as_weak();
    window.on_minimize(move || {
        if let Some(w) = weak.upgrade() {
            w.window().set_minimized(true);
        }
    });
    let weak = window.as_weak();
    window.on_toggle_maximize(move || {
        if let Some(w) = weak.upgrade() {
            let maximized = w.window().is_maximized();
            w.window().set_maximized(!maximized);
            w.set_is_maximized(!maximized);
        }
    });
    window.on_close_window(|| {
        // Ends `run_event_loop`, like closing a native window.
        let _ = slint::quit_event_loop();
    });
    let weak = window.as_weak();
    window.on_regions_changed(move || {
        if let Some(w) = weak.upgrade() {
            titlebar::set_regions(regions(&w));
        }
    });

    window.show()?;
    install(window.as_weak(), 0);
    slint::run_event_loop()
}

/// The bar's regions from the Slint layout, in logical pixels.
fn regions(w: &TitlebarSpike) -> Regions {
    let x = w.get_captions_x();
    let cw = w.get_caption_w();
    Regions {
        height: w.get_bar_height(),
        interactive: vec![
            (0.0, w.get_tabs_end()),
            (x, x + cw),
            (x + 2.0 * cw, x + 3.0 * cw),
        ],
        maximize: (x + cw, x + 2.0 * cw),
    }
}

/// Takes over the title bar once the native window exists (it is created when the event loop starts).
fn install(weak: slint::Weak<TitlebarSpike>, attempt: u32) {
    slint::Timer::single_shot(
        Duration::from_millis(if attempt == 0 { 0 } else { 50 }),
        move || {
            let Some(w) = weak.upgrade() else { return };
            let handle = w.window().window_handle();
            let Ok(raw) = handle.window_handle().map(|h| h.as_raw()) else {
                if attempt < 20 {
                    install(weak, attempt + 1);
                }
                return;
            };
            let events = weak.clone();
            let installed = titlebar::install(
                raw,
                Box::new(move |event| {
                    let events = events.clone();
                    // Windows calls back from inside its window procedure: apply on the next turn of the event loop.
                    let applied = slint::invoke_from_event_loop(move || {
                        if let Some(w) = events.upgrade() {
                            match event {
                                Event::MaximizeHover(on) => w.set_max_hovered(on),
                                Event::MaximizePressed(on) => w.set_max_pressed(on),
                                Event::Maximized(on) => w.set_is_maximized(on),
                            }
                        }
                    });
                    if let Err(error) = applied {
                        eprintln!(
                            "titlebar spike: event dropped, the event loop has ended: {error}"
                        );
                    }
                }),
            );
            if installed {
                titlebar::set_regions(regions(&w));
            }
        },
    );
}
