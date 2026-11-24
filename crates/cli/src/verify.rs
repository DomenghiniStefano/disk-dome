//! `diskdome-cli verify --dump FILE`: for a sample of files, the values read from the `$MFT`
//! against what Windows reports for the same file (File Explorer's Properties): last-modified date,
//! size, size on disk, attributes, hard links. Card 4.T12.
//!
//! The dump is a snapshot: a file whose date differs was changed since, and is counted apart rather
//! than compared.

use diskdome_core::{Tree, attr, flag};
use diskdome_platform::file_info;

use crate::export::SEP;

/// One mismatch kept for the report.
struct Miss {
    path: String,
    ours: u64,
    theirs: u64,
}

#[derive(Default)]
struct Field {
    equal: u64,
    misses: Vec<Miss>,
    different: u64,
}

impl Field {
    fn check(&mut self, path: &str, ours: u64, theirs: u64) {
        if ours == theirs {
            self.equal += 1;
        } else {
            self.different += 1;
            if self.misses.len() < 8 {
                self.misses.push(Miss {
                    path: path.to_string(),
                    ours,
                    theirs,
                });
            }
        }
    }

    fn print(&self, name: &str) {
        println!(
            "{name:<34} equal {:>6}, different {:>5}",
            self.equal, self.different
        );
        for m in &self.misses {
            println!("    {}  ours {} vs Windows {}", m.path, m.ours, m.theirs);
        }
    }
}

pub fn run(tree: &Tree, records: u64, drive: char, sample: usize) {
    // Files reachable from the root, one entry per file (its record, not its extra links).
    let files: Vec<u32> = (0..u32::try_from(records).unwrap_or(u32::MAX))
        .filter(|&i| {
            let f = tree.flags[i as usize];
            f & flag::LINKED != 0 && f & flag::DIR == 0 && f & flag::EXTRA_LINK == 0
        })
        .collect();
    let step = (files.len() / sample.max(1)).max(1);
    let mut opened = 0u64;
    let mut missing = 0u64;
    let mut changed = 0u64;
    let mut mtime = Field::default();
    let mut size = Field::default();
    let mut alloc = Field::default();
    // Size on disk that differs from `AllocationSize` for a known reason.
    let mut resident = 0u64;
    let mut compressed = 0u64;
    let mut attrs = Field::default();
    let mut links = Field::default();
    // Bits Windows may report that the scan does not compare: ARCHIVE flips on every write.
    let mask = !attr::ARCHIVE;

    for &i in files.iter().step_by(step).take(sample) {
        let path = format!("{drive}:{SEP}{}", tree.path(i, SEP));
        let Some(info) = file_info(&path) else {
            missing += 1;
            continue;
        };
        opened += 1;
        mtime.check(
            &path,
            u64::from(tree.mtime[i as usize]),
            u64::from(info.mtime),
        );
        if tree.mtime[i as usize] != info.mtime {
            changed += 1;
            continue;
        }
        size.check(&path, tree.size[i as usize], info.size);
        let ours = tree.alloc_bytes(i);
        let cluster = u64::from(tree.cluster_size);
        let a = tree.attrs[i as usize];
        if ours == 0 && info.alloc < cluster && info.alloc == info.size.div_ceil(8) * 8 {
            // Kept inside the MFT record: no cluster. Explorer's «Size on disk» says 0 too; the API
            // reports the size rounded to 8 bytes.
            resident += 1;
        } else if a & (attr::COMPRESSED | attr::SPARSE) != 0
            && ours == info.compressed.div_ceil(cluster) * cluster
        {
            // Compressed or sparse: the clusters really used, as Explorer shows.
            compressed += 1;
        } else {
            alloc.check(&path, ours, info.alloc);
        }
        attrs.check(
            &path,
            u64::from(tree.attrs[i as usize] & mask),
            u64::from(attr::from_windows(info.attributes) & mask),
        );
        links.check(
            &path,
            u64::from(tree.flags[i as usize] & flag::HARD_LINKS != 0),
            u64::from(info.links > 1),
        );
    }

    println!(
        "sample: {} files of {}; opened {opened}, gone or unreadable {missing}, changed since the dump {changed}",
        files.len().min(sample),
        files.len()
    );
    mtime.print("last modified (all opened)");
    size.print("size");
    alloc.print("size on disk");
    println!("    + resident in the MFT, 0 like Explorer: {resident}");
    println!("    + compressed or sparse, equal to the compressed size: {compressed}");
    attrs.print("attributes (archive bit ignored)");
    links.print("has hard links");
}
