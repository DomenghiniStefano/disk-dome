//! `$MFT` images built in memory: a record writer for tests, and a generator of a drive-sized tree
//! for benchmarks of the parsing and model side without administrator rights.

use crate::pipeline::{ChunkReader, ScanError, Source};
use crate::record::NS_WIN32;

/// What [`write_record`] puts in a record.
#[derive(Debug, Clone, Copy)]
pub struct RecordSpec<'a> {
    pub in_use: bool,
    pub dir: bool,
    /// Base record number for an extension record, 0 for a base record.
    pub base: u64,
    /// `(parent record, name, namespace)`.
    pub names: &'a [(u64, &'a str, u8)],
    /// Unnamed `$DATA`: `(size, allocated)`; stored resident when `allocated` is 0.
    pub size: Option<(u64, u64)>,
    pub mtime_unix: u32,
}

impl Default for RecordSpec<'_> {
    fn default() -> Self {
        Self {
            in_use: true,
            dir: false,
            base: 0,
            names: &[],
            size: None,
            mtime_unix: 0,
        }
    }
}

fn put16(b: &mut [u8], off: usize, v: u16) {
    b[off..off + 2].copy_from_slice(&v.to_le_bytes());
}
fn put32(b: &mut [u8], off: usize, v: u32) {
    b[off..off + 4].copy_from_slice(&v.to_le_bytes());
}
fn put64(b: &mut [u8], off: usize, v: u64) {
    b[off..off + 8].copy_from_slice(&v.to_le_bytes());
}

/// Writes resident attribute header + value of `value_len` bytes at `pos`; returns (value offset,
/// next attribute position).
fn resident(b: &mut [u8], pos: usize, kind: u32, value_len: usize) -> (usize, usize) {
    let len = (0x18 + value_len).div_ceil(8) * 8;
    put32(b, pos, kind);
    put32(b, pos + 4, len as u32);
    b[pos + 8] = 0;
    put32(b, pos + 0x10, value_len as u32);
    put16(b, pos + 0x14, 0x18);
    (pos + 0x18, pos + len)
}

/// Writes one file record of `buf.len()` bytes (a multiple of 512) for record number `number`, with
/// its update sequence array applied, as NTFS stores it on disk.
pub fn write_record(buf: &mut [u8], number: u64, spec: &RecordSpec<'_>) {
    buf.fill(0);
    let sectors = buf.len() / 512;
    let usa_count = sectors + 1;
    let attrs = (0x30 + usa_count * 2).div_ceil(8) * 8;
    buf[0..4].copy_from_slice(b"FILE");
    put16(buf, 4, 0x30);
    put16(buf, 6, usa_count as u16);
    put16(buf, 0x10, 1);
    put16(buf, 0x12, spec.names.len() as u16);
    put16(buf, 0x14, attrs as u16);
    let mut flags = 0u16;
    if spec.in_use {
        flags |= 1;
    }
    if spec.dir {
        flags |= 2;
    }
    put16(buf, 0x16, flags);
    put32(buf, 0x1C, buf.len() as u32);
    put64(buf, 0x20, spec.base);
    put32(buf, 0x2C, number as u32);

    let ft = (u64::from(spec.mtime_unix) + 11_644_473_600) * 10_000_000;
    let mut pos = attrs;
    if spec.base == 0 {
        let (v, next) = resident(buf, pos, 0x10, 0x48);
        for k in 0..4 {
            put64(buf, v + k * 8, ft);
        }
        pos = next;
    }
    for &(parent, name, namespace) in spec.names {
        let units: Vec<u16> = name.encode_utf16().collect();
        let (v, next) = resident(buf, pos, 0x30, 0x42 + units.len() * 2);
        put64(buf, v, parent | (1 << 48));
        buf[v + 0x40] = units.len() as u8;
        buf[v + 0x41] = namespace;
        for (k, u) in units.iter().enumerate() {
            put16(buf, v + 0x42 + k * 2, *u);
        }
        pos = next;
    }
    if let Some((size, alloc)) = spec.size {
        if alloc == 0 {
            let (_, next) = resident(buf, pos, 0x80, size as usize);
            pos = next;
        } else {
            let len = 0x48 + 8;
            put32(buf, pos, 0x80);
            put32(buf, pos + 4, len as u32);
            buf[pos + 8] = 1;
            let clusters = alloc.div_ceil(4096);
            put64(buf, pos + 0x18, clusters.saturating_sub(1));
            put16(buf, pos + 0x20, 0x48);
            put64(buf, pos + 0x28, alloc);
            put64(buf, pos + 0x30, size);
            put64(buf, pos + 0x38, size);
            put64(buf, pos + 0x40, alloc);
            // An empty run list: enough for the parser, which reads only the sizes.
            pos += len;
        }
    }
    put32(buf, pos, 0xFFFF_FFFF);
    put32(buf, 0x18, (pos + 8) as u32);

    // Update sequence array: the last two bytes of every sector move into the array.
    let usn: u16 = 0x0007;
    put16(buf, 0x30, usn);
    for i in 1..usa_count {
        let end = i * 512 - 2;
        let (a, b) = (buf[end], buf[end + 1]);
        buf[0x30 + i * 2] = a;
        buf[0x30 + i * 2 + 1] = b;
        put16(buf, end, usn);
    }
}

