//! The Icon off screen (`docs/design-system.md` § 3.12), on its test window `KitIconHarness`: the token sizes in
//! both densities, a side of its own that density does not change, and `InkTone.data` drawing the category's colour.

#![cfg(feature = "kit")]

use diskdome_ui::design_system;
use diskdome_ui::generated::{KitIconHarness, Size};
use diskdome_ui::headless::Headless;
use slint::ComponentHandle;

const WIDTH: u32 = 960;
const HEIGHT: u32 = 600;

/// `icon` (normal / compact), `icon-sm`, `icon-hero` (§ 3.12, § 3.5).
const NORMAL: f32 = 16.0;
const NORMAL_COMPACT: f32 = 14.0;
const SMALL: f32 = 12.0;
const HERO: f32 = 18.0;
/// The free side, `space-48`.
const FREE: f32 = 48.0;
/// Lucide's `square` is an 18 unit rectangle at 3, stroked 2 wide on a 24 grid: at 48 px its stroke runs from 4 to 8 px
/// from the edge, so 6 px in is the middle of the stroke, fully covered.
const STROKE_MIDDLE: f32 = 6.0;

/// Two turns of the loop: a change is drawn in the next one.
fn settle(headless: &mut Headless) {
    headless.frame();
    headless.frame();
}

/// One test: the headless platform is installed once per thread.
#[test]
fn icon_sizes_side_and_data_tone() {
    let mut headless = Headless::install(WIDTH, HEIGHT).expect("headless platform");
    let harness = KitIconHarness::new().expect("the harness compiles");
    design_system::install(&harness);
    harness.show().expect("the harness shows off screen");
    settle(&mut headless);

    assert_eq!(harness.get_normal_side(), NORMAL);
    assert_eq!(harness.get_small_side(), SMALL);
    assert_eq!(harness.get_hero_side(), HERO);
    assert_eq!(harness.get_free_side(), FREE);

    // `InkTone.data` draws the category's own colour.
    let x = (harness.get_free_x() + STROKE_MIDDLE) as usize;
    let y = (harness.get_free_y() + FREE / 2.0) as usize;
    let [r, g, b, a] = headless.pixel(x, y);
    let video = harness.get_video();
    let near = |drawn: u8, wanted: u8| drawn.abs_diff(wanted) <= 1;
    assert!(
        a == 255 && near(r, video.red()) && near(g, video.green()) && near(b, video.blue()),
        "the stroke is the category's colour: drawn {:?}, wanted {video:?}",
        [r, g, b, a]
    );

    // Compact density narrows the token size only.
    harness.global::<Size>().set_compact(true);
    settle(&mut headless);
    assert_eq!(harness.get_normal_side(), NORMAL_COMPACT);
    assert_eq!(harness.get_free_side(), FREE, "a side of its own stays");
}
