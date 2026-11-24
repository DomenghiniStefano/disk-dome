//! DiskDome's data model.
//!
//! The scanned tree is stored as one array per field (struct of arrays) with `u32` indices:
//! names in a single UTF-8 buffer, children contiguous (CSR layout), no full paths stored.
//! Scanners write into it; the interface, the treemap and the CLI read from it without copying.
//! See `docs/architecture.md` for the layout and the memory budget (~65 bytes per entry).
