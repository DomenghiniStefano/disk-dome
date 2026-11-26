//! Drives the menus and tooltips spike off screen (card 4.T8, `docs/design-system.md` § 12 test 8): Slint's
//! context menu opens on a right click and its keyboard reaches a submenu; both tooltips appear after the rest
//! delay. Nothing here judges the look: that is the owner's call on the shots.
//!
//! `DISKDOME_SPIKE_SHOTS=<folder>` saves every state as BMP, in dark and light.

#![cfg(feature = "spikes")]

use std::path::PathBuf;
use std::time::Duration;

use diskdome_ui::generated::{MenusSpike, MenusSpikeLayout, Theme, ThemeMode};
use diskdome_ui::headless::Headless;
use slint::ComponentHandle;
use slint::platform::Key;

const WIDTH: u32 = 900;
const HEIGHT: u32 = 560;
/// Past the slowest hover animation (`Motion.slow`, 200 ms) and before Slint's tooltip delay (500 ms, fixed in 1.19).
const HOVER_SETTLED: Duration = Duration::from_millis(300);
/// Past the tooltip delay, with a margin for a busy CI machine.
const TOOLTIP_SHOWN: Duration = Duration::from_millis(500);
/// Down arrows from a closed highlight to «Sort by»: Open, Copy path, then the separator is skipped.
const DOWN_TO_SUBMENU: usize = 3;
/// Inside a target, away from its edges.
const INSET: f32 = 12.0;
/// A point over the background, where nothing reacts.
const NOWHERE: (f32, f32) = (880.0, 540.0);

fn save_shot(headless: &Headless, name: &str) {
    let Some(folder) = std::env::var_os("DISKDOME_SPIKE_SHOTS") else {
        return;
    };
    let path = PathBuf::from(folder).join(format!("menus-{name}.bmp"));
    std::fs::write(&path, headless.bmp()).expect("the shots folder is writable");
}

/// Draws a fresh frame and returns it.
fn shot(headless: &mut Headless) -> Vec<u8> {
    headless.request_redraw();
    headless.frame();
    headless.bmp()
}

/// Rests the pointer on a point: the frame once the hover has settled, and the frame after the tooltip delay.
fn rest_on(headless: &mut Headless, x: f32, y: f32) -> (Vec<u8>, Vec<u8>) {
    headless.hover(x, y);
    headless.frame();
    std::thread::sleep(HOVER_SETTLED);
    let settled = shot(headless);
    std::thread::sleep(TOOLTIP_SHOWN);
    (settled, shot(headless))
}

/// One test: the headless platform is installed once per thread.
#[test]
fn slint_menu_and_tooltips_open_off_screen() {
    let mut headless = Headless::install(WIDTH, HEIGHT).expect("headless platform");
    let spike = MenusSpike::new().expect("the spike compiles");
    spike.show().expect("the spike shows off screen");
    let layout = spike.global::<MenusSpikeLayout>();
    let menu = (
        layout.get_menu_x() + layout.get_menu_w() / 2.0,
        layout.get_menu_y() + layout.get_menu_h() / 2.0,
    );
    let text_tip = (layout.get_text_tip_x() + INSET, layout.get_tip_y() + INSET);
    let custom_tip = (
        layout.get_custom_tip_x() + INSET,
        layout.get_tip_y() + INSET,
    );

    for (mode, name) in [(ThemeMode::Dark, "dark"), (ThemeMode::Light, "light")] {
        spike.global::<Theme>().set_mode(mode);
        headless.hover(NOWHERE.0, NOWHERE.1);
        // The controls animate into the new theme's colours: wait for the end.
        headless.frame();
        std::thread::sleep(HOVER_SETTLED);
        let plain = shot(&mut headless);
        save_shot(&headless, &format!("{name}-plain"));

        headless.right_click(menu.0, menu.1);
        let opened = shot(&mut headless);
        assert!(plain != opened, "a right click opens Slint's context menu");
        save_shot(&headless, &format!("{name}-menu"));

        for _ in 0..DOWN_TO_SUBMENU {
            headless.press_key(Key::DownArrow);
        }
        headless.press_key(Key::RightArrow);
        let submenu = shot(&mut headless);
        assert!(
            opened != submenu,
            "the arrows highlight and open the submenu"
        );
        save_shot(&headless, &format!("{name}-submenu"));

        // A click elsewhere closes the menu and its submenu.
        headless.click(NOWHERE.0, NOWHERE.1);
        let closed = shot(&mut headless);
        save_shot(&headless, &format!("{name}-closed"));
        assert!(plain == closed, "a click outside closes the menu");

        let (settled, text) = rest_on(&mut headless, text_tip.0, text_tip.1);
        save_shot(&headless, &format!("{name}-tooltip-text"));
        assert!(
            settled != text,
            "Slint's text tooltip shows after the delay"
        );
        headless.hover(NOWHERE.0, NOWHERE.1);

        let (settled, ours) = rest_on(&mut headless, custom_tip.0, custom_tip.1);
        save_shot(&headless, &format!("{name}-tooltip-ours"));
        assert!(
            settled != ours,
            "the tooltip with our content shows after the delay"
        );
        headless.hover(NOWHERE.0, NOWHERE.1);
    }
}
