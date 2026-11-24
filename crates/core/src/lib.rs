//! DiskDome's data model.
//!
//! The scanned tree is stored as one array per field (struct of arrays) with `u32` indices:
//! names in UTF-8 pages addressed by one global offset, children in one CSR array, no full paths
//! stored.
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
    /// The file has more than one hard link (set on every link).
    pub const HARD_LINKS: u8 = 1 << 4;
    /// The file's data is split into more than one run of clusters.
    pub const FRAGMENTED: u8 = 1 << 5;
}

/// File attributes (`Tree::attrs`): our own 16 bits, mapped from the operating system's.
pub mod attr {
    pub const READONLY: u16 = 1 << 0;
    pub const HIDDEN: u16 = 1 << 1;
    pub const SYSTEM: u16 = 1 << 2;
    pub const ARCHIVE: u16 = 1 << 3;
    pub const TEMPORARY: u16 = 1 << 4;
    pub const SPARSE: u16 = 1 << 5;
    pub const REPARSE_POINT: u16 = 1 << 6;
    pub const COMPRESSED: u16 = 1 << 7;
    pub const OFFLINE: u16 = 1 << 8;
    pub const NOT_INDEXED: u16 = 1 << 9;
    pub const ENCRYPTED: u16 = 1 << 10;
    /// Cloud files (OneDrive): kept on this device.
    pub const PINNED: u16 = 1 << 11;
    /// Cloud files: may be freed from this device.
    pub const UNPINNED: u16 = 1 << 12;
    /// Cloud files: the content is online only and downloaded when read.
    pub const RECALL_ON_DATA_ACCESS: u16 = 1 << 14;

    /// Maps Windows' `FILE_ATTRIBUTE_*` bits (as stored in NTFS) to ours. Plain bit arithmetic: no OS
    /// dependency.
    pub fn from_windows(a: u32) -> u16 {
        // 0x0004_0000 is not mapped: on disk NTFS uses it for «has extended attributes» (the signed
        // files of WinSxS), which the API reports as nothing; `RECALL_ON_OPEN` shares the value.
        const MAP: [(u32, u16); 14] = [
            (0x0000_0001, READONLY),
            (0x0000_0002, HIDDEN),
            (0x0000_0004, SYSTEM),
            (0x0000_0020, ARCHIVE),
            (0x0000_0100, TEMPORARY),
            (0x0000_0200, SPARSE),
            (0x0000_0400, REPARSE_POINT),
            (0x0000_0800, COMPRESSED),
            (0x0000_1000, OFFLINE),
            (0x0000_2000, NOT_INDEXED),
            (0x0000_4000, ENCRYPTED),
            (0x0008_0000, PINNED),
            (0x0010_0000, UNPINNED),
            (0x0040_0000, RECALL_ON_DATA_ACCESS),
        ];
        MAP.iter()
            .filter(|(w, _)| a & w != 0)
            .fold(0, |acc, (_, ours)| acc | ours)
    }
}

/// `Tree::security` value for a security descriptor number that does not fit in 16 bits.
pub const SECURITY_OTHER: u16 = u16::MAX;

/// Every name, UTF-8, back to back in pages, addressed by one global byte offset. A name never spans
/// two pages. A scanner hands over the buffer each of its workers filled as one page, so the names
/// are never copied into a single buffer (which would hold them twice for a moment).
#[derive(Debug, Default)]
pub struct Names {
    pages: Vec<Vec<u8>>,
    /// Global offset of each page's first byte.
    starts: Vec<u32>,
    len: u32,
}

impl Names {
    /// Adds a page and returns the global offset of its first byte.
    pub fn push_page(&mut self, mut page: Vec<u8>) -> u32 {
        let base = self.len;
        if !page.is_empty() {
            page.shrink_to_fit();
            self.len = self
                .len
                .saturating_add(u32::try_from(page.len()).unwrap_or(u32::MAX));
            self.starts.push(base);
            self.pages.push(page);
        }
        base
    }

