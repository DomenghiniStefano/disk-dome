//! Parsing of one MFT file record: no I/O, no allocation.
//!
//! Layout references: the NTFS documentation of the Linux-NTFS project and `ntfs-3g`'s `layout.h`.
//! Only what DiskDome needs is read: in-use and folder flags, the base record of an extension
//! record, the last-modified time, the names with their parent folders, and the sizes of the unnamed
//! `$DATA` stream.

/// Attribute types.
const AT_STANDARD_INFORMATION: u32 = 0x10;
const AT_FILE_NAME: u32 = 0x30;
const AT_DATA: u32 = 0x80;
const AT_END: u32 = 0xFFFF_FFFF;

/// Record header flags.
const FLAG_IN_USE: u16 = 0x0001;
const FLAG_DIRECTORY: u16 = 0x0002;

/// Attribute flags (non-resident `$DATA`).
const ATTR_COMPRESSED: u16 = 0x0001;
const ATTR_SPARSE: u16 = 0x8000;

/// `$FILE_NAME` namespaces.
pub const NS_POSIX: u8 = 0;
pub const NS_WIN32: u8 = 1;
pub const NS_DOS: u8 = 2;
pub const NS_WIN32_AND_DOS: u8 = 3;

/// Names kept per record; a record holding more hard links than this loses the extra ones.
pub const MAX_NAMES: usize = 8;

/// The stride of the update sequence array: always 512 bytes in NTFS, whatever the sector size.
const USA_STRIDE: usize = 512;

/// Low 48 bits of a file reference: the record number (the high 16 bits are the sequence number).
pub fn record_number(file_reference: u64) -> u64 {
    file_reference & 0x0000_FFFF_FFFF_FFFF
}

/// One `$FILE_NAME` attribute: the name stays in the record buffer as UTF-16LE.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct NameRef {
    /// Record number of the parent folder.
    pub parent: u64,
    /// Byte offset of the UTF-16LE name inside the record.
    pub offset: u16,
    /// Length in UTF-16 code units.
    pub units: u8,
    pub namespace: u8,
}

/// What one record says.
#[derive(Debug, Clone, Copy, Default)]
pub struct Record {
    pub in_use: bool,
    pub is_dir: bool,
    /// Record number of the base record, or 0 if this is a base record.
    pub base: u64,
    /// Last modification from `$STANDARD_INFORMATION`, Unix seconds (0 when absent).
    pub mtime: u32,
    /// Unnamed `$DATA` stream: (logical size, size on disk). `None` when the record holds no first
    /// segment of it (folders, or an extension record carrying a later segment).
    pub data: Option<(u64, u64)>,
    /// Names, at most one per parent folder: the best namespace wins (Win32 over DOS).
    pub names: [NameRef; MAX_NAMES],
    pub name_count: u8,
}

impl Record {
    pub fn names(&self) -> &[NameRef] {
        &self.names[..self.name_count as usize]
    }
}

fn u16_at(buf: &[u8], off: usize) -> Option<u16> {
    Some(u16::from_le_bytes(buf.get(off..off + 2)?.try_into().ok()?))
}

fn u32_at(buf: &[u8], off: usize) -> Option<u32> {
    Some(u32::from_le_bytes(buf.get(off..off + 4)?.try_into().ok()?))
}

fn u64_at(buf: &[u8], off: usize) -> Option<u64> {
    Some(u64::from_le_bytes(buf.get(off..off + 8)?.try_into().ok()?))
}

/// Applies the update sequence array in place. Returns `false` if the record is not a valid `FILE`
/// record (free record never written, torn write, `BAAD` record).
pub fn apply_fixup(rec: &mut [u8]) -> bool {
    if rec.len() < 0x30 || &rec[0..4] != b"FILE" {
        return false;
    }
    let (Some(usa_off), Some(usa_count)) = (u16_at(rec, 4), u16_at(rec, 6)) else {
        return false;
    };
    let (usa_off, usa_count) = (usa_off as usize, usa_count as usize);
    let sectors = rec.len() / USA_STRIDE;
    if usa_count == 0 || usa_count - 1 > sectors || usa_off + usa_count * 2 > rec.len() {
        return false;
    }
    let usn = [rec[usa_off], rec[usa_off + 1]];
    for i in 1..usa_count {
        let end = i * USA_STRIDE - 2;
        if rec[end..end + 2] != usn {
            return false;
        }
        let fix = usa_off + i * 2;
        rec[end] = rec[fix];
        rec[end + 1] = rec[fix + 1];
    }
    true
}

/// FILETIME (100 ns since 1601) to Unix seconds, clamped to `u32`.
fn filetime_to_unix(ft: u64) -> u32 {
    let secs = (ft / 10_000_000).saturating_sub(11_644_473_600);
    u32::try_from(secs).unwrap_or(u32::MAX)
}

