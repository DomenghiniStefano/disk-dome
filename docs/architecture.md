# Architecture

The parts that decide performance are DiskDome's own and independent of the UI toolkit: the scanners, the
data model, the treemap and the virtual tree table. Slint only draws the frame around them (panels, menus,
fields, text).

## Crates

| Folder | Package | Role |
| --- | --- | --- |
| `crates/core` | `diskdome-core` | data model, folder totals, sorting, per-extension statistics; no OS or UI dependency |
| `crates/scan-ntfs` | `diskdome-scan-ntfs` | Windows: reads the `$MFT` directly with `windows-rs`, parses it in parallel with `rayon`; empty on other platforms |
| `crates/scan-fs` | `diskdome-scan-fs` | other drives, single folders, network paths, the no-admin mode and macOS: parallel directory walk (`getattrlistbulk` on macOS) |
| `crates/treemap` | `diskdome-treemap` | squarified layout and its rendering into a pixel buffer |
| `crates/ui` | `diskdome-ui` (binary `diskdome`) | the Slint app: design system, screens, a lazy `Model` that serves only the visible rows |
| `crates/cli` | `diskdome-cli` | scan benchmarks and exports; every engine change is measured here |
| `crates/platform` | `diskdome-platform` | OS services outside the scanners: process memory now, the shell actions later |

Packages are prefixed `diskdome-` because `core` is the name of Rust's own core library.

Data flows one way: the scanners write into `core`; `ui`, `treemap` and `cli` read from `core` without
copying; the treemap hands `ui` nothing but a pixel buffer.

## Data model

One array per field (struct of arrays), `u32` indices, no full path stored (a path is rebuilt by walking up
the parents when needed). `diskdome_core::Tree`.

```
names:       Vec<u8>   every name, UTF-8, in one buffer
name_off:    Vec<u32>  + name_len: Vec<u16>
parent:      Vec<u32>
size, alloc: Vec<u64>  folders: the totals of their subtree
mtime:       Vec<u32>  Unix seconds, enough until 2106
flags:       Vec<u8>   present, folder, extra hard link, linked to the root
child_start: Vec<u32>  + children: Vec<u32>  (CSR: the children of i are children[child_start[i]..child_start[i+1]])
ext:         Vec<u16>  interned extensions (not yet: arrives with the extension statistics)
```

An entry's index is the scanner's own dense number: for NTFS, the MFT record number. The scanner writes each
record straight into its slot and nothing is copied or renumbered afterwards; free records stay as empty slots
(a few percent). Extra hard links are appended after the last record. `Tree::finish` builds the CSR arrays and
sums the totals in one breadth-first pass.

Measured on a generated drive of 14.3 million records (13.9 million entries, `scan --synthetic`): 817 MB,
61.5 bytes per entry (names 286 MB); peak private memory of the whole scan 1.1 GB. WizTree: 6.2 GB.

## NTFS scan pipeline

1. Get the `$MFT`'s extents with `FSCTL_GET_RETRIEVAL_POINTERS` on `X:\$MFT` (a 13.7 GB MFT is
   fragmented); if that is refused, decode the data runs of record 0.
2. Cut the `$MFT` into 4 MB chunks. A pool of threads (twice the cores, at most 32) takes chunks in turn: each
   reads its chunk with an unbuffered read on its own volume handle (Windows serialises synchronous reads on a
   shared handle), then parses it. With more threads than cores, some always wait on the drive while others
   parse: reading and parsing overlap with no queue.
3. Each record is parsed in place (fixup, `$STANDARD_INFORMATION`, `$FILE_NAME`, unnamed `$DATA`) into its
   own slot: the record number is the index, so no hash map and no lock (`ntfs-reader`'s benchmark: 3.8 s with
   a `Vec`, 5.3 s with a `HashMap`). Names go to a per-chunk buffer, joined into one at the end.
4. Extension records (attribute lists) and second hard links are collected per chunk and merged after.
5. Sum sizes in one bottom-up pass (sequential: 0.55 s for 14 million entries; parallel only if it matters).