    /// Appends one name to the last page and returns its offset.
    pub fn push(&mut self, name: &[u8]) -> u32 {
        let off = self.len;
        if self.pages.is_empty() {
            self.starts.push(off);
            self.pages.push(Vec::new());
        }
        if let Some(last) = self.pages.last_mut() {
            last.extend_from_slice(name);
            self.len = self
                .len
                .saturating_add(u32::try_from(name.len()).unwrap_or(u32::MAX));
        }
        off
    }

    /// The `len` bytes at global offset `off` (empty if out of range).
    pub fn get(&self, off: u32, len: u16) -> &[u8] {
        let page = self.starts.partition_point(|&s| s <= off);
        let Some(index) = page.checked_sub(1) else {
            return &[];
        };
        let start = (off - self.starts[index]) as usize;
        self.pages[index]
            .get(start..start + len as usize)
            .unwrap_or(&[])
    }

    /// Total bytes of names.
    pub fn len(&self) -> usize {
        self.len as usize
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Bytes held, pages and their index included.
    pub fn heap_bytes(&self) -> usize {
        self.pages.iter().map(Vec::capacity).sum::<usize>()
            + self.pages.capacity() * size_of::<Vec<u8>>()
            + self.starts.capacity() * 4
    }

    /// Every byte in order, whatever the pages.
    pub fn bytes(&self) -> impl Iterator<Item = u8> + '_ {
        self.pages.iter().flatten().copied()
    }
}

/// Two `Names` are equal when they hold the same bytes, however they are paged.
impl PartialEq for Names {
    fn eq(&self, other: &Self) -> bool {
        self.len == other.len && self.bytes().eq(other.bytes())
    }
}

impl Eq for Names {}

/// One entry, for [`Tree::push`].
#[derive(Debug, Clone, Copy, Default)]
pub struct Entry {
    pub name_off: u32,
    pub name_len: u16,
    pub parent: u32,
    pub size: u64,
    /// Clusters.
    pub alloc: u32,
    pub mtime: u32,
    pub flags: u8,
    pub attrs: u16,
    pub security: u16,
}

/// Sentinel for "no entry" in `u32` index arrays.
pub const NONE: u32 = u32::MAX;

