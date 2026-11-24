//! DiskDome's data model.
//!
//! The scanned tree is stored as one array per field (struct of arrays) with `u32` indices:
//! names in a single UTF-8 buffer, children in one CSR array, no full paths stored.
//! Scanners write into it; the interface, the treemap and the CLI read from it without copying.
//! See `docs/architecture.md` for the layout and the memory budget (~65 bytes per entry).
//!
//! A scanner fills the per-entry arrays (an entry's index is the scanner's own dense number, the
//! MFT record number for NTFS), then calls [`Tree::finish`], which links the children of every
//! folder and sums the folder totals bottom-up.

/// Entry flags (`Tree::flags`).
pub mod flag {
    /// The slot holds an entry. Slots without it (free MFT records, extension records) are skipped.
    pub const PRESENT: u8 = 1 << 0;
    /// The entry is a folder.
    pub const DIR: u8 = 1 << 1;
    /// A second (third…) hard link to a file listed elsewhere: it shows the file's size, but its
    /// size on disk is 0, so the clusters are counted once (as WizTree does).
    pub const EXTRA_LINK: u8 = 1 << 2;
    /// Set by [`super::Tree::finish`]: the entry is reachable from the root.
    pub const LINKED: u8 = 1 << 3;
}

/// Sentinel for "no entry" in `u32` index arrays.
pub const NONE: u32 = u32::MAX;

/// The scanned tree.
///
/// Every `Vec` indexed by entry has the same length (`len()`); `child_start` has one more element.
#[derive(Debug, Default)]
pub struct Tree {
    /// Every name, UTF-8, back to back.
    pub names: Vec<u8>,
    pub name_off: Vec<u32>,
    pub name_len: Vec<u16>,
    /// Parent entry; the root is its own parent.
    pub parent: Vec<u32>,
    /// Logical size (folders: the sum of everything below, after `finish`).
    pub size: Vec<u64>,
    /// Size on disk (folders: the sum of everything below, after `finish`).
    pub alloc: Vec<u64>,
    /// Last modification, Unix seconds.
    pub mtime: Vec<u32>,
    pub flags: Vec<u8>,
    /// CSR: the children of entry `i` are `children[child_start[i]..child_start[i + 1]]`.
    pub child_start: Vec<u32>,
    pub children: Vec<u32>,
    pub root: u32,
}

/// What [`Tree::finish`] found.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct FinishStats {
    /// Entries reachable from the root (the root included).
    pub linked: u32,
    pub files: u32,
    pub dirs: u32,
    /// Present entries whose parent chain does not reach the root (orphans, broken records).
    pub unreachable: u32,
}

impl Tree {
    /// A tree with `len` empty slots and room for `name_bytes` bytes of names.
    pub fn with_slots(len: usize, name_bytes: usize) -> Self {
        Self {
            names: Vec::with_capacity(name_bytes),
            name_off: vec![0; len],
            name_len: vec![0; len],
            parent: vec![NONE; len],
            size: vec![0; len],
            alloc: vec![0; len],
            mtime: vec![0; len],
            flags: vec![0; len],
            child_start: Vec::new(),
            children: Vec::new(),
            root: 0,
        }
    }

    /// Number of slots (present or not).
    pub fn len(&self) -> usize {
        self.flags.len()
    }

    pub fn is_empty(&self) -> bool {
        self.flags.is_empty()
    }

