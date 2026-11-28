//! Totals per extension and per category over a scanned tree (`docs/design-system.md` § 3.2, § 5.12 the side
//! panel, § 4.8 the saved summaries). One pass over the linked files; the extension is lower-cased into a key on the
//! stack, so no name allocates. A hash map holds one entry per distinct extension (a few thousand at most), never one
//! per file.

use std::collections::HashMap;

use crate::category::{FileCategory, category_of};
use crate::{Tree, flag};

/// The longest extension kept as its own row; a longer one counts as no extension.
pub const MAX_EXTENSION: usize = 16;

/// A lower-case extension, without the dot; empty for a file without one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Extension {
    bytes: [u8; MAX_EXTENSION],
    len: u8,
}

impl Extension {
    /// The extension of a file called `name`, lower-cased (empty: none, or longer than [`MAX_EXTENSION`]).
    pub fn of(name: &str) -> Self {
        let mut out = Self::default();
        let Some(dot) = name.rfind('.') else {
            return out;
        };
        let ext = &name.as_bytes()[dot + 1..];
        if ext.len() > MAX_EXTENSION {
            return out;
        }
        for (to, from) in out.bytes.iter_mut().zip(ext) {
            *to = from.to_ascii_lowercase();
        }
        out.len = ext.len() as u8;
        out
    }

    pub fn as_str(&self) -> &str {
        std::str::from_utf8(&self.bytes[..usize::from(self.len)]).unwrap_or("")
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}

/// What a group of files adds up to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Totals {
    pub files: u64,
    /// Logical size, bytes.
    pub size: u64,
    /// Size on disk, bytes.
    pub alloc: u64,
}

impl Totals {
    fn add(&mut self, size: u64, alloc: u64) {
        self.files += 1;
        self.size += size;
        self.alloc += alloc;
    }
}

/// One extension's row: its totals and its category.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExtensionRow {
    pub extension: Extension,
    pub category: FileCategory,
    pub totals: Totals,
}

/// The totals of a tree.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Stats {
    /// One row per extension, the largest on disk first (ties by name).
    pub extensions: Vec<ExtensionRow>,
    /// Indexed by `FileCategory as usize`, in [`FileCategory::ALL`]'s order.
    pub categories: [Totals; 12],
}

impl Stats {
    pub fn category(&self, category: FileCategory) -> Totals {
        self.categories[category as usize]
    }
}

/// Sums every linked file of `tree` (after [`Tree::finish`]) by extension and by category. A folder counts nothing:
/// its colour is its files'. An extra hard link counts as a file with no size on disk, as in the tree.
pub fn stats(tree: &Tree) -> Stats {
    let mut by_extension: HashMap<(Extension, FileCategory), Totals> = HashMap::new();
    let mut categories = [Totals::default(); 12];
    for &i in &tree.children {
        let flags = tree.flags[i as usize];
        if flags & flag::LINKED == 0 || flags & flag::DIR != 0 {
            continue;
        }
        let name = tree.name(i);
        let category = category_of(name);
        let (size, alloc) = (tree.size[i as usize], tree.alloc_bytes(i));
        by_extension
            .entry((Extension::of(name), category))
            .or_default()
            .add(size, alloc);
        categories[category as usize].add(size, alloc);
    }
    let mut extensions: Vec<ExtensionRow> = by_extension
        .into_iter()
        .map(|((extension, category), totals)| ExtensionRow {
            extension,
            category,
            totals,
        })
        .collect();
    extensions.sort_by(|a, b| {
        b.totals
            .alloc
            .cmp(&a.totals.alloc)
            .then_with(|| a.extension.as_str().cmp(b.extension.as_str()))
            .then_with(|| a.category.cmp(&b.category))
    });
    Stats {
        extensions,
        categories,
    }
}

#[cfg(test)]
mod tests {
    use super::{Extension, MAX_EXTENSION, stats};
    use crate::category::FileCategory;
    use crate::testing::add;
    use crate::{Tree, flag};

    /// A cluster as NTFS usually has it: sizes on disk are the clusters times this.
    const CLUSTER: u32 = 4096;

    #[test]
    fn extensions_are_lower_case_and_bounded() {
        assert_eq!(Extension::of("Movie.MKV").as_str(), "mkv");
        assert_eq!(Extension::of("a.tar.gz").as_str(), "gz");
        assert!(Extension::of("README").is_empty());
        assert!(Extension::of("x.").is_empty());
        let long = format!("x.{}", "e".repeat(MAX_EXTENSION + 1));
        assert!(Extension::of(&long).is_empty(), "longer than the key");
    }

    #[test]
    fn files_add_up_by_extension_and_category() {
        let mut t = Tree {
            cluster_size: CLUSTER,
            ..Tree::default()
        };
        let root = add(&mut t, ".", 0, 0, true);
        let videos = add(&mut t, "Videos", root, 0, true);
        add(&mut t, "a.mp4", videos, 100, false);
        add(&mut t, "B.MP4", videos, 50, false);
        add(&mut t, "c.mkv", videos, 30, false);
        add(&mut t, "notes.txt", root, 5, false);
        add(&mut t, "pagefile.sys", root, 400, false);
        add(&mut t, "driver.sys", root, 2, false);
        add(&mut t, "orphan.mp4", 99, 1000, false);
        t.finish(root);

        let s = stats(&t);
        let rows: Vec<(&str, FileCategory, u64, u64)> = s
            .extensions
            .iter()
            .map(|r| {
                (
                    r.extension.as_str(),
                    r.category,
                    r.totals.files,
                    r.totals.size,
                )
            })
            .collect();
        assert_eq!(
            rows,
            [
                ("sys", FileCategory::System, 1, 400),
                ("mp4", FileCategory::Video, 2, 150),
                ("mkv", FileCategory::Video, 1, 30),
                ("txt", FileCategory::Document, 1, 5),
                ("sys", FileCategory::App, 1, 2),
            ],
            "largest first; the page file is System, another .sys a program; the orphan is not counted"
        );
        assert_eq!(s.category(FileCategory::Video).size, 180);
        assert_eq!(s.category(FileCategory::Video).files, 3);
        assert_eq!(
            s.category(FileCategory::Other).files,
            0,
            "folders count nothing"
        );
        let total: u64 = s.categories.iter().map(|c| c.size).sum();
        assert_eq!(
            total, t.size[root as usize],
            "the categories add up to the drive"
        );
    }

    #[test]
    fn an_extra_hard_link_has_no_size_on_disk() {
        let mut t = Tree {
            cluster_size: CLUSTER,
            ..Tree::default()
        };
        let root = add(&mut t, ".", 0, 0, true);
        add(&mut t, "a.dll", root, 8, false);
        let link = add(&mut t, "b.dll", root, 8, false);
        t.flags[link as usize] |= flag::EXTRA_LINK;
        t.alloc[link as usize] = 0;
        t.finish(root);
        let app = stats(&t).category(FileCategory::App);
        assert_eq!((app.files, app.size), (2, 16));
        assert_eq!(
            app.alloc,
            8 * u64::from(t.cluster_size),
            "the clusters counted once"
        );
    }
}
