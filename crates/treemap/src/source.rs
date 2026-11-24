//! What the map is drawn from: a tree of sizes with a category per entry.
//!
//! [`MapSource`] is the only thing the layout and the name placement read, so the benchmark and
//! the tests use a [`SimpleTree`] while the app reads the scanned [`diskdome_core::Tree`] through
//! a [`CoreSource`], without copying it.

use diskdome_core::Tree;

use crate::CATEGORIES;

/// A tree the map can be drawn from.
pub trait MapSource {
    /// Size of an entry; a folder's is the total of everything below it.
    fn size(&self, i: u32) -> u64;
    fn is_dir(&self, i: u32) -> bool;
    fn children(&self, i: u32) -> &[u32];
    /// A file's category, or a folder's largest category (by bytes below it), below
    /// [`CATEGORIES`]: the index into the [`crate::Palette`].
    fn category(&self, i: u32) -> u8;
    /// Length of the entry's name in characters, for the width of its label.
    fn name_chars(&self, i: u32) -> u32;
}

/// A self-contained tree for benchmarks and tests: entries are added parent first, then
/// [`SimpleTree::finish`] links the children and sums the folder sizes and categories.
#[derive(Debug, Default, Clone)]
pub struct SimpleTree {
    parent: Vec<u32>,
    size: Vec<u64>,
    dir: Vec<bool>,
    category: Vec<u8>,
    name_chars: Vec<u16>,
    child_start: Vec<u32>,
    children: Vec<u32>,
}

impl SimpleTree {
    /// The root folder is entry 0.
    pub const ROOT: u32 = 0;

    /// A tree holding only its root folder.
    pub fn new() -> Self {
        let mut tree = Self::default();
        tree.push(Self::ROOT, 0, true, 0, 4);
        tree
    }

    pub fn len(&self) -> usize {
        self.size.len()
    }

    pub fn is_empty(&self) -> bool {
        self.size.is_empty()
    }

    fn push(&mut self, parent: u32, size: u64, dir: bool, category: u8, name_chars: u16) -> u32 {
        let index = u32::try_from(self.size.len()).unwrap_or(u32::MAX);
        self.parent.push(parent);
        self.size.push(size);
        self.dir.push(dir);
        self.category.push(category.min(CATEGORIES as u8 - 1));
        self.name_chars.push(name_chars);
        index
    }

    /// Adds a folder under `parent` (an entry added before) and returns its index.
    pub fn add_dir(&mut self, parent: u32, name_chars: u16) -> u32 {
        self.push(parent, 0, true, 0, name_chars)
    }

    /// Adds a file under `parent` (a folder added before) and returns its index.
    pub fn add_file(&mut self, parent: u32, size: u64, category: u8, name_chars: u16) -> u32 {
        self.push(parent, size, false, category, name_chars)
    }

    /// Links the children (largest first is not required: the layout sorts) and sums every
    /// folder's size and largest category.
    pub fn finish(&mut self) {
        let n = self.size.len();
        // Children in CSR form, counted then placed.
        let mut count = vec![0u32; n + 1];
        for i in 1..n {
            count[self.parent[i] as usize + 1] += 1;
        }
        for i in 0..n {
            count[i + 1] += count[i];
        }
        let mut cursor = count.clone();
        self.children = vec![0; n.saturating_sub(1)];
        for i in 1..n {
            let p = self.parent[i] as usize;
            self.children[cursor[p] as usize] = i as u32;
            cursor[p] += 1;
        }
        self.child_start = count;

        // Parents precede children, so one backward pass sums sizes and category bytes.
        let mut bytes = vec![[0u64; CATEGORIES]; n];
        for i in (0..n).rev() {
            if self.dir[i] {
                self.category[i] = largest(&bytes[i]);
            } else {
                bytes[i][self.category[i] as usize] += self.size[i];
            }
            if i > 0 {
                let p = self.parent[i] as usize;
                self.size[p] += self.size[i];
                let own = bytes[i];
                for (total, add) in bytes[p].iter_mut().zip(own) {
                    *total += add;
                }
            }
        }
    }
}

/// The index of the largest total (the first one on a tie).
fn largest(bytes: &[u64; CATEGORIES]) -> u8 {
    let mut best = 0;
    for (k, &b) in bytes.iter().enumerate() {
        if b > bytes[best] {
            best = k;
        }
    }
    best as u8
}

impl MapSource for SimpleTree {
    fn size(&self, i: u32) -> u64 {
        self.size[i as usize]
    }

    fn is_dir(&self, i: u32) -> bool {
        self.dir[i as usize]
    }

