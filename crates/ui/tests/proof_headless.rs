//! Drives both proof windows off screen (Slint's software renderer, no window opened): keyboard navigation and
//! sorting by a click on a column header.

#![cfg(feature = "spikes")]

use std::sync::Arc;

use diskdome_ui::headless::Headless;
use diskdome_ui::proof::data::FakeTree;
use diskdome_ui::proof::model::ProofRows;
use diskdome_ui::proof::{ProofWindow, View};
use slint::platform::Key;

const ROWS: u32 = 5_000;

fn sizes_in_display_order(rows: &ProofRows) -> Vec<u64> {
    (0..rows.len())
        .map(|position| {
            rows.tree()
                .size(rows.entry_at(position).expect("position in range"))
        })
        .collect()
}

/// Both views in one test: the headless platform is installed once per thread.
#[test]
fn keyboard_and_header_sort_work_off_screen() {
    let mut headless = Headless::install(960, 600).unwrap();
    let (width, _) = headless.size();
    // The Size header: the second column, after Name (stretch 4 of 8).
    let size_header_x = width as f32 * 4.0 / 8.0 + 20.0;

    for view in [View::Table, View::List] {
        let tree = Arc::new(FakeTree::generate(ROWS));
        let rows = ProofRows::new(tree);
        let window = ProofWindow::new(view, rows.clone(), false).unwrap();
        window.show().unwrap();
        headless.frame();

        window.focus_table();
        for _ in 0..3 {
            headless.press_key(Key::DownArrow);
        }
        headless.frame();
        assert_eq!(
            window.current_row(),
            2,
            "{view:?}: three Down presses from no selection"
        );

        if view == View::List {
            headless.press_key(Key::End);
            headless.frame();
            assert_eq!(
                window.current_row(),
                ROWS as i32 - 1,
                "List: End selects the last row"
            );
            let max_scroll = window.content_height() - window.visible_height();
            assert!(
                (window.scroll() - max_scroll).abs() < 1.0,
                "List: End scrolls to the bottom"
            );
        }

        let header_y = if view == View::Table { 20.0 } else { 6.0 };
        headless.click(size_header_x, header_y);
        headless.frame();
        let sizes = sizes_in_display_order(&rows);
        assert!(
            sizes.windows(2).all(|pair| pair[0] <= pair[1]),
            "{view:?}: first click sorts by size, ascending"
        );

        headless.click(size_header_x, header_y);
        headless.frame();
        let sizes = sizes_in_display_order(&rows);
        assert!(
            sizes.windows(2).all(|pair| pair[0] >= pair[1]),
            "{view:?}: second click sorts descending"
        );

        window.window().hide().unwrap();
    }
}
