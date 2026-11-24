//! Fake entries for the table proof, laid out like the real model: one array per field, `u32` indices,
//! every name in one UTF-8 buffer. Nothing is stored per row as a heap object.

use std::cmp::Ordering;
use std::cmp::Reverse;

use rayon::prelude::{
    IndexedParallelIterator, IntoParallelIterator, ParallelIterator, ParallelSlice,
    ParallelSliceMut,
};

/// Entries that share one fake parent folder, so the "% of parent" column has something to show.
pub const SIBLINGS: usize = 64;

/// Rows generated per parallel block.
const BLOCK: usize = 1 << 16;

/// Longest generated name, in bytes.
const MAX_NAME: usize = 48;

const STEMS: [&str; 32] = [
    "report",
    "IMG",
    "Setup",
    "readme",
    "invoice",
    "DSC",
    "backup",
    "notes",
    "Thumbs",
    "chrome",
    "photo",
    "Video",
    "archive",
    "main",
    "index",
    "LICENSE",
    "budget",
    "Screenshot",
    "package",
    "config",
    "draft",
    "Meeting",
    "song",
    "track",
    "level",
    "save",
    "texture",
    "model",
    "shader",
    "cache",
    "temp",
    "Document",
];

const EXTENSIONS: [&str; 16] = [
    "pdf", "jpg", "exe", "txt", "docx", "png", "zip", "rs", "dll", "mp4", "json", "mp3", "dat",
    "log", "iso", "",
];

/// The columns the table can be sorted by.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SortColumn {
    Name,
    Size,
    Percent,
    Modified,
}

impl SortColumn {
    /// The column index used by the `.slint` side (0 name, 1 size, 2 % of parent, 3 modified).
    pub fn from_index(index: i32) -> Option<Self> {
        match index {
            0 => Some(Self::Name),
            1 => Some(Self::Size),
            2 => Some(Self::Percent),
            3 => Some(Self::Modified),
            _ => None,
        }
    }
}

/// Millions of fake entries, struct of arrays.
pub struct FakeTree {
    names: Vec<u8>,
    name_off: Vec<u32>,
    name_len: Vec<u8>,
    size: Vec<u64>,
    mtime: Vec<u32>,
    parent: Vec<u32>,
    parent_size: Vec<u64>,
}

