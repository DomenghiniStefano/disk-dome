//! Lazy Slint models over the fake entries: Slint asks for a row, the row is formatted at that moment.
//! Nothing is kept per row besides the sort permutation (`u32` per row).

use std::any::Any;
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::Arc;

use slint::{
    Model, ModelNotify, ModelRc, ModelTracker, SharedString, StandardListViewItem, VecModel,
};

use super::data::FakeTree;
use super::format;
use crate::generated::ProofRow;

/// Counts what Slint asked of the model, for the measurements.
#[derive(Default)]
pub struct RowStats {
    calls: Cell<u64>,
    lowest: Cell<u32>,
    highest: Cell<u32>,
}

impl RowStats {
    fn record(&self, row: usize) {
        let row = row as u32;
        if self.calls.get() == 0 {
            self.lowest.set(row);
            self.highest.set(row);
        } else {
            self.lowest.set(self.lowest.get().min(row));
            self.highest.set(self.highest.get().max(row));
        }
        self.calls.set(self.calls.get() + 1);
    }

    /// Returns (calls, lowest row, highest row) since the last call, and starts counting again.
    pub fn take(&self) -> (u64, u32, u32) {
        let result = (self.calls.get(), self.lowest.get(), self.highest.get());
        self.calls.set(0);
        result
    }
}

/// The rows in display order: the entries and the current sort permutation.
pub struct ProofRows {
    tree: Arc<FakeTree>,
    /// Display position -> entry index. Empty means "unsorted", i.e. the identity, so no memory is spent
    /// until the first sort.
    order: RefCell<Vec<u32>>,
    notify: ModelNotify,
    stats: Rc<RowStats>,
}

/// The formatted cells of one row.
struct Cells {
    name: SharedString,
    size: SharedString,
    percent: SharedString,
    modified: SharedString,
}

impl ProofRows {
    pub fn new(tree: Arc<FakeTree>) -> Rc<Self> {
        Rc::new(Self {
            tree,
            order: RefCell::new(Vec::new()),
            notify: ModelNotify::default(),
            stats: Rc::new(RowStats::default()),
        })
    }

    pub fn tree(&self) -> &Arc<FakeTree> {
        &self.tree
    }

    pub fn stats(&self) -> Rc<RowStats> {
        self.stats.clone()
    }

    pub fn len(&self) -> usize {
        self.tree.len()
    }

    pub fn is_empty(&self) -> bool {
        self.tree.is_empty()
    }

    /// Replaces the permutation and tells the views that every row changed.
    pub fn set_order(&self, order: Vec<u32>) {
        *self.order.borrow_mut() = order;
        self.notify.reset();
    }

    /// The entry shown at a display position.
    pub fn entry_at(&self, position: usize) -> Option<u32> {
        if position >= self.tree.len() {
            return None;
        }
        let order = self.order.borrow();
        if order.is_empty() {
            Some(position as u32)
        } else {
            order.get(position).copied()
        }
    }

    fn cells(&self, position: usize) -> Option<Cells> {
        self.stats.record(position);
        let entry = self.entry_at(position)?;
        let tree = &self.tree;

        let name = SharedString::from(String::from_utf8_lossy(tree.name(entry)).as_ref());
        let mut text = String::with_capacity(24);
        format::size(tree.size(entry), &mut text);
        let size = SharedString::from(text.as_str());
        text.clear();
        format::percent(tree.size(entry), tree.parent_size(entry), &mut text);
        let percent = SharedString::from(text.as_str());
        text.clear();
        format::date(tree.mtime(entry), &mut text);
        let modified = SharedString::from(text.as_str());
        Some(Cells {
            name,
            size,
            percent,
            modified,
        })
    }
}

/// The rows as `StandardTableView` wants them: one small model of four cells per row.
pub struct TableModel(pub Rc<ProofRows>);

impl Model for TableModel {
    type Data = ModelRc<StandardListViewItem>;

    fn row_count(&self) -> usize {
        self.0.len()
    }

    fn row_data(&self, row: usize) -> Option<Self::Data> {
        let cells = self.0.cells(row)?;
        let items =
            [cells.name, cells.size, cells.percent, cells.modified].map(StandardListViewItem::from);
        Some(ModelRc::new(VecModel::from(Vec::from(items))))
    }

    fn model_tracker(&self) -> &dyn ModelTracker {
        &self.0.notify
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// The rows as the `ListView` variant wants them: one struct per row.
pub struct ListModel(pub Rc<ProofRows>);

impl Model for ListModel {
    type Data = ProofRow;

    fn row_count(&self) -> usize {
        self.0.len()
    }

    fn row_data(&self, row: usize) -> Option<Self::Data> {
        let cells = self.0.cells(row)?;
        Some(ProofRow {
            name: cells.name,
            size: cells.size,
            percent: cells.percent,
            modified: cells.modified,
        })
    }

    fn model_tracker(&self) -> &dyn ModelTracker {
        &self.0.notify
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
