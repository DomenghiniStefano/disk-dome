//! Folder totals as CSV, in the shape of WizTree's export (`"C:\path\",size,allocated`), so the two
//! can be compared line by line.

use std::io::Write;
use std::path::Path;

use diskdome_core::Tree;

/// The Windows path separator (a backslash).
pub const SEP: char = 92u8 as char;

/// Writes every folder reachable from the root with its totals. Returns the number of folders.
pub fn folders_csv(tree: &Tree, drive: char, path: &Path) -> std::io::Result<u64> {
    let file = std::fs::File::create(path)?;
    let mut out = std::io::BufWriter::with_capacity(1 << 20, file);
    writeln!(out, "File Name,Size,Allocated")?;
    // Depth-first with one growing path buffer: no path is stored, none is rebuilt from scratch.
    let mut buf = format!("{drive}:\\");
    let mut stack: Vec<(u32, usize)> = vec![(tree.root, buf.len())];
    let mut count = 0u64;
    while let Some((node, len)) = stack.pop() {
        buf.truncate(len);
        if node != tree.root {
            buf.push_str(tree.name(node));
            buf.push(SEP);
        }
        let i = node as usize;
        writeln!(
            out,
            "\"{}\",{},{}",
            buf.replace('"', "\"\""),
            tree.size[i],
            tree.alloc[i]
        )?;
        count += 1;
        let here = buf.len();
        for &child in tree.children(node).iter().rev() {
            if tree.is_dir(child) {
                stack.push((child, here));
            }
        }
    }
    out.flush()?;
    Ok(count)
}