/// Parses a record whose fixup was already applied. Returns `None` for a record that is not in use
/// or is malformed.
pub fn parse(rec: &[u8]) -> Option<Record> {
    let flags = u16_at(rec, 0x16)?;
    if flags & FLAG_IN_USE == 0 {
        return None;
    }
    let mut out = Record {
        in_use: true,
        is_dir: flags & FLAG_DIRECTORY != 0,
        base: record_number(u64_at(rec, 0x20)?),
        ..Record::default()
    };
    let used = (u32_at(rec, 0x18)? as usize).min(rec.len());
    let mut pos = u16_at(rec, 0x14)? as usize;

    while pos + 16 <= used {
        let kind = u32_at(rec, pos)?;
        if kind == AT_END {
            break;
        }
        let len = u32_at(rec, pos + 4)? as usize;
        if len < 16 || pos + len > used {
            break;
        }
        let attr = &rec[pos..pos + len];
        let non_resident = attr[8] != 0;
        let name_len = attr[9];
        match kind {
            AT_STANDARD_INFORMATION if !non_resident => {
                let voff = u16_at(attr, 0x14)? as usize;
                if let Some(ft) = u64_at(attr, voff + 8) {
                    out.mtime = filetime_to_unix(ft);
                }
            }
            AT_FILE_NAME if !non_resident => {
                let voff = u16_at(attr, 0x14)? as usize;
                let parent = record_number(u64_at(attr, voff)?);
                let units = *attr.get(voff + 0x40)?;
                let namespace = *attr.get(voff + 0x41)?;
                let name_start = voff + 0x42;
                if name_start + units as usize * 2 > attr.len() {
                    break;
                }
                add_name(
                    &mut out,
                    NameRef {
                        parent,
                        offset: u16::try_from(pos + name_start).ok()?,
                        units,
                        namespace,
                    },
                );
            }
            AT_DATA if name_len == 0 => {
                if non_resident {
                    // Sizes are valid only in the segment that starts at VCN 0.
                    if u64_at(attr, 0x10)? == 0 {
                        let attr_flags = u16_at(attr, 0x0C)?;
                        let alloc = u64_at(attr, 0x28)?;
                        let size = u64_at(attr, 0x30)?;
                        let on_disk = if attr_flags & (ATTR_COMPRESSED | ATTR_SPARSE) != 0 {
                            u64_at(attr, 0x40).unwrap_or(alloc)
                        } else {
                            alloc
                        };
                        out.data = Some((size, on_disk));
                    }
                } else {
                    // Resident: the content lives inside the record and takes no cluster.
                    out.data = Some((u64::from(u32_at(attr, 0x10)?), 0));
                }
            }
            _ => {}
        }
        pos += len;
    }
    Some(out)
}

/// Keeps one name per parent folder, the best namespace first: a DOS 8.3 alias never replaces a
/// long name.
fn add_name(out: &mut Record, name: NameRef) {
    let count = out.name_count as usize;
    if let Some(existing) = out.names[..count]
        .iter_mut()
        .find(|n| n.parent == name.parent)
    {
        if existing.namespace == NS_DOS && name.namespace != NS_DOS {
            *existing = name;
        }
        return;
    }
    if count < MAX_NAMES {
        out.names[count] = name;
        out.name_count += 1;
    }
}

/// Decodes the UTF-16LE name of `name` from `rec` and appends it as UTF-8 to `out`. Unpaired
/// surrogates (possible on NTFS) become U+FFFD. Returns the number of bytes appended.
pub fn push_name_utf8(rec: &[u8], name: &NameRef, out: &mut Vec<u8>) -> usize {
    let start = name.offset as usize;
    let end = start + name.units as usize * 2;
    let Some(bytes) = rec.get(start..end) else {
        return 0;
    };
    let before = out.len();
    // Fast path: plain ASCII, the common case.
    let (pairs, _) = bytes.as_chunks::<2>();
    if pairs.iter().all(|c| c[1] == 0 && c[0] < 0x80) {
        out.extend(pairs.iter().map(|c| c[0]));
        return out.len() - before;
    }
    let units = pairs.iter().map(|c| u16::from_le_bytes(*c));
    let mut tmp = [0u8; 4];
    for ch in char::decode_utf16(units) {
        let ch = ch.unwrap_or(char::REPLACEMENT_CHARACTER);
        out.extend_from_slice(ch.encode_utf8(&mut tmp).as_bytes());
    }
    out.len() - before
}

/// One data run: `(vcn, lcn, clusters)`, `lcn = None` for a sparse run.
pub type Run = (u64, Option<u64>, u64);

