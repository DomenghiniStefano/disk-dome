# Decisions

Every product and technical decision with its reason, newest first. A change of direction is a new entry,
never an edit of an old one.

| Date | Decision | Why |
| --- | --- | --- |
| 2026-11-24 | Data model indexed by the MFT record number, children in CSR arrays (`child_start` + `children`) instead of renumbering entries so children are adjacent | the parser writes each record straight into its slot from any thread and nothing is copied after; costs 4 bytes per entry plus the few percent of free records; measured 61.5 bytes per entry |
| 2026-11-24 | Read and parse on the same threads (twice the cores, 4 MB chunks), one volume handle per thread | overlaps reading and parsing without a queue; Windows serialises synchronous reads on a shared handle |
| 2026-11-24 | Our own `$MFT` parser, no `ntfs` / `ntfs-reader` / `mft` crate | they allocate per record or per attribute; ours parses in place into the model's arrays |
| 2026-11-24 | `crates/platform` created now, for the process memory the benchmarks report | keeps `#[cfg(windows)]` out of the CLI; the shell actions will join it |
| 2026-11-24 | A file counts its unnamed `$DATA` only; hard links counted once (other links shown with size 0) | first version, to check against WizTree; alternate streams decided on what the comparison shows |
| 2026-11-24 | Branches: `develop` for development, `main` for releases only (merge from `develop` + tag, by the owner) | the owner's usual flow in the Dome projects |
| 2026-11-24 | Licence: GPL-3.0-or-later | an application, so companies use it freely; no closed forks; Slint under the GPL needs no attribution; a commercial licence stays possible (with a CLA) |
| 2026-11-24 | v1: Windows only; macOS after v1, built in CI from day one | the `$MFT` scanner is Windows-only; macOS needs its own scanner, signing and notarisation |
| 2026-11-24 | v1 is complete: analysis plus file-manager actions (`docs/features.md`) | the owner wants a finished tool, not a viewer |
| 2026-11-24 | Administrator rights: scan mode chosen at first launch, changeable in the settings (UAC / service / none); the interface is never elevated | safety of deletions, drag and drop with Explorer, user choice |
| 2026-11-24 | Every action updates the model at once; outside changes come from the USN journal | the view never lies after a delete or a move |
| 2026-11-24 | Name: DiskDome | clear, fits the "word + Dome" family; rejected SpaceDome (planetariums), VolumeDome (sounds like an audio app next to SoundDome), FreeDiskDome (sounds like freeware, blocks a commercial licence), two-word names (break the family's pattern) |
| 2026-11-24 | Language: Rust for the engine and the interface | native code, no runtime, one language |
| 2026-11-24 | Interface toolkit: Slint; no web technology (Tauri / WebView) | lightness is the product's reason to exist; stable 1.x API; typed tokens; live preview. egui and Iced are 0.x; GPUI is not a reference for this project |
| 2026-11-24 | Windows scan: read the `$MFT` directly | 10–50× faster than walking directories |
| 2026-11-24 | Treemap drawn by the CPU into a pixel buffer; wgpu only if profiling asks | redrawn only on resize or zoom; Slint's wgpu integration is behind an unstable feature |
| 2026-11-24 | Compact, data-dense interface | 14 million entries to explore |
| 2026-11-24 | Design system base: EmuDome's structure and checks, NoteDome's principles, rapportino's density | EmuDome is the only one whose rules are checked automatically |
| 2026-11-24 | Repository public on GitHub, CI on GitHub Actions (Windows + macOS) | free runners for public repositories |
| 2026-11-24 | Code and documentation in English; interface text through `@tr()`, English source with an Italian translation | same convention as the other Dome projects |
