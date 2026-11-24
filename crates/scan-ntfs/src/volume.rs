//! Windows: reading an NTFS volume's `$MFT` straight from the volume device.
//!
//! The volume is opened read-only (`GENERIC_READ`, unbuffered); nothing here can write, delete or
//! move a file (rule 4 of `CLAUDE.md`). Needs administrator rights.

use std::ptr::null_mut;

use windows_sys::Win32::Foundation::{
    CloseHandle, ERROR_ACCESS_DENIED, ERROR_MORE_DATA, GENERIC_READ, GetLastError, HANDLE,
    INVALID_HANDLE_VALUE,
};
use windows_sys::Win32::Storage::FileSystem::{
    CreateFileW, FILE_FLAG_BACKUP_SEMANTICS, FILE_FLAG_NO_BUFFERING, FILE_READ_ATTRIBUTES,
    FILE_SHARE_DELETE, FILE_SHARE_READ, FILE_SHARE_WRITE, OPEN_EXISTING, ReadFile,
};
use windows_sys::Win32::System::IO::{DeviceIoControl, OVERLAPPED};
use windows_sys::Win32::System::Ioctl::{
    FSCTL_GET_NTFS_VOLUME_DATA, FSCTL_GET_RETRIEVAL_POINTERS, NTFS_VOLUME_DATA_BUFFER,
    STARTING_VCN_INPUT_BUFFER,
};

use crate::buf::AlignedBuf;
use crate::pipeline::{ChunkReader, ScanError, Source};
use crate::record::{apply_fixup, data_runs};

/// A handle closed on drop.
struct Handle(HANDLE);

// SAFETY: a file handle can be used from any thread; each reader owns its own.
unsafe impl Send for Handle {}
// SAFETY: as above; the volume handle is only used for ioctls that do not mutate shared state.
unsafe impl Sync for Handle {}

impl Drop for Handle {
    fn drop(&mut self) {
        // SAFETY: the handle was returned by CreateFileW and is closed once.
        unsafe { CloseHandle(self.0) };
    }
}

fn last_error() -> u32 {
    // SAFETY: no preconditions.
    unsafe { GetLastError() }
}

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

fn io_error(op: &'static str) -> ScanError {
    ScanError::Io {
        op,
        code: last_error() as i32,
    }
}

fn open(path: &str, access: u32, flags: u32) -> Result<Handle, ScanError> {
    let path = wide(path);
    // SAFETY: `path` is NUL-terminated and outlives the call.
    let h = unsafe {
        CreateFileW(
            path.as_ptr(),
            access,
            FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
            null_mut(),
            OPEN_EXISTING,
            flags,
            null_mut(),
        )
    };
    if h == INVALID_HANDLE_VALUE {
        return Err(if last_error() == ERROR_ACCESS_DENIED {
            ScanError::NotAdmin
        } else {
            io_error("CreateFileW")
        });
    }
    Ok(Handle(h))
}

/// Geometry of the volume and of its `$MFT`.
#[derive(Debug, Clone, Copy)]
pub struct VolumeInfo {
    pub bytes_per_sector: u32,
    pub bytes_per_cluster: u32,
    pub record_size: u32,
    pub mft_valid_len: u64,
    pub mft_start_lcn: u64,
}

/// A run of `$MFT` clusters: `clusters` clusters from virtual cluster `vcn`, stored at logical
/// cluster `lcn` (`None`: a hole, read as zeros).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Extent {
    pub vcn: u64,
    pub lcn: Option<u64>,
    pub clusters: u64,
}

/// Where the extents were found.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExtentSource {
    RetrievalPointers,
    Record0,
}

pub struct Volume {
    device: String,
    pub info: VolumeInfo,
    pub extents: Vec<Extent>,
    pub extent_source: ExtentSource,
}

impl Volume {
    /// Opens drive `letter` (e.g. `'C'`), reads its NTFS geometry and the `$MFT`'s extents.
    pub fn open(letter: char) -> Result<Self, ScanError> {
        let device = format!(r"\\.\{letter}:");
        let handle = open(&device, GENERIC_READ, FILE_FLAG_NO_BUFFERING)?;
        let info = volume_info(&handle)?;
        let (extents, extent_source) = match retrieval_pointers(letter) {
            Ok(extents) if !extents.is_empty() => (extents, ExtentSource::RetrievalPointers),
            Ok(_) | Err(_) => (record0_extents(&handle, &info)?, ExtentSource::Record0),
        };
        let mapped: u64 =
            extents.iter().map(|e| e.clusters).sum::<u64>() * u64::from(info.bytes_per_cluster);
        if mapped < info.mft_valid_len {
            return Err(ScanError::BadMft("extents shorter than the $MFT"));
        }
        Ok(Self {
            device,
            info,
            extents,
            extent_source,
        })
    }
}