/// Data runs of a non-resident attribute.
pub fn decode_runs(runs: &[u8], first_vcn: u64) -> Option<Vec<Run>> {
    let mut out = Vec::new();
    let mut pos = 0;
    let mut vcn = first_vcn;
    let mut lcn: i64 = 0;
    while let Some(&header) = runs.get(pos) {
        if header == 0 {
            break;
        }
        let len_bytes = (header & 0x0F) as usize;
        let off_bytes = (header >> 4) as usize;
        pos += 1;
        if len_bytes == 0 || len_bytes > 8 || off_bytes > 8 {
            return None;
        }
        let mut length: u64 = 0;
        for (k, b) in runs.get(pos..pos + len_bytes)?.iter().enumerate() {
            length |= u64::from(*b) << (8 * k);
        }
        pos += len_bytes;
        let run_lcn = if off_bytes == 0 {
            None
        } else {
            let raw = runs.get(pos..pos + off_bytes)?;
            let mut delta: i64 = 0;
            for (k, b) in raw.iter().enumerate() {
                delta |= i64::from(*b) << (8 * k);
            }
            // Sign-extend.
            let shift = 64 - 8 * off_bytes as u32;
            delta = (delta << shift) >> shift;
            pos += off_bytes;
            lcn += delta;
            Some(u64::try_from(lcn).ok()?)
        };
        out.push((vcn, run_lcn, length));
        vcn += length;
    }
    Some(out)
}

/// The data runs of the unnamed `$DATA` attribute of a record (fixup applied), with a flag telling
/// whether the record also has an `$ATTRIBUTE_LIST` (then the runs may continue in other records).
pub fn data_runs(rec: &[u8]) -> Option<(Vec<Run>, bool)> {
    const AT_ATTRIBUTE_LIST: u32 = 0x20;
    let used = (u32_at(rec, 0x18)? as usize).min(rec.len());
    let mut pos = u16_at(rec, 0x14)? as usize;
    let mut has_list = false;
    let mut runs = None;
    while pos + 16 <= used {
        let kind = u32_at(rec, pos)?;
        if kind == AT_END {
            break;
        }
        let len = u32_at(rec, pos + 4)? as usize;
        if len < 16 || pos + len > used {
            break;
        }
        let attr = &rec[pos..pos + len];
        if kind == AT_ATTRIBUTE_LIST {
            has_list = true;
        }
        if kind == AT_DATA && attr[9] == 0 && attr[8] != 0 && runs.is_none() {
            let first_vcn = u64_at(attr, 0x10)?;
            let runs_off = u16_at(attr, 0x20)? as usize;
            runs = decode_runs(attr.get(runs_off..)?, first_vcn);
        }
        pos += len;
    }
    Some((runs?, has_list))
}

#[cfg(test)]
mod tests {
    use super::{NS_DOS, NS_WIN32, apply_fixup, decode_runs, parse, push_name_utf8};
    use crate::synthetic::{RecordSpec, write_record};

    #[test]
    fn parses_a_file_record() {
        let mut rec = vec![0u8; 1024];
        write_record(
            &mut rec,
            7,
            &RecordSpec {
                names: &[
                    (5, "Hello wörld.txt", NS_WIN32),
                    (5, "HELLOW~1.TXT", NS_DOS),
                ],
                size: Some((5000, 8192)),
                mtime_unix: 1_700_000_000,
                ..RecordSpec::default()
            },
        );
        assert!(apply_fixup(&mut rec));
        let r = parse(&rec).expect("in use");
        assert!(!r.is_dir);
        assert_eq!(r.base, 0);
        assert_eq!(r.mtime, 1_700_000_000);
        assert_eq!(r.data, Some((5000, 8192)));
        assert_eq!(
            r.names().len(),
            1,
            "the DOS alias of the same parent is dropped"
        );
        let mut out = Vec::new();
        push_name_utf8(&rec, &r.names()[0], &mut out);
        assert_eq!(std::str::from_utf8(&out), Ok("Hello wörld.txt"));
    }

    #[test]
    fn a_torn_record_is_rejected() {
        let mut rec = vec![0u8; 1024];
        write_record(&mut rec, 7, &RecordSpec::default());
        rec[510] ^= 0xFF;
        assert!(!apply_fixup(&mut rec));
    }

    #[test]
    fn hard_links_in_two_folders_are_two_names() {
        let mut rec = vec![0u8; 1024];
        write_record(
            &mut rec,
            9,
            &RecordSpec {
                names: &[(5, "a", NS_WIN32), (30, "b", NS_WIN32)],
                size: Some((1, 0)),
                ..RecordSpec::default()
            },
        );
        assert!(apply_fixup(&mut rec));
        let r = parse(&rec).expect("in use");
        assert_eq!(r.names().len(), 2);
        assert_eq!(r.names()[1].parent, 30);
    }

    #[test]
    fn runs_decode_with_negative_offsets() {
        // 0x21: 1 length byte, 2 offset bytes. Run 1: 0x10 clusters at LCN 0x1000.
        // Run 2: 0x08 clusters at LCN 0x1000 - 0x100. Then a sparse run of 4.
        let runs = [
            0x21, 0x10, 0x00, 0x10, 0x21, 0x08, 0x00, 0xFF, 0x01, 0x04, 0x00,
        ];
        let decoded = decode_runs(&runs, 0).expect("valid");
        assert_eq!(
            decoded,
            vec![
                (0, Some(0x1000), 0x10),
                (0x10, Some(0x0F00), 0x08),
                (0x18, None, 0x04)
            ]
        );
    }
}
