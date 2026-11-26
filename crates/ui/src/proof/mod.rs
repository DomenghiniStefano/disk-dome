//! Technical proof of step 3 (Trello card 3c): a Slint table over millions of fake rows.
//!
//! `diskdome --proof-rows 14000000 [--proof-view table|list]` opens it in a window;
//! `cargo run -p diskdome-ui --release --features spikes --example proof_bench` measures it headless.

pub mod data;
pub mod format;
pub mod model;

use std::rc::Rc;
use std::sync::Arc;
use std::time::Instant;

use slint::{ComponentHandle, Model, ModelRc};

use crate::generated::{ProofListWindow, ProofTableWindow};
use data::{FakeTree, SortColumn};
use model::{ListModel, ProofRows, TableModel};

/// Which widget shows the rows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum View {
    /// Slint's `StandardTableView`.
    Table,
    /// A plain `ListView` with one `Text` per cell.
    List,
}

impl View {
    pub fn parse(text: &str) -> Option<Self> {
        match text {
            "table" => Some(Self::Table),
            "list" => Some(Self::List),
            _ => None,
        }
    }
}

/// What the command line asked for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProofOptions {
    pub rows: u32,
    pub view: View,
}

impl ProofOptions {
    /// Reads `--proof-rows N` and `--proof-view table|list`. `None` when `--proof-rows` is absent or invalid.
    pub fn from_args(args: impl IntoIterator<Item = String>) -> Option<Self> {
        let mut rows = None;
        let mut view = View::Table;
        let mut args = args.into_iter();
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--proof-rows" => rows = args.next().and_then(|value| value.parse().ok()),
                "--proof-view" => {
                    view = args.next().as_deref().and_then(View::parse).unwrap_or(view)
                }
                _ => {}
            }
        }
        rows.map(|rows| Self { rows, view })
    }
}

/// Either window of the proof, behind the few calls the proof needs.
pub enum ProofWindow {
    Table(ProofTableWindow),
    List(ProofListWindow),
}

impl ProofWindow {
    /// Builds the window over `rows` and wires the sort. The sort runs on the calling thread when
    /// `background_sort` is false (measurements), on a worker thread otherwise (the real window).
    pub fn new(
        view: View,
        rows: Rc<ProofRows>,
        background_sort: bool,
    ) -> Result<Self, slint::PlatformError> {
        let count = i32::try_from(rows.len()).unwrap_or(i32::MAX);
        match view {
            View::Table => {
                let window = ProofTableWindow::new()?;
                window.set_rows(ModelRc::new(TableModel(rows)));
                window.set_row_count(count);
                let weak = window.as_weak();
                window.on_sort(move |column, descending| {
                    if let Some(window) = weak.upgrade() {
                        start_sort(
                            &ProofWindow::Table(window),
                            column,
                            descending,
                            background_sort,
                        );
                    }
                });
                Ok(Self::Table(window))
            }
            View::List => {
                let window = ProofListWindow::new()?;
                window.set_rows(ModelRc::new(ListModel(rows)));
                window.set_row_count(count);
                let weak = window.as_weak();
                window.on_sort(move |column, descending| {
                    if let Some(window) = weak.upgrade() {
                        start_sort(
                            &ProofWindow::List(window),
                            column,
                            descending,
                            background_sort,
                        );
                    }
                });
                Ok(Self::List(window))
            }
        }
    }

    pub fn window(&self) -> &slint::Window {
        match self {
            Self::Table(window) => window.window(),
            Self::List(window) => window.window(),
        }
    }

    pub fn show(&self) -> Result<(), slint::PlatformError> {
        match self {
            Self::Table(window) => window.show(),
            Self::List(window) => window.show(),
        }
    }

    pub fn run(&self) -> Result<(), slint::PlatformError> {
        match self {
            Self::Table(window) => window.run(),
            Self::List(window) => window.run(),
        }
    }

    /// The shared rows behind the window's model.
    pub fn rows(&self) -> Option<Rc<ProofRows>> {
        match self {
            Self::Table(window) => {
                let model = window.get_rows();
                model
                    .as_any()
                    .downcast_ref::<TableModel>()
                    .map(|model| model.0.clone())
            }
            Self::List(window) => {
                let model = window.get_rows();
                model
                    .as_any()
                    .downcast_ref::<ListModel>()
                    .map(|model| model.0.clone())
            }
        }
    }

    /// Scroll offset from the top, in logical pixels (positive downwards).
    pub fn scroll(&self) -> f32 {
        match self {
            Self::Table(window) => -window.get_scroll_y(),
            Self::List(window) => -window.get_scroll_y(),
        }
    }

    pub fn set_scroll(&self, offset: f32) {
        match self {
            Self::Table(window) => window.set_scroll_y(-offset),
            Self::List(window) => window.set_scroll_y(-offset),
        }
    }

