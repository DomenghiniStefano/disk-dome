//! The scan pipeline: the `$MFT` is cut into chunks, a pool of threads reads and parses them, each
//! writing straight into the slots of its own records (the record number is the slot index), then
//! the names are joined into one buffer, the records spread over several MFT records (attribute
//! lists, extra hard links) are merged, and the tree is linked and summed.
//!
//! Each thread reads and then parses its chunk: with more threads than cores, some are always
//! waiting on the drive while the others parse, so reading and parsing overlap without a queue.

use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use diskdome_core::{FinishStats, NONE, Tree, flag};

use crate::buf::AlignedBuf;
use crate::record::{NS_DOS, apply_fixup, parse, push_name_utf8};

/// Record number of the root folder (`.`) on every NTFS volume.
pub const ROOT_RECORD: u32 = 5;
/// Record number of `$BadClus`, whose `$Bad` stream is a sparse map as large as the volume: its
/// streams take no real space and are not counted.
const BADCLUS_RECORD: u64 = 8;

/// Scratch flags used only during the scan, cleared before `Tree::finish`.
const IN_USE_BASE: u8 = 1 << 6;
const DOS_NAME: u8 = 1 << 7;

/// Why a scan could not run. Codes and values only: the interface turns them into sentences.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScanError {
    /// Opening the volume was refused: administrator rights are needed.
    NotAdmin,
    /// The volume is not NTFS.
    NotNtfs,
    /// An OS call failed: which one and its error code.
    Io { op: &'static str, code: i32 },
    /// The `$MFT` layout is not what was expected.
    BadMft(&'static str),
    /// Memory for the buffers could not be allocated.
    OutOfMemory,
    /// Reading a volume's `$MFT` is not available on this platform.
    Unsupported,
}

impl std::fmt::Display for ScanError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScanError::NotAdmin => write!(f, "administrator rights are needed to read the $MFT"),
            ScanError::NotNtfs => write!(f, "the volume is not NTFS"),
            ScanError::Io { op, code } => write!(f, "{op} failed (OS error {code})"),
            ScanError::BadMft(what) => write!(f, "unexpected $MFT layout: {what}"),
            ScanError::OutOfMemory => write!(f, "out of memory"),
            ScanError::Unsupported => write!(f, "reading the $MFT needs Windows"),
        }
    }
}

impl std::error::Error for ScanError {}

/// Where the `$MFT` bytes come from: a volume, a dump file, memory or a generator.
pub trait Source: Sync {
    type Reader<'a>: ChunkReader + Send
    where
        Self: 'a;
    /// A reader for one thread (a volume reader opens its own handle: synchronous reads on a shared
    /// handle are serialised by Windows).
    fn open_reader(&self) -> Result<Self::Reader<'_>, ScanError>;
    /// Bytes of `$MFT` holding records (its valid data length).
    fn len(&self) -> u64;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
    /// Bytes per file record (1024 or 4096).
    fn record_size(&self) -> usize;
    /// Offsets and lengths of reads must be multiples of this (the cluster size for a volume).
    fn alignment(&self) -> usize;
}

pub trait ChunkReader {
    /// Fills `buf` with the `$MFT` bytes starting at `offset` (bytes past the end read as zero).
    fn read_at(&mut self, offset: u64, buf: &mut [u8]) -> Result<(), ScanError>;
}

#[derive(Debug, Clone, Copy)]
pub struct Options {
    pub threads: usize,
    pub chunk_bytes: usize,
}

impl Default for Options {
    fn default() -> Self {
        let cores = std::thread::available_parallelism().map_or(4, |n| n.get());
        Self {
            threads: (cores * 2).clamp(4, 32),
            chunk_bytes: 4 << 20,
        }
    }
}

/// What a read-only pass measured.
#[derive(Debug, Clone, Copy)]
pub struct ReadReport {
    pub bytes: u64,
    pub wall: Duration,
    pub threads: usize,
    pub chunk_bytes: usize,
}

/// What a scan measured.
#[derive(Debug, Clone, Copy, Default)]
pub struct ScanReport {
    pub mft_bytes: u64,
    pub records: u64,
    pub threads: usize,
    pub chunk_bytes: usize,
    /// Reading and parsing, overlapped.
    pub read_parse: Duration,
    /// Joining the names and merging extension records and extra hard links.
    pub merge: Duration,
    /// Linking children and summing folder totals.
    pub finish: Duration,
    pub extension_records: u64,
    pub extra_links: u64,
    pub tree: FinishStats,
}

