//! `diskdome-cli inspect --dump FILE --path "C:\some\folder"`: a folder of an `$MFT` dump with its
//! largest children and the raw attributes of their records. For finding why a total differs from
//! WizTree's.

use diskdome_core::{Tree, flag};
use diskdome_scan_ntfs::dump::DumpSource;
use diskdome_scan_ntfs::record::describe;

use crate::export::SEP;

fn kind_name(kind: u32) -> &'static str {
    match kind {
        0x10 => "STANDARD_INFORMATION",
        0x20 => "ATTRIBUTE_LIST",
        0x30 => "FILE_NAME",
        0x40 => "OBJECT_ID",
        0x50 => "SECURITY_DESCRIPTOR",
        0x60 => "VOLUME_NAME",
        0x70 => "VOLUME_INFORMATION",
        0x80 => "DATA",
        0x90 => "INDEX_ROOT",
        0xA0 => "INDEX_ALLOCATION",
        0xB0 => "BITMAP",
        0xC0 => "REPARSE_POINT",
        0xD0 => "EA_INFORMATION",
        0xE0 => "EA",
        0x100 => "LOGGED_UTILITY_STREAM",
        _ => "?",
    }
}

/// The entry at `path` (case-insensitive, `\`-separated, drive letter optional).
fn find(tree: &Tree, path: &str) -> Option<u32> {
    let mut node = tree.root;
    let rest = path.split_once(':').map_or(path, |(_, r)| r);
    for part in rest.split(SEP).filter(|p| !p.is_empty()) {
        node = tree
            .children(node)
            .iter()
            .copied()
            .find(|&c| tree.name(c).eq_ignore_ascii_case(part))?;
    }
    Some(node)
}

pub fn run(
    tree: &Tree,
    dump: &DumpSource,
    records: u64,
    path: &str,
    top: usize,
) -> Result<(), String> {
    let node = find(tree, path).ok_or_else(|| format!("not found: {path}"))?;
    println!(
        "{path}: size {}, allocated {}, {} children",
        tree.size[node as usize],
        tree.alloc[node as usize],
        tree.children(node).len()
    );
    let mut kids: Vec<u32> = tree.children(node).to_vec();
    kids.sort_by_key(|&c| std::cmp::Reverse(tree.size[c as usize]));
    for &c in kids.iter().take(top) {
        let i = c as usize;
        let mut marks = String::new();
        if tree.flags[i] & flag::DIR != 0 {
            marks.push_str(" dir");
        }
        if tree.flags[i] & flag::EXTRA_LINK != 0 {
            marks.push_str(" extra-link");
        }
        println!(
            "  {:<48} size {:>14} alloc {:>14}  record {c}{marks}",
            tree.name(c),
            tree.size[i],
            tree.alloc[i]
        );
        if u64::from(c) < records
            && tree.flags[i] & flag::DIR == 0
            && let Some(rec) = dump.record(u64::from(c))
        {
            for a in describe(&rec) {
                println!(
                    "      {:#05x} {:<22} {:<18} {} flags {:#06x} vcn {} size {} alloc {} compressed {}",
                    a.kind,
                    kind_name(a.kind),
                    if a.name.is_empty() {
                        "\"\"".to_string()
                    } else {
                        a.name.clone()
                    },
                    if a.non_resident {
                        "non-resident"
                    } else {
                        "resident"
                    },
                    a.flags,
                    a.first_vcn,
                    a.size,
                    a.alloc,
                    a.compressed
                );
            }
        }
    }
    Ok(())
}