/// A cheap, well-mixed hash of the row index (SplitMix64).
fn mix(mut x: u64) -> u64 {
    x = x.wrapping_add(0x9E37_79B9_7F4A_7C15);
    x = (x ^ (x >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    x ^ (x >> 31)
}

/// Writes the name of row `row` into `out` and returns its length.
fn fake_name(row: u64, out: &mut [u8; MAX_NAME]) -> usize {
    let h = mix(row);
    let stem = STEMS[(h % STEMS.len() as u64) as usize].as_bytes();
    let ext = EXTENSIONS[((h >> 8) % EXTENSIONS.len() as u64) as usize].as_bytes();
    let mut number = (h >> 16) % 10u64.pow(1 + ((h >> 40) % 7) as u32);

    let mut len = 0;
    out[..stem.len()].copy_from_slice(stem);
    len += stem.len();
    out[len] = b'_';
    len += 1;

    let mut digits = [0u8; 20];
    let mut count = 0;
    loop {
        digits[count] = b'0' + (number % 10) as u8;
        count += 1;
        number /= 10;
        if number == 0 {
            break;
        }
    }
    for digit in digits[..count].iter().rev() {
        out[len] = *digit;
        len += 1;
    }

    if !ext.is_empty() {
        out[len] = b'.';
        len += 1;
        out[len..len + ext.len()].copy_from_slice(ext);
        len += ext.len();
    }
    len
}

/// Size in bytes, spread over every order of magnitude from 0 to 32 GB.
fn fake_size(row: u64) -> u64 {
    let h = mix(row ^ 0xA5A5_A5A5);
    let bits = (h >> 58) % 36;
    if bits == 0 {
        0
    } else {
        mix(h) & ((1u64 << bits) - 1)
    }
}

/// Modification time, Unix seconds between 2004 and 2026.
fn fake_mtime(row: u64) -> u32 {
    let h = mix(row ^ 0x5A5A_5A5A);
    1_080_000_000 + (h % 700_000_000) as u32
}

impl FakeTree {
    /// Generates `rows` entries in parallel.
    pub fn generate(rows: u32) -> Self {
        let count = rows as usize;

        // Pass 1: every field but the names, plus each name's length.
        let mut name_len = vec![0u8; count];
        let mut size = vec![0u64; count];
        let mut mtime = vec![0u32; count];
        name_len
            .par_chunks_mut(BLOCK)
            .zip(size.par_chunks_mut(BLOCK))
            .zip(mtime.par_chunks_mut(BLOCK))
            .enumerate()
            .for_each(|(block, ((lens, sizes), mtimes))| {
                let mut buffer = [0u8; MAX_NAME];
                let first = (block * BLOCK) as u64;
                for (i, ((len, size), mtime)) in lens
                    .iter_mut()
                    .zip(sizes.iter_mut())
                    .zip(mtimes.iter_mut())
                    .enumerate()
                {
                    let row = first + i as u64;
                    *len = fake_name(row, &mut buffer) as u8;
                    *size = fake_size(row);
                    *mtime = fake_mtime(row);
                }
            });

        // Offsets: a prefix sum of the lengths.
        let mut name_off = Vec::with_capacity(count);
        let mut total = 0u32;
        for len in &name_len {
            name_off.push(total);
            total += u32::from(*len);
        }

        // Pass 2: the names, written in parallel into disjoint slices of one buffer.
        let mut names = vec![0u8; total as usize];
        let mut slices = Vec::with_capacity(count.div_ceil(BLOCK));
        let mut rest = names.as_mut_slice();
        for block_start in (0..count).step_by(BLOCK) {
            let block_end = (block_start + BLOCK).min(count);
            let end = if block_end == count {
                total
            } else {
                name_off[block_end]
            };
            let (head, tail) = rest.split_at_mut((end - name_off[block_start]) as usize);
            slices.push((block_start, head));
            rest = tail;
        }
        slices.into_par_iter().for_each(|(block_start, slice)| {
            let mut buffer = [0u8; MAX_NAME];
            let mut at = 0;
            let block_end = (block_start + BLOCK).min(count);
            for row in block_start..block_end {
                let len = fake_name(row as u64, &mut buffer);
                slice[at..at + len].copy_from_slice(&buffer[..len]);
                at += len;
            }
        });

        // Fake folders: every SIBLINGS consecutive rows share a parent, whose size is their sum.
        let parent = (0..rows).map(|row| row / SIBLINGS as u32).collect();
        let parent_size = size
            .par_chunks(SIBLINGS)
            .map(|chunk| chunk.iter().sum())
            .collect();

        Self {
            names,
            name_off,
            name_len,
            size,
            mtime,
            parent,
            parent_size,
        }
    }

    pub fn len(&self) -> usize {
        self.size.len()
    }

    pub fn is_empty(&self) -> bool {
        self.size.is_empty()
    }

    pub fn name(&self, row: u32) -> &[u8] {
        let start = self.name_off[row as usize] as usize;
        &self.names[start..start + usize::from(self.name_len[row as usize])]
    }

    pub fn size(&self, row: u32) -> u64 {
        self.size[row as usize]
    }

    pub fn mtime(&self, row: u32) -> u32 {
        self.mtime[row as usize]
    }

    pub fn parent_size(&self, row: u32) -> u64 {
        self.parent_size[self.parent[row as usize] as usize]
    }

    /// Bytes held by the arrays (capacity, not length).
    pub fn heap_bytes(&self) -> usize {
        self.names.capacity()
            + self.name_off.capacity() * 4
            + self.name_len.capacity()
            + self.size.capacity() * 8
            + self.mtime.capacity() * 4
            + self.parent.capacity() * 4
            + self.parent_size.capacity() * 8
    }

    /// The row order for a column, computed in parallel. Ties are broken by row index, so the result is
    /// deterministic and a descending order is not merely the ascending one reversed.
    pub fn sorted_order(&self, column: SortColumn, descending: bool) -> Vec<u32> {
        let mut order: Vec<u32> = (0..self.len() as u32).collect();
        match (column, descending) {
            (SortColumn::Size, false) => {
                order.par_sort_unstable_by_key(|&row| (self.size(row), row))
            }
            (SortColumn::Size, true) => {
                order.par_sort_unstable_by_key(|&row| (Reverse(self.size(row)), row))
            }
            (SortColumn::Modified, false) => {
                order.par_sort_unstable_by_key(|&row| (self.mtime(row), row))
            }
            (SortColumn::Modified, true) => {
                order.par_sort_unstable_by_key(|&row| (Reverse(self.mtime(row)), row));
            }
            (SortColumn::Name, descending) => return self.sorted_by_name(descending),
            (SortColumn::Percent, descending) => order.par_sort_unstable_by(|&a, &b| {
                let ordering = self.compare_percent(a, b);
                if descending {
                    ordering.reverse()
                } else {
                    ordering
                }
                .then(a.cmp(&b))
            }),
        }
        order
    }

    /// Sorts (first 8 bytes of the name, row) pairs: most comparisons end on the prefix, in a contiguous
    /// array, instead of chasing two names through the buffer. Costs 16 bytes per row while it runs.
    fn sorted_by_name(&self, descending: bool) -> Vec<u32> {
        let mut keys: Vec<(u64, u32)> = (0..self.len() as u32)
            .into_par_iter()
            .map(|row| (name_prefix(self.name(row)), row))
            .collect();
        keys.par_sort_unstable_by(|a, b| {
            let ordering =
                a.0.cmp(&b.0)
                    .then_with(|| compare_names(self.name(a.1), self.name(b.1)));
            if descending {
                ordering.reverse()
            } else {
                ordering
            }
            .then(a.1.cmp(&b.1))
        });
        keys.into_par_iter().map(|(_, row)| row).collect()
    }

    /// Compares size / parent size of two rows exactly, without floating point.
    fn compare_percent(&self, a: u32, b: u32) -> Ordering {
        let (sa, pa) = (u128::from(self.size(a)), u128::from(self.parent_size(a)));
        let (sb, pb) = (u128::from(self.size(b)), u128::from(self.parent_size(b)));
        match (pa, pb) {
            (0, 0) => Ordering::Equal,
            (0, _) => Ordering::Less,
            (_, 0) => Ordering::Greater,
            (_, _) => (sa * pb).cmp(&(sb * pa)),
        }
    }
}

/// The first 8 bytes of a name, lowercased, big-endian: comparing two prefixes gives the order of
/// `compare_names` whenever they differ.
fn name_prefix(name: &[u8]) -> u64 {
    let mut bytes = [0u8; 8];
    for (byte, source) in bytes.iter_mut().zip(name) {
        *byte = source.to_ascii_lowercase();
    }
    u64::from_be_bytes(bytes)
}

/// ASCII case-insensitive comparison. The real app needs a proper collation; this is enough to measure.
pub fn compare_names(a: &[u8], b: &[u8]) -> Ordering {
    a.iter()
        .map(u8::to_ascii_lowercase)
        .cmp(b.iter().map(u8::to_ascii_lowercase))
}

#[cfg(test)]
mod tests {
    use super::{FakeTree, SIBLINGS, SortColumn, compare_names};

    #[test]
    fn generation_is_deterministic_and_consistent() {
        let a = FakeTree::generate(200_000);
        let b = FakeTree::generate(200_000);
        assert_eq!(a.len(), 200_000);
        for row in [0u32, 1, 65_535, 65_536, 199_999] {
            assert_eq!(a.name(row), b.name(row));
            assert!(!a.name(row).is_empty());
            assert!(std::str::from_utf8(a.name(row)).is_ok());
        }
        let parent_total: u64 = (0..SIBLINGS as u32).map(|row| a.size(row)).sum();
        assert_eq!(a.parent_size(0), parent_total);
    }

    #[test]
    fn every_sort_is_a_permutation_in_order() {
        let tree = FakeTree::generate(50_000);
        for column in [
            SortColumn::Name,
            SortColumn::Size,
            SortColumn::Percent,
            SortColumn::Modified,
        ] {
            for descending in [false, true] {
                let order = tree.sorted_order(column, descending);
                let mut seen = vec![false; tree.len()];
                for &row in &order {
                    assert!(!seen[row as usize]);
                    seen[row as usize] = true;
                }
                for pair in order.windows(2) {
                    let (a, b) = (pair[0], pair[1]);
                    let ordering = match column {
                        SortColumn::Name => compare_names(tree.name(a), tree.name(b)),
                        SortColumn::Size => tree.size(a).cmp(&tree.size(b)),
                        SortColumn::Percent => tree.compare_percent(a, b),
                        SortColumn::Modified => tree.mtime(a).cmp(&tree.mtime(b)),
                    };
                    let ordering = if descending {
                        ordering.reverse()
                    } else {
                        ordering
                    };
                    assert!(
                        ordering.is_le(),
                        "{column:?} descending={descending}: rows {a} and {b}"
                    );
                }
            }
        }
    }
}