fn chunk_len(source: &impl Source, opts: &Options) -> usize {
    let unit = source.record_size().max(source.alignment()).max(1);
    (opts.chunk_bytes / unit).max(1) * unit
}

/// Runs `work(chunk_index, offset, valid_bytes, reader, buffer)` for every chunk on `opts.threads`
/// threads, stopping at the first error.
fn for_each_chunk<S, F>(source: &S, opts: &Options, chunk: usize, work: F) -> Result<(), ScanError>
where
    S: Source,
    F: Fn(usize, u64, usize, &mut [u8]) -> Result<(), ScanError> + Sync,
{
    let total = source.len();
    let chunks = total.div_ceil(chunk as u64) as usize;
    let next = std::sync::atomic::AtomicUsize::new(0);
    let failed = AtomicBool::new(false);
    let error: Mutex<Option<ScanError>> = Mutex::new(None);
    let align = source.alignment().max(1) as u64;

    std::thread::scope(|scope| {
        for _ in 0..opts.threads.max(1) {
            scope.spawn(|| {
                let run = || -> Result<(), ScanError> {
                    let mut reader = source.open_reader()?;
                    let mut buf = AlignedBuf::new(chunk).ok_or(ScanError::OutOfMemory)?;
                    let buf = buf.as_mut_slice();
                    loop {
                        if failed.load(Ordering::Relaxed) {
                            return Ok(());
                        }
                        let index = next.fetch_add(1, Ordering::Relaxed);
                        if index >= chunks {
                            return Ok(());
                        }
                        let offset = index as u64 * chunk as u64;
                        let valid = (total - offset).min(chunk as u64);
                        let read_len = valid.div_ceil(align) * align;
                        let read_len = (read_len as usize).min(buf.len());
                        reader.read_at(offset, &mut buf[..read_len])?;
                        work(index, offset, valid as usize, &mut buf[..read_len])?;
                    }
                };
                if let Err(e) = run() {
                    failed.store(true, Ordering::Relaxed);
                    if let Ok(mut slot) = error.lock() {
                        slot.get_or_insert(e);
                    }
                }
            });
        }
    });
    match error.into_inner() {
        Ok(Some(e)) => Err(e),
        Ok(None) => Ok(()),
        Err(_) => Err(ScanError::BadMft("a worker thread panicked")),
    }
}

/// Reads the whole `$MFT` without parsing it: the drive's side of the budget.
pub fn read_only(source: &impl Source, opts: &Options) -> Result<ReadReport, ScanError> {
    let chunk = chunk_len(source, opts);
    let start = Instant::now();
    for_each_chunk(source, opts, chunk, |_, _, _, buf| {
        // Touch the data so nothing can be optimised away.
        std::hint::black_box(buf.first());
        Ok(())
    })?;
    Ok(ReadReport {
        bytes: source.len(),
        wall: start.elapsed(),
        threads: opts.threads,
        chunk_bytes: chunk,
    })
}

/// A name or hard link found outside its record's own slot.
#[derive(Clone, Copy)]
enum Extra {
    /// A name of record `rec` (from an extension record, or a second parent folder).
    Link {
        rec: u32,
        parent: u32,
        name_off: u32,
        name_len: u16,
        namespace: u8,
    },
    /// Sizes of record `rec` found in one of its extension records: the unnamed stream's size if
    /// it is there, and the size on disk of the streams there (added to the record's).
    Data {
        rec: u32,
        size: Option<u64>,
        alloc: u64,
    },
}

/// One chunk's output besides its slots.
#[derive(Default)]
struct ChunkOut {
    names: Vec<u8>,
    extras: Vec<Extra>,
    extension_records: u64,
}

/// The slots of one chunk's records.
struct Slots<'a> {
    name_off: &'a mut [u32],
    name_len: &'a mut [u16],
    parent: &'a mut [u32],
    size: &'a mut [u64],
    alloc: &'a mut [u64],
    mtime: &'a mut [u32],
    flags: &'a mut [u8],
}