    /// Appends a slot and returns its index.
    #[allow(
        clippy::too_many_arguments,
        reason = "one argument per field of the entry"
    )]
    pub fn push(
        &mut self,
        name_off: u32,
        name_len: u16,
        parent: u32,
        size: u64,
        alloc: u64,
        mtime: u32,
        flags: u8,
    ) -> u32 {
        let index = u32::try_from(self.flags.len()).unwrap_or(NONE);
        self.name_off.push(name_off);
        self.name_len.push(name_len);
        self.parent.push(parent);
        self.size.push(size);
        self.alloc.push(alloc);
        self.mtime.push(mtime);
        self.flags.push(flags);
        index
    }

    /// The name of entry `i` (empty if the bytes are not valid UTF-8, which a scanner never writes).
    pub fn name(&self, i: u32) -> &str {
        let start = self.name_off[i as usize] as usize;
        let end = start + self.name_len[i as usize] as usize;
        self.names
            .get(start..end)
            .and_then(|bytes| std::str::from_utf8(bytes).ok())
            .unwrap_or("")
    }

    pub fn is_dir(&self, i: u32) -> bool {
        self.flags[i as usize] & flag::DIR != 0
    }

    /// The children of entry `i` (empty before `finish`).
    pub fn children(&self, i: u32) -> &[u32] {
        let i = i as usize;
        match (self.child_start.get(i), self.child_start.get(i + 1)) {
            (Some(&start), Some(&end)) => &self.children[start as usize..end as usize],
            (Some(_), None) | (None, Some(_)) | (None, None) => &[],
        }
    }

    /// The path of entry `i` below the root, with `separator` between names (the root's own name is
    /// not included). Walks up the parents: nothing stores full paths.
    pub fn path(&self, i: u32, separator: char) -> String {
        let mut chain = Vec::new();
        let mut node = i;
        while node != self.root && node != NONE && chain.len() < 4096 {
            chain.push(node);
            node = self.parent[node as usize];
        }
        let mut out = String::new();
        for (k, node) in chain.iter().rev().enumerate() {
            if k > 0 {
                out.push(separator);
            }
            out.push_str(self.name(*node));
        }
        out
    }

    /// Links children to parents (CSR) for every present entry reachable from `root`, and sums sizes
    /// bottom-up so every folder holds the total of its subtree.
    ///
    /// Entries whose parent is missing, not a folder, or out of range are left unlinked and counted
    /// in `unreachable`. Allocation: two `u32` arrays (`child_start`, `children`) kept, one `u32`
    /// array (the visit order) dropped on return.
    pub fn finish(&mut self, root: u32) -> FinishStats {
        let n = self.len();
        self.root = root;
        let mut stats = FinishStats::default();
        if (root as usize) >= n {
            return stats;
        }

        // 1. Count the children of every folder.
        let mut start = vec![0u32; n + 1];
        for i in 0..n {
            if let Some(p) = self.linkable_parent(i, root) {
                start[p as usize + 1] += 1;
            }
        }
        for i in 0..n {
            start[i + 1] += start[i];
        }
        // 2. Place them.
        let total = start[n] as usize;
        let mut children = vec![NONE; total];
        let mut cursor: Vec<u32> = start[..n].to_vec();
        for i in 0..n {
            if let Some(p) = self.linkable_parent(i, root) {
                let slot = &mut cursor[p as usize];
                children[*slot as usize] = i as u32;
                *slot += 1;
            }
        }
        drop(cursor);
        self.child_start = start;
        self.children = children;

        // 3. Breadth-first from the root: every reachable entry once, parents before children.
        let mut order: Vec<u32> = Vec::with_capacity(n);
        order.push(root);
        self.flags[root as usize] |= flag::LINKED;
        let mut head = 0;
        while head < order.len() {
            let node = order[head] as usize;
            head += 1;
            let (a, b) = (
                self.child_start[node] as usize,
                self.child_start[node + 1] as usize,
            );
            for k in a..b {
                let child = self.children[k];
                let f = &mut self.flags[child as usize];
                if *f & flag::LINKED == 0 {
                    *f |= flag::LINKED;
                    order.push(child);
                }
            }
        }

        // 4. Folder totals: folders start from zero, then children are added in reverse visit order.
        for &node in &order {
            if self.flags[node as usize] & flag::DIR != 0 {
                self.size[node as usize] = 0;
                self.alloc[node as usize] = 0;
                stats.dirs += 1;
            } else {
                stats.files += 1;
            }
        }
        for &node in order.iter().rev() {
            if node == root {
                continue;
            }
            let p = self.parent[node as usize] as usize;
            self.size[p] += self.size[node as usize];
            self.alloc[p] += self.alloc[node as usize];
        }

        stats.linked = order.len() as u32;
        let present = self
            .flags
            .iter()
            .filter(|f| **f & flag::PRESENT != 0)
            .count() as u32;
        stats.unreachable = present.saturating_sub(stats.linked);
        stats
    }

    /// The parent `i` hangs from, if `i` is a present non-root entry whose parent is a present folder.
    fn linkable_parent(&self, i: usize, root: u32) -> Option<u32> {
        if i == root as usize || self.flags[i] & flag::PRESENT == 0 {
            return None;
        }
        let p = self.parent[i];
        let pf = *self.flags.get(p as usize)?;
        (pf & flag::PRESENT != 0 && pf & flag::DIR != 0 && p as usize != i).then_some(p)
    }

    /// Bytes held by the model's arrays (capacity, not length): the number the memory budget is
    /// about.
    pub fn heap_bytes(&self) -> usize {
        self.names.capacity()
            + self.name_off.capacity() * 4
            + self.name_len.capacity() * 2
            + self.parent.capacity() * 4
            + self.size.capacity() * 8
            + self.alloc.capacity() * 8
            + self.mtime.capacity() * 4
            + self.flags.capacity()
            + self.child_start.capacity() * 4
            + self.children.capacity() * 4
    }
}