    pub fn content_height(&self) -> f32 {
        match self {
            Self::Table(window) => window.get_content_height(),
            Self::List(window) => window.get_content_height(),
        }
    }

    pub fn visible_height(&self) -> f32 {
        match self {
            Self::Table(window) => window.get_visible_height(),
            Self::List(window) => window.get_visible_height(),
        }
    }

    pub fn current_row(&self) -> i32 {
        match self {
            Self::Table(window) => window.get_current_row(),
            Self::List(window) => window.get_current_row(),
        }
    }

    pub fn focus_table(&self) {
        match self {
            Self::Table(window) => window.invoke_focus_table(),
            Self::List(window) => window.invoke_focus_table(),
        }
    }

    /// Asks for a sort as a click on a column header would.
    pub fn request_sort(&self, column: i32, descending: bool) {
        match self {
            Self::Table(window) => window.invoke_sort(column, descending),
            Self::List(window) => window.invoke_sort(column, descending),
        }
    }

    fn set_sorting(&self, sorting: bool) {
        match self {
            Self::Table(window) => window.set_sorting(sorting),
            Self::List(window) => window.set_sorting(sorting),
        }
    }

    fn is_sorting(&self) -> bool {
        match self {
            Self::Table(window) => window.get_sorting(),
            Self::List(window) => window.get_sorting(),
        }
    }

    fn set_last_sort_ms(&self, milliseconds: i32) {
        match self {
            Self::Table(window) => window.set_last_sort_ms(milliseconds),
            Self::List(window) => window.set_last_sort_ms(milliseconds),
        }
    }

    fn weak(&self) -> WeakProofWindow {
        match self {
            Self::Table(window) => WeakProofWindow::Table(window.as_weak()),
            Self::List(window) => WeakProofWindow::List(window.as_weak()),
        }
    }
}

/// A handle that can cross to the worker thread and come back to the event loop.
enum WeakProofWindow {
    Table(slint::Weak<ProofTableWindow>),
    List(slint::Weak<ProofListWindow>),
}

impl WeakProofWindow {
    fn upgrade(&self) -> Option<ProofWindow> {
        match self {
            Self::Table(weak) => weak.upgrade().map(ProofWindow::Table),
            Self::List(weak) => weak.upgrade().map(ProofWindow::List),
        }
    }
}

fn start_sort(window: &ProofWindow, column: i32, descending: bool, background: bool) {
    let (Some(column), Some(rows)) = (SortColumn::from_index(column), window.rows()) else {
        return;
    };
    if window.is_sorting() {
        return;
    }
    let tree = rows.tree().clone();
    if !background {
        let start = Instant::now();
        let order = tree.sorted_order(column, descending);
        apply_sort(window, order, start);
        return;
    }
    window.set_sorting(true);
    let weak = window.weak();
    std::thread::spawn(move || {
        let start = Instant::now();
        let order = tree.sorted_order(column, descending);
        let applied = slint::invoke_from_event_loop(move || {
            if let Some(window) = weak.upgrade() {
                apply_sort(&window, order, start);
            }
        });
        if let Err(error) = applied {
            eprintln!("proof: sort not applied, the event loop has ended: {error}");
        }
    });
}

fn apply_sort(window: &ProofWindow, order: Vec<u32>, start: Instant) {
    if let Some(rows) = window.rows() {
        rows.set_order(order);
    }
    window.set_sorting(false);
    window.set_last_sort_ms(i32::try_from(start.elapsed().as_millis()).unwrap_or(i32::MAX));
}

/// Generates the rows and opens the proof window (a real, visible window: for the owner).
pub fn run(options: ProofOptions) -> Result<(), slint::PlatformError> {
    let tree = Arc::new(FakeTree::generate(options.rows));
    let window = ProofWindow::new(options.view, ProofRows::new(tree), true)?;
    window.run()
}

#[cfg(test)]
mod tests {
    use super::{ProofOptions, View};

    fn parse(args: &[&str]) -> Option<ProofOptions> {
        ProofOptions::from_args(args.iter().map(|arg| (*arg).to_owned()))
    }

    #[test]
    fn reads_the_command_line() {
        assert_eq!(parse(&["diskdome"]), None);
        assert_eq!(
            parse(&["diskdome", "--proof-rows", "12"]),
            Some(ProofOptions {
                rows: 12,
                view: View::Table
            })
        );
        assert_eq!(
            parse(&["diskdome", "--proof-view", "list", "--proof-rows", "5"]),
            Some(ProofOptions {
                rows: 5,
                view: View::List
            })
        );
        assert_eq!(parse(&["diskdome", "--proof-rows", "many"]), None);
    }
}