fn to_u32(n: u64) -> u32 {
    u32::try_from(n).unwrap_or(NONE)
}

fn parse_chunk(buf: &mut [u8], first_record: u64, rs: usize, slots: &mut Slots<'_>) -> ChunkOut {
    let mut out = ChunkOut {
        // ~25 bytes of name per record.
        names: Vec::with_capacity(buf.len() / rs * 28),
        ..ChunkOut::default()
    };
    for (k, rec) in buf.chunks_exact_mut(rs).enumerate() {
        if k >= slots.flags.len() {
            break;
        }
        if !apply_fixup(rec) {
            continue;
        }
        let Some(p) = parse(rec) else {
            continue;
        };
        let rec: &[u8] = rec;
        if p.base != 0 {
            out.extension_records += 1;
            let base = to_u32(p.base);
            if p.data.is_some() || p.stream_alloc > 0 {
                out.extras.push(Extra::Data {
                    rec: base,
                    size: p.data.map(|d| d.0),
                    alloc: p.data.map_or(0, |d| d.1) + p.stream_alloc,
                });
            }
            for name in p.names() {
                let name_off = out.names.len() as u32;
                let len = push_name_utf8(rec, name, &mut out.names);
                out.extras.push(Extra::Link {
                    rec: base,
                    parent: to_u32(name.parent),
                    name_off,
                    name_len: len as u16,
                    namespace: name.namespace,
                });
            }
            continue;
        }
        slots.mtime[k] = p.mtime;
        let mut f = IN_USE_BASE;
        if p.is_dir {
            f |= flag::DIR;
        }
        if let Some((size, _)) = p.data {
            slots.size[k] = size;
        }
        let streams = if first_record + k as u64 == BADCLUS_RECORD {
            0
        } else {
            p.stream_alloc
        };
        slots.alloc[k] = p.data.map_or(0, |d| d.1) + streams;
        for (i, name) in p.names().iter().enumerate() {
            let name_off = out.names.len() as u32;
            let len = push_name_utf8(rec, name, &mut out.names) as u16;
            if i == 0 {
                slots.name_off[k] = name_off;
                slots.name_len[k] = len;
                slots.parent[k] = to_u32(name.parent);
                if name.namespace == NS_DOS {
                    f |= DOS_NAME;
                }
            } else {
                out.extras.push(Extra::Link {
                    rec: to_u32(first_record + k as u64),
                    parent: to_u32(name.parent),
                    name_off,
                    name_len: len,
                    namespace: name.namespace,
                });
            }
        }
        slots.flags[k] = f;
    }
    out
}

