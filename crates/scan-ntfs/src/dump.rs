//! An `$MFT` saved to a file (`diskdome-cli mft-dump`): lets the parser be measured and debugged
//! on a real drive's records without administrator rights.

use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::PathBuf;

use crate::pipeline::{ChunkReader, ScanError, Source};

/// The file starts with a 16-byte header: magic, record size, cluster size (0 in early dumps: 4096),
/// then the records.
pub const MAGIC: &[u8; 8] = b"DDMFT001";
pub const HEADER: u64 = 16;

pub struct DumpSource {
    path: PathBuf,
    len: u64,
    record_size: usize,
    cluster_size: u32,
}

impl DumpSource {
    pub fn open(path: PathBuf) -> Result<Self, ScanError> {
        let io = |code: std::io::Error| ScanError::Io {
            op: "open dump",
            code: code.raw_os_error().unwrap_or(-1),
        };
        let mut file = File::open(&path).map_err(io)?;
        let mut header = [0u8; HEADER as usize];
        file.read_exact(&mut header).map_err(io)?;
        if &header[..8] != MAGIC {
            return Err(ScanError::BadMft("not a DiskDome $MFT dump"));
        }
        let record_size = u32::from_le_bytes([header[8], header[9], header[10], header[11]]);
        let cluster_size =
            match u32::from_le_bytes([header[12], header[13], header[14], header[15]]) {
                0 => 4096,
                c => c,
            };
        let len = file.metadata().map_err(io)?.len().saturating_sub(HEADER);
        Ok(Self {
            path,
            len,
            record_size: record_size as usize,
            cluster_size,
        })
    }
}

impl DumpSource {
    /// Record `number` with its fixup applied, or `None` if it is out of range or not a valid record.
    pub fn record(&self, number: u64) -> Option<Vec<u8>> {
        let mut file = File::open(&self.path).ok()?;
        file.seek(SeekFrom::Start(HEADER + number * self.record_size as u64))
            .ok()?;
        let mut rec = vec![0u8; self.record_size];
        file.read_exact(&mut rec).ok()?;
        crate::record::apply_fixup(&mut rec).then_some(rec)
    }
}

pub struct DumpReader(File);

impl ChunkReader for DumpReader {
    fn read_at(&mut self, offset: u64, buf: &mut [u8]) -> Result<(), ScanError> {
        let io = |e: std::io::Error| ScanError::Io {
            op: "read dump",
            code: e.raw_os_error().unwrap_or(-1),
        };
        self.0.seek(SeekFrom::Start(HEADER + offset)).map_err(io)?;
        let mut filled = 0;
        while filled < buf.len() {
            let n = self.0.read(&mut buf[filled..]).map_err(io)?;
            if n == 0 {
                break;
            }
            filled += n;
        }
        buf[filled..].fill(0);
        Ok(())
    }
}

impl Source for DumpSource {
    type Reader<'a> = DumpReader;
    fn open_reader(&self) -> Result<Self::Reader<'_>, ScanError> {
        File::open(&self.path)
            .map(DumpReader)
            .map_err(|e| ScanError::Io {
                op: "open dump",
                code: e.raw_os_error().unwrap_or(-1),
            })
    }
    fn len(&self) -> u64 {
        self.len
    }
    fn record_size(&self) -> usize {
        self.record_size
    }
    fn alignment(&self) -> usize {
        self.record_size
    }
    fn cluster_size(&self) -> u32 {
        self.cluster_size
    }
}

/// Copies every record of `source` into `out`, with the dump header. Single-threaded, in order.
pub fn write_dump(source: &impl Source, out: &mut impl std::io::Write) -> Result<u64, ScanError> {
    let io = |e: std::io::Error| ScanError::Io {
        op: "write dump",
        code: e.raw_os_error().unwrap_or(-1),
    };
    let mut header = [0u8; HEADER as usize];
    header[..8].copy_from_slice(MAGIC);
    header[8..12].copy_from_slice(&(source.record_size() as u32).to_le_bytes());
    header[12..16].copy_from_slice(&source.cluster_size().to_le_bytes());
    out.write_all(&header).map_err(io)?;
    let align = source.alignment().max(source.record_size()) as u64;
    let chunk = (16u64 << 20) / align * align;
    let mut buf = crate::buf::AlignedBuf::new(chunk as usize).ok_or(ScanError::OutOfMemory)?;
    let buf = buf.as_mut_slice();
    let mut reader = source.open_reader()?;
    let mut offset = 0;
    while offset < source.len() {
        let valid = (source.len() - offset).min(chunk);
        let read_len = (valid.div_ceil(align) * align) as usize;
        reader.read_at(offset, &mut buf[..read_len])?;
        out.write_all(&buf[..valid as usize]).map_err(io)?;
        offset += valid;
    }
    Ok(offset)
}
