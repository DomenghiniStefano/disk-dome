//! Windows NTFS scanner.
//!
//! Reads the volume's `$MFT` directly: its extents from `FSCTL_GET_RETRIEVAL_POINTERS`, large reads
//! with several requests in flight, records parsed on every core while the next blocks are read.
//! The MFT record number is already a dense index, so a parent is found by array access, not a map.
//! Needs administrator rights; it runs in the small elevated component, never in the interface.
//! On other platforms this crate is empty. See `docs/architecture.md`.