/// Scans the `$MFT` into a [`Tree`] whose entry `i` is MFT record `i` (extra hard links appended
/// after the last record), with folder totals summed.
pub fn scan(source: &impl Source, opts: &Options) -> Result<(Tree, ScanReport), ScanError> {
    let rs = source.record_size();
    if rs < 512 || source.alignment() == 0 {
        return Err(ScanError::BadMft("record size"));
    }
    let chunk = chunk_len(source, opts);
    let per_chunk = chunk / rs;
    let records = source.len() / rs as u64;
    let n = usize::try_from(records).map_err(|_| ScanError::BadMft("too many records"))?;
    if n >= NONE as usize {
        return Err(ScanError::BadMft("too many records"));
    }
    let mut report = ScanReport {
        mft_bytes: source.len(),
        records,
        threads: opts.threads,
        chunk_bytes: chunk,
        ..ScanReport::default()
    };

    let mut tree = Tree::with_slots(n, 0);
    let start = Instant::now();
    let outs: Vec<Mutex<Option<ChunkOut>>>;
    {
        // One `Slots` per chunk, each borrowing its own range of every array.
        let slots: Vec<Mutex<Option<Slots<'_>>>> = tree
            .name_off
            .chunks_mut(per_chunk)
            .zip(tree.name_len.chunks_mut(per_chunk))
            .zip(tree.parent.chunks_mut(per_chunk))
            .zip(tree.size.chunks_mut(per_chunk))
            .zip(tree.alloc.chunks_mut(per_chunk))
            .zip(tree.mtime.chunks_mut(per_chunk))
            .zip(tree.flags.chunks_mut(per_chunk))
            .map(
                |((((((name_off, name_len), parent), size), alloc), mtime), flags)| {
                    Mutex::new(Some(Slots {
                        name_off,
                        name_len,
                        parent,
                        size,
                        alloc,
                        mtime,
                        flags,
                    }))
                },
            )
            .collect();
        outs = (0..slots.len()).map(|_| Mutex::new(None)).collect();
        for_each_chunk(source, opts, chunk, |index, offset, valid, buf| {
            let mut slot = slots
                .get(index)
                .and_then(|m| m.lock().ok())
                .and_then(|mut g| g.take())
                .ok_or(ScanError::BadMft("chunk without slots"))?;
            let first = offset / rs as u64;
            let out = parse_chunk(&mut buf[..valid], first, rs, &mut slot);
            if let Some(m) = outs.get(index)
                && let Ok(mut g) = m.lock()
            {
                *g = Some(out);
            }
            Ok(())
        })?;
    }
    report.read_parse = start.elapsed();

    // Join the names into one buffer, chunk by chunk, freeing each chunk's buffer as it goes.
    let start = Instant::now();
    let mut outs: Vec<ChunkOut> = outs
        .into_iter()
        .map(|m| m.into_inner().ok().flatten().unwrap_or_default())
        .collect();
    let total_names: usize = outs.iter().map(|o| o.names.len()).sum();
    tree.names = Vec::with_capacity(total_names);
    let mut bases = Vec::with_capacity(outs.len());
    for (index, out) in outs.iter_mut().enumerate() {
        let base = tree.names.len() as u32;
        bases.push(base);
        tree.names.extend_from_slice(&out.names);
        out.names = Vec::new();
        let first = index * per_chunk;
        let last = (first + per_chunk).min(n);
        for off in &mut tree.name_off[first..last] {
            *off += base;
        }
        report.extension_records += out.extension_records;
    }
    // Sizes first, so every extra link below can copy its file's final size.
    for out in &outs {
        for extra in &out.extras {
            if let Extra::Data { rec, size, alloc } = *extra
                && let Some(a) = tree.alloc.get_mut(rec as usize)
            {
                *a += alloc;
                if let Some(size) = size {
                    tree.size[rec as usize] = size;
                }
            }
        }
    }
    for (out, base) in outs.iter().zip(bases) {
        for extra in &out.extras {
            match *extra {
                Extra::Data { .. } => {}
                Extra::Link {
                    rec,
                    parent,
                    name_off,
                    name_len,
                    namespace,
                } => {
                    let r = rec as usize;
                    let Some(&f) = tree.flags.get(r) else {
                        continue;
                    };
                    if f & IN_USE_BASE == 0 {
                        continue;
                    }
                    let name_off = name_off + base;
                    let better = f & DOS_NAME != 0 && namespace != NS_DOS;
                    if tree.name_len[r] == 0 || (tree.parent[r] == parent && better) {
                        tree.name_off[r] = name_off;
                        tree.name_len[r] = name_len;
                        tree.parent[r] = parent;
                        tree.flags[r] = if namespace == NS_DOS {
                            f | DOS_NAME
                        } else {
                            f & !DOS_NAME
                        };
                    } else if namespace != NS_DOS {
                        // Another hard link (in another folder, or in the same one under another
                        // name); a DOS alias found here belongs to a name already listed.
                        // Folders cannot be hard-linked: an extra link is always a file. As in
                        // WizTree, every link shows the file's size, but its clusters are counted
                        // once (on the first link).
                        let link_flags = IN_USE_BASE | flag::EXTRA_LINK;
                        let size = tree.size[r];
                        tree.push(
                            name_off,
                            name_len,
                            parent,
                            size,
                            0,
                            tree.mtime[r],
                            link_flags,
                        );
                        report.extra_links += 1;
                    }
                }
            }
        }
    }
    drop(outs);
    for (i, f) in tree.flags.iter_mut().enumerate() {
        let named = tree.name_len[i] > 0 || i == ROOT_RECORD as usize;
        *f = if *f & IN_USE_BASE != 0 && named {
            (*f | flag::PRESENT) & !(IN_USE_BASE | DOS_NAME)
        } else {
            0
        };
    }
    report.merge = start.elapsed();

    let start = Instant::now();
    report.tree = tree.finish(ROOT_RECORD);
    report.finish = start.elapsed();
    Ok((tree, report))
}