Measured (`scan --synthetic 14300000`, Ryzen 7 9700X, 8 cores): parsing and building 1.1 s in all
(read+parse 0.46 s on 16 threads, merge 0.06 s, totals 0.55 s). On a real drive the read is the bound: a
13.7 GB `$MFT` at 4–5 GB/s is 3–3.5 s, so ~4 s is expected; `scripts/proof.ps1` measures it.

What is counted, as WizTree does (checked folder by folder on a 3.4 million entry drive: 99.1% of folders
identical, the rest changed between the two scans or are hard links whose clusters WizTree places on another
link):

- size: the unnamed `$DATA` stream;
- size on disk: every `$DATA` stream, named ones included (files compressed by Windows keep their data in
  `WofCompressedData`, the USN journal in `$J`), using the compressed size for compressed and sparse streams;
  resident data takes no cluster; `$BadClus:$Bad` (a sparse map as large as the volume) is not counted;
- hard links: every link is listed with the file's size; the clusters are counted once, on the record's first
  name; DOS 8.3 aliases are not links.

NTFS cases to handle: attribute lists, non-resident attributes and data runs, hard links (counted once),
sparse and compressed files, alternate data streams, logical vs allocated size, reparse points. Check: folder
totals compared with WizTree's on the same drive.

Crates worth reading before writing our own parser: `ntfs-reader` (MFT in memory + USN journal), `ntfs`
(Colin Finck), `mft`.

## Keeping the model current

Every action of the app (delete, move, rename) updates the model and the totals of every parent folder at
once. Changes made outside the app arrive through the USN change journal, with no rescan.

## Administrator rights

- The interface always runs with the user's normal rights: deletions use the user's rights, and drag and drop
  with Explorer works (Windows blocks it between processes at different integrity levels).
- Only a small elevated component reads the `$MFT` and hands the data to the interface through shared memory
  (the array layout of the model suits it).
- The user picks the scan mode in a dialog at first launch and can change it in the settings:
  1. fast, with a UAC prompt at each scan (recommended);
  2. fast, without prompts: a service installed once (as Everything does);
  3. without administrator rights: directory walk, slower.

## Treemap

Redrawn only on resize or zoom: the CPU draws into a buffer (1920×1080, tens of thousands of shaded
rectangles, split across cores, a few milliseconds), handed to Slint as a `SharedPixelBuffer` (stable API).
Hover is a rectangle drawn on top. Rectangles under 1–2 px are merged. wgpu (Slint's integration sits behind an
`unstable-wgpu-*` feature) only where the profiler asks for it.

## Interface (Slint)

- `.slint` files are compiled into Rust at build time (`slint_build::compile` in `build.rs`,
  `slint::include_modules!()`).
- Lists and tables implement Slint's `Model` trait (row count + row N): Slint asks only for the visible rows.
- Live preview while developing: the VS Code extension, and inside the running app with
  `SLINT_LIVE_PREVIEW=1` and the `slint/live-preview` feature.
- Text through `@tr()`.
- Renderer: Slint's default for now; Skia is to be evaluated for text quality.

## CI and releases

- GitHub Actions: `windows-latest` and `macos-latest` run format, clippy and tests; `cargo-deny` checks
  licences, advisories and sources.
- To add: a benchmark of `diskdome-cli` on an NTFS test image, so a time or memory regression shows at once.
- Releases from a tag. Code signing: decided before the first release.

## Technical proof (step 3): the go / no-go

On the reference drive: scan ≤ 8 s, memory ≤ 1.5 GB, folder totals equal to WizTree's. Alongside: a Slint
table with 14 million fake rows that scrolls smoothly. If the engine misses the target, the project stops
before the design work is spent.

`./scripts/proof.ps1` runs it on the PC it is launched on: it builds the CLI, asks for administrator rights
once, reads the `$MFT` alone three times (`mft-read`), scans three times (`scan`), exports the folder totals,
has WizTree export the same drive and compares the two (`compare`). Report in
`%TEMP%\diskdome-proof\<date-time>\report.txt`. With `-Dump` it also saves the `$MFT` to a file, so the
parser can be measured and debugged later without administrator rights (`scan --dump`).
