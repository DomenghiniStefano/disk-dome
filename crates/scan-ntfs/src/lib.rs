//! Windows NTFS scanner.
//!
//! Reads the volume's `$MFT` directly: its extents from `FSCTL_GET_RETRIEVAL_POINTERS` (or, failing
//! that, from the data runs of record 0), large unbuffered reads on a few threads at once (eight: more
//! concurrent reads slow an NVMe drive down), each thread parsing the chunk it just read while the
//! others wait on the drive. The MFT record number
//! is already a dense index, so a record's slot and its parent are found by array access, not a map.
//! Needs administrator rights; it runs in the small elevated component, never in the interface.
//!
//! The parser, the pipeline and the dump / synthetic sources are plain Rust and build everywhere,
//! so they are tested on every platform; only reading a volume (`volume`) is Windows-only.
//! See `docs/architecture.md`.

mod buf;
pub mod dump;
pub mod pipeline;
pub mod record;
pub mod synthetic;
#[cfg(windows)]
mod volume;
#[cfg(not(windows))]
#[path = "volume_stub.rs"]
mod volume;

pub use pipeline::{Options, ReadReport, ScanError, ScanReport, Source, read_only, scan};
pub use volume::Volume;

/// What the CLI prints about a volume before reading it.
#[derive(Debug, Clone, Copy, Default)]
pub struct VolumeSummary {
    pub bytes_per_cluster: u32,
    pub record_size: u32,
    pub mft_bytes: u64,
    pub extents: usize,
    /// `true`: extents from `FSCTL_GET_RETRIEVAL_POINTERS`; `false`: from record 0's data runs.
    pub from_retrieval_pointers: bool,
}

#[cfg(test)]
mod tests;
