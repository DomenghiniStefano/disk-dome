//! Compares folder totals exported by DiskDome (`scan --export`) with WizTree's CSV export
//! (`WizTree64.exe C: /export="wiztree.csv" /admin=1 /exportfiles=0`).
//!
//! Paths are compared case-insensitively; a folder is matched when both its size and its
//! allocated size are equal.

use std::collections::HashMap;
use std::io::{BufRead, BufReader};

use crate::export::SEP;

/// One CSV line into fields: quotes and doubled quotes handled.
fn fields(line: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut quoted = false;
    let mut chars = line.chars().peekable();
    while let Some(c) = chars.next() {
        match (c, quoted) {
            ('"', true) if chars.peek() == Some(&'"') => {
                cur.push('"');
                chars.next();
            }
            ('"', _) => quoted = !quoted,
            (',', false) => out.push(std::mem::take(&mut cur)),
            (c, _) => cur.push(c),
        }
    }
    out.push(cur);
    out
}

/// Folder path (lowercase, ending in `\`) → (size, allocated). Files and other lines are skipped.
fn load(path: &str) -> Result<HashMap<String, (u64, u64)>, String> {
    let file = std::fs::File::open(path).map_err(|e| format!("{path}: {e}"))?;
    let mut map = HashMap::new();
    for line in BufReader::with_capacity(1 << 20, file).lines() {
        let line = line.map_err(|e| format!("{path}: {e}"))?;
        let line = line.trim_start_matches('\u{feff}');
        let f = fields(line);
        let (Some(name), Some(size), Some(alloc)) = (f.first(), f.get(1), f.get(2)) else {
            continue;
        };
        if !name.ends_with(SEP) {
            continue;
        }
        let (Ok(size), Ok(alloc)) = (size.trim().parse::<u64>(), alloc.trim().parse::<u64>())
        else {
            continue;
        };
        map.insert(name.to_lowercase(), (size, alloc));
    }
    Ok(map)
}

/// (size difference, path, ours, WizTree's)
type Diff<'a> = (i128, &'a String, (u64, u64), (u64, u64));

pub fn run(paths: &[String], top: usize) -> Result<(), String> {
    let (Some(ours), Some(theirs)) = (paths.first(), paths.get(1)) else {
        return Err("compare needs two CSV files: ours, then WizTree's".into());
    };
    let ours = load(ours)?;
    let theirs = load(theirs)?;
    let mut equal = 0u64;
    let mut size_diff: Vec<Diff<'_>> = Vec::new();
    let mut only_ours = Vec::new();
    for (path, &a) in &ours {
        match theirs.get(path) {
            Some(&b) if a == b => equal += 1,
            Some(&b) => size_diff.push((i128::from(a.0) - i128::from(b.0), path, a, b)),
            None => only_ours.push(path),
        }
    }
    let only_theirs: Vec<&String> = theirs.keys().filter(|p| !ours.contains_key(*p)).collect();
    println!(
        "folders: ours {}, WizTree {}; equal {equal}, different {}, only ours {}, only WizTree {}",
        ours.len(),
        theirs.len(),
        size_diff.len(),
        only_ours.len(),
        only_theirs.len()
    );
    let same_size = size_diff.iter().filter(|d| d.0 == 0).count();
    println!("  of the different ones, same size but different allocated: {same_size}");
    // Shallowest first, then the largest gap: the root cause is usually near the top.
    size_diff.sort_by_key(|d| (d.1.matches(SEP).count(), std::cmp::Reverse(d.0.abs())));
    for (delta, path, a, b) in size_diff.iter().take(top) {
        println!(
            "  {path}  size {} vs {} ({delta:+}), allocated {} vs {}",
            a.0, b.0, a.1, b.1
        );
    }
    only_ours.sort_by_key(|p| p.matches(SEP).count());
    for p in only_ours.iter().take(top) {
        println!("  only ours: {p}");
    }
    let mut only_theirs = only_theirs;
    only_theirs.sort_by_key(|p| p.matches(SEP).count());
    for p in only_theirs.iter().take(top) {
        println!("  only WizTree: {p}");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::fields;

    #[test]
    fn csv_fields() {
        assert_eq!(
            fields(r#""C:\a ""b"", c\",10,20"#),
            vec![r#"C:\a "b", c\"#, "10", "20"]
        );
    }
}
