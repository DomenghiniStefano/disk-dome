//! Directory-walking scanner.
//!
//! Used where the `$MFT` cannot be read: other file systems, single folders, network paths, the
//! "no administrator rights" scan mode, and macOS (`getattrlistbulk`). Walks folders in parallel and
//! fills the same data model as the NTFS scanner. See `docs/architecture.md`.