impl Volume {
    pub fn summary(&self) -> crate::VolumeSummary {
        crate::VolumeSummary {
            bytes_per_cluster: self.info.bytes_per_cluster,
            record_size: self.info.record_size,
            mft_bytes: self.info.mft_valid_len,
            extents: self.extents.len(),
            from_retrieval_pointers: self.extent_source == ExtentSource::RetrievalPointers,
        }
    }
}

fn volume_info(handle: &Handle) -> Result<VolumeInfo, ScanError> {
    // SAFETY: plain-old-data structure, zero is a valid value.
    let mut data: NTFS_VOLUME_DATA_BUFFER = unsafe { std::mem::zeroed() };
    let mut returned = 0u32;
    // SAFETY: the output buffer is a properly sized NTFS_VOLUME_DATA_BUFFER.
    let ok = unsafe {
        DeviceIoControl(
            handle.0,
            FSCTL_GET_NTFS_VOLUME_DATA,
            null_mut(),
            0,
            (&raw mut data).cast(),
            size_of::<NTFS_VOLUME_DATA_BUFFER>() as u32,
            &mut returned,
            null_mut(),
        )
    };
    if ok == 0 {
        return Err(ScanError::NotNtfs);
    }
    Ok(VolumeInfo {
        bytes_per_sector: data.BytesPerSector,
        bytes_per_cluster: data.BytesPerCluster,
        record_size: data.BytesPerFileRecordSegment,
        mft_valid_len: u64::try_from(data.MftValidDataLength).unwrap_or(0),
        mft_start_lcn: u64::try_from(data.MftStartLcn).unwrap_or(0),
    })
}

/// The `$MFT`'s extents from `FSCTL_GET_RETRIEVAL_POINTERS` on the `$MFT` file itself.
fn retrieval_pointers(letter: char) -> Result<Vec<Extent>, ScanError> {
    let mft = open(
        &format!(r"\\?\{letter}:\$MFT"),
        FILE_READ_ATTRIBUTES,
        FILE_FLAG_BACKUP_SEMANTICS,
    )?;
    let mut extents = Vec::new();
    let mut next_vcn: i64 = 0;
    // u64 elements keep the buffer 8-byte aligned for RETRIEVAL_POINTERS_BUFFER.
    let mut out = vec![0u64; 8192];
    loop {
        let input = STARTING_VCN_INPUT_BUFFER {
            StartingVcn: next_vcn,
        };
        let mut returned = 0u32;
        // SAFETY: input and output buffers are valid for the sizes given.
        let ok = unsafe {
            DeviceIoControl(
                mft.0,
                FSCTL_GET_RETRIEVAL_POINTERS,
                (&raw const input).cast(),
                size_of::<STARTING_VCN_INPUT_BUFFER>() as u32,
                out.as_mut_ptr().cast(),
                (out.len() * 8) as u32,
                &mut returned,
                null_mut(),
            )
        };
        let more = ok == 0 && last_error() == ERROR_MORE_DATA;
        if ok == 0 && !more {
            return Err(io_error("FSCTL_GET_RETRIEVAL_POINTERS"));
        }
        // Layout: ExtentCount u32, padding, StartingVcn i64, then (NextVcn i64, Lcn i64) pairs.
        let count = (out[0] & 0xFFFF_FFFF) as usize;
        let mut vcn = out[1] as i64;
        for k in 0..count {
            let (Some(&next), Some(&lcn)) = (out.get(2 + 2 * k), out.get(3 + 2 * k)) else {
                break;
            };
            let (next, lcn) = (next as i64, lcn as i64);
            extents.push(Extent {
                vcn: u64::try_from(vcn).unwrap_or(0),
                lcn: u64::try_from(lcn).ok(),
                clusters: u64::try_from(next - vcn).unwrap_or(0),
            });
            vcn = next;
        }
        next_vcn = vcn;
        if !more || count == 0 {
            break;
        }
    }
    Ok(extents)
}