/// An `$MFT` image held in memory.
pub struct MemorySource {
    pub data: Vec<u8>,
    pub record_size: usize,
}

pub struct MemoryReader<'a>(&'a [u8]);

impl ChunkReader for MemoryReader<'_> {
    fn read_at(&mut self, offset: u64, buf: &mut [u8]) -> Result<(), ScanError> {
        buf.fill(0);
        let start = (offset as usize).min(self.0.len());
        let end = (start + buf.len()).min(self.0.len());
        buf[..end - start].copy_from_slice(&self.0[start..end]);
        Ok(())
    }
}

impl Source for MemorySource {
    type Reader<'a> = MemoryReader<'a>;
    fn open_reader(&self) -> Result<Self::Reader<'_>, ScanError> {
        Ok(MemoryReader(&self.data))
    }
    fn len(&self) -> u64 {
        self.data.len() as u64
    }
    fn record_size(&self) -> usize {
        self.record_size
    }
    fn alignment(&self) -> usize {
        self.record_size
    }
}

/// A generated drive: `records` MFT records shaped like a real system drive (one folder every 8
/// records, ~3% free records, names of 6–30 characters with some non-ASCII, sizes spread over
/// several orders of magnitude, small files resident). Deterministic.
pub struct SyntheticSource {
    pub records: u64,
    pub record_size: usize,
}

pub struct SyntheticReader {
    record_size: usize,
    records: u64,
    name: String,
}

fn mix(mut x: u64) -> u64 {
    // splitmix64
    x = x.wrapping_add(0x9E37_79B9_7F4A_7C15);
    x = (x ^ (x >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    x ^ (x >> 31)
}

impl SyntheticSource {
    /// Is record `r` a folder, and who is its parent?
    pub fn shape(r: u64) -> (bool, u64) {
        if r < 24 {
            return (r == 5, 5);
        }
        let dir = r.is_multiple_of(8);
        let dirs_before = r / 8;
        let parent = if dirs_before <= 3 {
            5
        } else {
            // Favour recent folders a little, so the tree gets some depth.
            let h = mix(r);
            let span = dirs_before - 3;
            let pick = if h & 1 == 0 {
                span - 1 - (h >> 8) % span.min(4096)
            } else {
                (h >> 8) % span
            };
            (pick + 3) * 8
        };
        (dir, parent)
    }

    /// Is record `r` free, and its size if it is a file?
    pub fn content(r: u64) -> (bool, u64, u64) {
        let h = mix(r ^ 0xABCD);
        let (dir, _) = Self::shape(r);
        let free = !dir && r >= 24 && h.is_multiple_of(33);
        let magnitude = (h >> 8) % 28; // up to 2^28 = 256 MB
        let size = if dir {
            0
        } else {
            (h >> 16) % (1u64 << magnitude)
        };
        let alloc = if size <= 600 {
            0
        } else {
            size.div_ceil(4096) * 4096
        };
        (free, size, alloc)
    }
}

impl ChunkReader for SyntheticReader {
    fn read_at(&mut self, offset: u64, buf: &mut [u8]) -> Result<(), ScanError> {
        let rs = self.record_size;
        let first = offset / rs as u64;
        for (k, rec) in buf.chunks_exact_mut(rs).enumerate() {
            let r = first + k as u64;
            if r >= self.records {
                rec.fill(0);
                continue;
            }
            let (dir, parent) = SyntheticSource::shape(r);
            let (free, size, alloc) = SyntheticSource::content(r);
            let h = mix(r ^ 0x5555);
            self.name.clear();
            if r == 5 {
                self.name.push('.');
            } else {
                let len = 6 + (h % 25) as usize;
                for i in 0..len {
                    let c = (mix(h + i as u64) % 36) as u8;
                    self.name.push(if c < 10 {
                        (b'0' + c) as char
                    } else {
                        (b'a' + c - 10) as char
                    });
                }
                if h.is_multiple_of(50) {
                    self.name.push_str("_è");
                }
                if !dir {
                    self.name.push_str(".dat");
                }
            }
            let names = [(parent, self.name.as_str(), NS_WIN32)];
            write_record(
                rec,
                r,
                &RecordSpec {
                    in_use: !free,
                    dir,
                    base: 0,
                    names: &names,
                    size: (!dir).then_some((size, alloc)),
                    mtime_unix: 1_600_000_000 + (h % 100_000_000) as u32,
                },
            );
        }
        Ok(())
    }
}

impl Source for SyntheticSource {
    type Reader<'a> = SyntheticReader;
    fn open_reader(&self) -> Result<Self::Reader<'_>, ScanError> {
        Ok(SyntheticReader {
            record_size: self.record_size,
            records: self.records,
            name: String::with_capacity(64),
        })
    }
    fn len(&self) -> u64 {
        self.records * self.record_size as u64
    }
    fn record_size(&self) -> usize {
        self.record_size
    }
    fn alignment(&self) -> usize {
        self.record_size
    }
}