/// The scanned tree.
///
/// Every `Vec` indexed by entry has the same length (`len()`); `child_start` has one more element.
#[derive(Debug, Default)]
pub struct Tree {
    /// Every name, UTF-8.
    pub names: Names,
    pub name_off: Vec<u32>,
    pub name_len: Vec<u16>,
    /// Parent entry; the root is its own parent.
    pub parent: Vec<u32>,
    /// Logical size (folders: the sum of everything below, after `finish`).
    pub size: Vec<u64>,
    /// Size on disk in clusters of `cluster_size` bytes (folders: the sum of everything below, after
    /// `finish`). An NTFS volume has at most 2^32 clusters, so a `u32` holds a whole drive.
    pub alloc: Vec<u32>,
    pub cluster_size: u32,
    /// Last modification, Unix seconds (seconds, not minutes: equal to what Explorer shows).
    pub mtime: Vec<u32>,
    pub flags: Vec<u8>,
    /// File attributes, see [`attr`].
    pub attrs: Vec<u16>,
    /// Number of the file's security descriptor (NTFS `security_id`), [`SECURITY_OTHER`] above 16 bits.
    /// The owner is resolved from it later, through a table of descriptors.
    pub security: Vec<u16>,
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
    /// A tree with `len` empty slots.
    pub fn with_slots(len: usize) -> Self {
        Self {
            names: Names::default(),
            name_off: vec![0; len],
            name_len: vec![0; len],
            parent: vec![NONE; len],
            size: vec![0; len],
            alloc: vec![0; len],
            cluster_size: 4096,
            mtime: vec![0; len],
            flags: vec![0; len],
            attrs: vec![0; len],
            security: vec![0; len],
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

    /// Makes room for `extra` more entries without growing the arrays by doubling.
    pub fn reserve_exact(&mut self, extra: usize) {
        self.name_off.reserve_exact(extra);
        self.name_len.reserve_exact(extra);
        self.parent.reserve_exact(extra);
        self.size.reserve_exact(extra);
        self.alloc.reserve_exact(extra);
        self.mtime.reserve_exact(extra);
        self.flags.reserve_exact(extra);
        self.attrs.reserve_exact(extra);
        self.security.reserve_exact(extra);
    }

    /// Appends an entry and returns its index.
    pub fn push(&mut self, e: Entry) -> u32 {
        let index = u32::try_from(self.flags.len()).unwrap_or(NONE);
        self.name_off.push(e.name_off);
        self.name_len.push(e.name_len);
        self.parent.push(e.parent);
        self.size.push(e.size);
        self.alloc.push(e.alloc);
        self.mtime.push(e.mtime);
        self.flags.push(e.flags);
        self.attrs.push(e.attrs);
        self.security.push(e.security);
        index
    }

    /// Size on disk of entry `i` in bytes.
    pub fn alloc_bytes(&self, i: u32) -> u64 {
        u64::from(self.alloc[i as usize]) * u64::from(self.cluster_size)
    }

    /// The name of entry `i` (empty if the bytes are not valid UTF-8, which a scanner never writes).
    pub fn name(&self, i: u32) -> &str {
        let bytes = self
            .names
            .get(self.name_off[i as usize], self.name_len[i as usize]);
        std::str::from_utf8(bytes).unwrap_or("")
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
            self.alloc[p] = self.alloc[p].saturating_add(self.alloc[node as usize]);
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
        self.names.heap_bytes()
            + self.name_off.capacity() * 4
            + self.name_len.capacity() * 2
            + self.parent.capacity() * 4
            + self.size.capacity() * 8
            + self.alloc.capacity() * 4
            + self.mtime.capacity() * 4
            + self.flags.capacity()
            + self.attrs.capacity() * 2
            + self.security.capacity() * 2
            + self.child_start.capacity() * 4
            + self.children.capacity() * 4
    }
}

#[cfg(test)]
mod tests {
    use super::{Entry, NONE, Tree, attr, flag};

    fn add(tree: &mut Tree, name: &str, parent: u32, size: u64, dir: bool) -> u32 {
        let off = tree.names.push(name.as_bytes());
        let flags = flag::PRESENT | if dir { flag::DIR } else { 0 };
        tree.push(Entry {
            name_off: off,
            name_len: name.len() as u16,
            parent,
            size,
            alloc: size as u32,
            flags,
            ..Entry::default()
        })
    }

    #[test]
    fn names_are_found_across_pages() {
        let mut a = super::Names::default();
        assert_eq!(a.push_page(b"alphabeta".to_vec()), 0);
        assert_eq!(a.push_page(Vec::new()), 9, "an empty page takes no offset");
        assert_eq!(a.push_page(b"gamma".to_vec()), 9);
        assert_eq!(a.get(5, 4), b"beta");
        assert_eq!(a.get(9, 5), b"gamma");
        assert_eq!(a.get(12, 9), b"", "a name never spans two pages");
        let mut b = super::Names::default();
        b.push(b"alphabetagamma");
        assert_eq!(a, b, "equal bytes, different pages");
    }

    #[test]
    fn windows_attributes_map_to_ours() {
        // Hidden + system + recall on data access (an online-only OneDrive file) + a bit we drop.
        let a = attr::from_windows(0x2 | 0x4 | 0x40_0000 | 0x8000_0000);
        assert_eq!(a, attr::HIDDEN | attr::SYSTEM | attr::RECALL_ON_DATA_ACCESS);
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
        t.push(Entry {
            parent: NONE,
            ..Entry::default()
        });

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