/// Fallback: the extents from the data runs of record 0 (the `$MFT`'s own record).
fn record0_extents(handle: &Handle, info: &VolumeInfo) -> Result<Vec<Extent>, ScanError> {
    let bpc = info.bytes_per_cluster as usize;
    let len = (info.record_size as usize).max(bpc);
    let mut buf = AlignedBuf::new(len).ok_or(ScanError::OutOfMemory)?;
    let buf = buf.as_mut_slice();
    read_exact_at(handle, info.mft_start_lcn * bpc as u64, &mut buf[..len])?;
    let rec = &mut buf[..info.record_size as usize];
    if !apply_fixup(rec) {
        return Err(ScanError::BadMft("record 0 is not a FILE record"));
    }
    let (runs, has_list) = data_runs(rec).ok_or(ScanError::BadMft("record 0 has no $DATA"))?;
    let extents: Vec<Extent> = runs
        .into_iter()
        .map(|(vcn, lcn, clusters)| Extent { vcn, lcn, clusters })
        .collect();
    let mapped = extents.iter().map(|e| e.clusters).sum::<u64>() * bpc as u64;
    if has_list && mapped < info.mft_valid_len {
        return Err(ScanError::BadMft("$MFT runs continue in an attribute list"));
    }
    Ok(extents)
}

fn read_exact_at(handle: &Handle, offset: u64, buf: &mut [u8]) -> Result<(), ScanError> {
    let mut done = 0usize;
    while done < buf.len() {
        let pos = offset + done as u64;
        // SAFETY: plain-old-data structure, zero is a valid value.
        let mut ov: OVERLAPPED = unsafe { std::mem::zeroed() };
        ov.Anonymous.Anonymous.Offset = pos as u32;
        ov.Anonymous.Anonymous.OffsetHigh = (pos >> 32) as u32;
        let want = u32::try_from(buf.len() - done).unwrap_or(1 << 30);
        let mut read = 0u32;
        // SAFETY: the destination range is inside `buf`; `ov` lives until the synchronous call
        // returns (the handle is not opened for overlapped I/O).
        let ok = unsafe { ReadFile(handle.0, buf[done..].as_mut_ptr(), want, &mut read, &mut ov) };
        if ok == 0 {
            return Err(io_error("ReadFile"));
        }
        if read == 0 {
            buf[done..].fill(0);
            return Ok(());
        }
        done += read as usize;
    }
    Ok(())
}

/// One thread's reader: its own volume handle.
pub struct VolumeReader<'a> {
    handle: Handle,
    volume: &'a Volume,
}

impl ChunkReader for VolumeReader<'_> {
    fn read_at(&mut self, offset: u64, buf: &mut [u8]) -> Result<(), ScanError> {
        let bpc = u64::from(self.volume.info.bytes_per_cluster);
        let mut done = 0usize;
        while done < buf.len() {
            let pos = offset + done as u64;
            let vcn = pos / bpc;
            let within = pos % bpc;
            let Some(ext) = self
                .volume
                .extents
                .iter()
                .find(|e| vcn >= e.vcn && vcn < e.vcn + e.clusters)
            else {
                buf[done..].fill(0);
                return Ok(());
            };
            let avail = ((ext.vcn + ext.clusters - vcn) * bpc - within) as usize;
            let n = avail.min(buf.len() - done);
            match ext.lcn {
                None => buf[done..done + n].fill(0),
                Some(lcn) => {
                    let at = (lcn + vcn - ext.vcn) * bpc + within;
                    read_exact_at(&self.handle, at, &mut buf[done..done + n])?;
                }
            }
            done += n;
        }
        Ok(())
    }
}

impl Source for Volume {
    type Reader<'a> = VolumeReader<'a>;
    fn open_reader(&self) -> Result<Self::Reader<'_>, ScanError> {
        Ok(VolumeReader {
            handle: open(&self.device, GENERIC_READ, FILE_FLAG_NO_BUFFERING)?,
            volume: self,
        })
    }
    fn len(&self) -> u64 {
        self.info.mft_valid_len
    }
    fn record_size(&self) -> usize {
        self.info.record_size as usize
    }
    fn alignment(&self) -> usize {
        self.info.bytes_per_cluster as usize
    }
    fn cluster_size(&self) -> u32 {
        self.info.bytes_per_cluster
    }
}