#[cfg(test)]
mod tests {
    use super::{NONE, Tree, flag};

    fn add(tree: &mut Tree, name: &str, parent: u32, size: u64, dir: bool) -> u32 {
        let off = tree.names.len() as u32;
        tree.names.extend_from_slice(name.as_bytes());
        let flags = flag::PRESENT | if dir { flag::DIR } else { 0 };
        tree.push(off, name.len() as u16, parent, size, size, 0, flags)
    }

    #[test]
    fn totals_and_paths() {
        let mut t = Tree::default();
        let root = add(&mut t, ".", 0, 0, true);
        let a = add(&mut t, "a", root, 0, true);
        let f1 = add(&mut t, "f1", a, 10, false);
        let b = add(&mut t, "b", a, 0, true);
        let _f2 = add(&mut t, "f2", b, 5, false);
        let _f3 = add(&mut t, "f3", root, 7, false);
        let orphan = add(&mut t, "orphan", 99, 1000, false);
        let under_file = add(&mut t, "x", f1, 1000, false);
        // A free slot.
        t.push(0, 0, NONE, 0, 0, 0, 0);

        let stats = t.finish(root);
        assert_eq!(stats.linked, 6);
        assert_eq!(stats.dirs, 3);
        assert_eq!(stats.files, 3);
        assert_eq!(stats.unreachable, 2);
        assert_eq!(t.size[root as usize], 22);
        assert_eq!(t.size[a as usize], 15);
        assert_eq!(t.size[b as usize], 5);
        assert_eq!(t.path(4, '\\'), "a\\b\\f2");
        assert_eq!(t.children(a), &[f1, b]);
        assert_eq!(t.flags[orphan as usize] & flag::LINKED, 0);
        assert_eq!(t.flags[under_file as usize] & flag::LINKED, 0);
    }

    #[test]
    fn a_parent_cycle_is_not_counted() {
        let mut t = Tree::default();
        let root = add(&mut t, ".", 0, 0, true);
        // Two folders pointing at each other, off the root.
        let x = add(&mut t, "x", 2, 0, true);
        let _y = add(&mut t, "y", x, 3, true);
        let stats = t.finish(root);
        assert_eq!(stats.linked, 1);
        assert_eq!(stats.unreachable, 2);
        assert_eq!(t.size[root as usize], 0);
    }
}
