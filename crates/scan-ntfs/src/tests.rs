//! End-to-end tests of the pipeline on `$MFT` images built in memory.

use diskdome_core::flag;

use crate::pipeline::{Options, scan};
use crate::record::{NS_DOS, NS_POSIX, NS_WIN32, NS_WIN32_AND_DOS};
use crate::synthetic::{MemorySource, RecordSpec, SyntheticSource, write_record};

const RS: usize = 1024;

/// An image of `count` records, all free except those `fill` writes.
fn image(count: usize, fill: impl Fn(u64, &mut [u8])) -> MemorySource {
    let mut data = vec![0u8; count * RS];
    let (records, _) = data.as_chunks_mut::<RS>();
    for (r, rec) in records.iter_mut().enumerate() {
        fill(r as u64, rec);
    }
    MemorySource {
        data,
        record_size: RS,
    }
}

fn small_options() -> Options {
    // Tiny chunks and several threads, so records spread over many chunks and workers.
    Options {
        threads: 3,
        chunk_bytes: 4 * RS,
    }
}

#[test]
fn builds_the_tree_with_totals() {
    let src = image(40, |r, rec| match r {
        5 => write_record(
            rec,
            r,
            &RecordSpec {
                dir: true,
                names: &[(5, ".", NS_WIN32_AND_DOS)],
                ..RecordSpec::default()
            },
        ),
        // $MFT-like system file in the root.
        0 => write_record(
            rec,
            r,
            &RecordSpec {
                names: &[(5, "$MFT", NS_WIN32_AND_DOS)],
                size: Some((40 * 1024, 40 * 1024)),
                ..RecordSpec::default()
            },
        ),
        // Folder "Users" with a long name and a DOS alias.
        16 => write_record(
            rec,
            r,
            &RecordSpec {
                dir: true,
                names: &[(5, "USERS~1", NS_DOS), (5, "Users", NS_WIN32)],
                ..RecordSpec::default()
            },
        ),
        // Files in Users.
        17 => write_record(
            rec,
            r,
            &RecordSpec {
                names: &[(16, "big.bin", NS_WIN32_AND_DOS)],
                size: Some((10_000, 12_288)),
                mtime_unix: 1_650_000_000,
                ..RecordSpec::default()
            },
        ),
        18 => write_record(
            rec,
            r,
            &RecordSpec {
                names: &[(16, "tiny.txt", NS_POSIX)],
                size: Some((100, 0)),
                ..RecordSpec::default()
            },
        ),
        // A file hard-linked into the root and into Users: counted once.
        19 => write_record(
            rec,
            r,
            &RecordSpec {
                names: &[(16, "link-a", NS_WIN32), (5, "link-b", NS_WIN32)],
                size: Some((5_000, 8_192)),
                ..RecordSpec::default()
            },
        ),
        // A file whose sizes and name live in an extension record (attribute list).
        20 => write_record(
            rec,
            r,
            &RecordSpec {
                ..RecordSpec::default()
            },
        ),
        33 => write_record(
            rec,
            r,
            &RecordSpec {
                base: 20,
                names: &[(16, "fragmented.vhdx", NS_WIN32)],
                size: Some((1_000_000, 1_048_576)),
                ..RecordSpec::default()
            },
        ),
        // A deleted file: not in use.
        21 => write_record(
            rec,
            r,
            &RecordSpec {
                in_use: false,
                names: &[(16, "deleted", NS_WIN32)],
                size: Some((999_999, 1_000_000)),
                ..RecordSpec::default()
            },
        ),
        // A torn record: fixup fails.
        22 => {
            write_record(
                rec,
                r,
                &RecordSpec {
                    names: &[(16, "torn", NS_WIN32)],
                    size: Some((777, 4096)),
                    ..RecordSpec::default()
                },
            );
            rec[1022] ^= 0xFF;
        }
        // An orphan: its parent is a free record.
        23 => write_record(
            rec,
            r,
            &RecordSpec {
                names: &[(30, "orphan", NS_WIN32)],
                size: Some((1, 0)),
                ..RecordSpec::default()
            },
        ),
        _ => {}
    });

    let (tree, report) = scan(&src, &small_options()).expect("scan");
    assert_eq!(report.records, 40);
    assert_eq!(report.extension_records, 1);
    assert_eq!(report.extra_links, 1);
    assert_eq!(report.tree.unreachable, 1, "the orphan");

    assert_eq!(
        tree.name(16),
        "Users",
        "the long name wins over the DOS alias"
    );
    assert_eq!(tree.name(20), "fragmented.vhdx");
    assert_eq!(tree.mtime[17], 1_650_000_000);
    assert_eq!(tree.size[20], 1_000_000);
    assert_eq!(tree.flags[21] & flag::PRESENT, 0);
    assert_eq!(tree.flags[22] & flag::PRESENT, 0);

    let users = 10_000 + 100 + 5_000 + 1_000_000;
    assert_eq!(tree.size[16], users);
    assert_eq!(tree.alloc[16], 12_288 + 8_192 + 1_048_576);
    assert_eq!(
        tree.size[5],
        users + 40 * 1024 + 5_000,
        "the extra link shows the file's size"
    );
    assert_eq!(
        tree.alloc[5],
        12_288 + 8_192 + 1_048_576 + 40 * 1024,
        "the extra link's clusters are counted once"
    );
    assert_eq!(tree.path(20, '\\'), "Users\\fragmented.vhdx");

    let extra = tree
        .children(5)
        .iter()
        .copied()
        .find(|&c| tree.name(c) == "link-b");
    let extra = extra.expect("the second link is listed in the root");
    assert_ne!(tree.flags[extra as usize] & flag::EXTRA_LINK, 0);
    assert_eq!(tree.size[extra as usize], 5_000);
    assert_eq!(tree.alloc[extra as usize], 0);
}