    fn children(&self, i: u32) -> &[u32] {
        let i = i as usize;
        match (self.child_start.get(i), self.child_start.get(i + 1)) {
            (Some(&start), Some(&end)) => &self.children[start as usize..end as usize],
            (Some(_), None) | (None, Some(_)) | (None, None) => &[],
        }
    }

    fn category(&self, i: u32) -> u8 {
        self.category[i as usize]
    }

    fn name_chars(&self, i: u32) -> u32 {
        u32::from(self.name_chars[i as usize])
    }
}

/// The scanned tree seen by the map: sizes and children read in place; one byte per entry holds
/// the category (files) or the largest category (folders), computed once by [`CoreSource::new`].
pub struct CoreSource<'a> {
    tree: &'a Tree,
    category: Vec<u8>,
}

impl<'a> CoreSource<'a> {
    /// `file_category` gives a file's category (below [`CATEGORIES`]). The extension map lives
    /// in `diskdome-core` once it exists; until then any function of the entry will do.
    /// One depth-first pass from the root sums the folders' largest categories.
    pub fn new(tree: &'a Tree, file_category: impl Fn(&Tree, u32) -> u8) -> Self {
        let mut category = vec![0u8; tree.len()];
        if tree.is_empty() {
            return Self { tree, category };
        }
        // Explicit stack: real trees can be deeper than the main thread's stack allows.
        struct Frame {
            entry: u32,
            next: usize,
            bytes: [u64; CATEGORIES],
        }
        let mut stack = vec![Frame {
            entry: tree.root,
            next: 0,
            bytes: [0; CATEGORIES],
        }];
        while let Some(top) = stack.last_mut() {
            let kids = tree.children(top.entry);
            if let Some(&child) = kids.get(top.next) {
                top.next += 1;
                if tree.is_dir(child) {
                    stack.push(Frame {
                        entry: child,
                        next: 0,
                        bytes: [0; CATEGORIES],
                    });
                } else {
                    let c = file_category(tree, child).min(CATEGORIES as u8 - 1);
                    category[child as usize] = c;
                    top.bytes[c as usize] += tree.size[child as usize];
                }
            } else {
                let done_entry = top.entry;
                let done = top.bytes;
                category[done_entry as usize] = largest(&done);
                stack.pop();
                if let Some(parent) = stack.last_mut() {
                    for (total, add) in parent.bytes.iter_mut().zip(done) {
                        *total += add;
                    }
                }
            }
        }
        Self { tree, category }
    }

    pub fn root(&self) -> u32 {
        self.tree.root
    }
}

impl MapSource for CoreSource<'_> {
    fn size(&self, i: u32) -> u64 {
        self.tree.size[i as usize]
    }

    fn is_dir(&self, i: u32) -> bool {
        self.tree.is_dir(i)
    }

    fn children(&self, i: u32) -> &[u32] {
        self.tree.children(i)
    }

    fn category(&self, i: u32) -> u8 {
        self.category[i as usize]
    }

    fn name_chars(&self, i: u32) -> u32 {
        u32::try_from(self.tree.name(i).chars().count()).unwrap_or(u32::MAX)
    }
}

#[cfg(test)]
mod tests {
    use super::{CoreSource, MapSource, SimpleTree};
    use diskdome_core::{Tree, flag};

    #[test]
    fn simple_tree_sums_sizes_and_largest_category() {
        let mut t = SimpleTree::new();
        let a = t.add_dir(SimpleTree::ROOT, 1);
        t.add_file(a, 10, 3, 1);
        t.add_file(a, 30, 5, 1);
        t.add_file(SimpleTree::ROOT, 15, 3, 1);
        t.finish();
        assert_eq!(t.size(SimpleTree::ROOT), 55);
        assert_eq!(t.size(a), 40);
        assert_eq!(t.category(a), 5);
        // Root: category 3 has 25 bytes, category 5 has 30.
        assert_eq!(t.category(SimpleTree::ROOT), 5);
        assert_eq!(t.children(SimpleTree::ROOT).len(), 2);
    }

    #[test]
    fn core_source_reads_the_scanned_tree() {
        let mut tree = Tree::with_slots(0, 0);
        let present = flag::PRESENT;
        let root = tree.push(0, 0, 0, 0, 0, 0, present | flag::DIR);
        let dir = tree.push(0, 0, root, 0, 0, 0, present | flag::DIR);
        tree.push(0, 0, dir, 100, 100, 0, present);
        tree.push(0, 0, root, 40, 40, 0, present);
        tree.finish(root);
        let source = CoreSource::new(&tree, |t, i| if t.size[i as usize] > 50 { 7 } else { 2 });
        assert_eq!(source.size(root), 140);
        assert_eq!(source.category(dir), 7);
        assert_eq!(source.category(root), 7);
    }
}