#[test]
fn chunking_does_not_change_the_result() {
    let src = SyntheticSource {
        records: 50_000,
        record_size: RS,
    };
    let (a, ra) = scan(
        &src,
        &Options {
            threads: 1,
            chunk_bytes: 1 << 20,
        },
    )
    .expect("scan");
    let (b, rb) = scan(
        &src,
        &Options {
            threads: 7,
            chunk_bytes: 3 * RS,
        },
    )
    .expect("scan");
    assert_eq!(ra.tree, rb.tree);
    assert_eq!(a.size, b.size);
    assert_eq!(a.alloc, b.alloc);
    assert_eq!(a.names, b.names);
    assert!(ra.tree.dirs > 6000 && ra.tree.files > 40_000);
    assert_eq!(ra.tree.unreachable, 0);
    // The root holds the sum of every file.
    let files: u64 = (0..a.len() as u32)
        .filter(|&i| a.flags[i as usize] & flag::LINKED != 0 && !a.is_dir(i))
        .map(|i| a.size[i as usize])
        .sum();
    assert_eq!(a.size[5], files);
}

#[test]
fn a_dump_scans_like_its_source() {
    use crate::dump::{DumpSource, write_dump};
    let src = SyntheticSource {
        records: 5_000,
        record_size: RS,
    };
    let path = std::env::temp_dir().join(format!("diskdome-test-{}.ddmft", std::process::id()));
    let mut file = std::fs::File::create(&path).expect("create");
    let bytes = write_dump(&src, &mut file).expect("dump");
    drop(file);
    assert_eq!(bytes, 5_000 * RS as u64);
    let dump = DumpSource::open(path.clone()).expect("open");
    let (a, _) = scan(&src, &small_options()).expect("scan");
    let (b, _) = scan(&dump, &small_options()).expect("scan dump");
    let _ = std::fs::remove_file(&path);
    assert_eq!(a.size, b.size);
    assert_eq!(a.names, b.names);
}
