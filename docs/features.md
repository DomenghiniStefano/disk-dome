# Features

v1 is complete: not only seeing where the space goes, but acting on files as in a file manager. Windows only;
macOS after v1. The owner widened v1 on 2026-11-24 to the views and features of `docs/design-system.md` § 4.5–4.8.

| Area | Feature | Version |
| --- | --- | --- |
| Scan | NTFS scan from the `$MFT`, every local drive | v1 |
| Scan | Directory-walk scan for other drives, single folders and network paths | v1 |
| Scan | Rescan of a single folder | v1 |
| Scan | Live updates from the USN journal, no rescan | v1 |
| Views | Tree with size, allocated space, %, files, folders, modified date, attributes | v1 |
| Views | Treemap with zoom, hover and selection linked to the tree | v1 |
| Views | The largest files of the whole drive | v1 |
| Views | Per-extension statistics, in the treemap's colours | v1 |
| Views | Start: every drive with Windows' numbers, folders, network paths, recent scans | v1 |
| Views | Overview: one sentence on the drive, the map, what can be freed, largest folders and files | v1 |
| Views | Map shapes Rectangles, Rings, Layers; map coloured by type or by age | v1 |
| Views | Timeline: space by month, calendar, history and forecast, compare with a past scan | v1 |
| Views | Statistics: content, sizes, space on disk, attributes, links, owners, names and paths, downloads, NTFS | v1 |
| Views | Activity: what changes now, from the USN journal | v1 |
| Views | Windows' own space explained, with links to Windows' tools (never deleted by DiskDome) | v1 |
| Search | By name, wildcards and regular expressions | v1 |
| Search | Filters by size, date, extension and attributes, applied to every view | v1 |
| Search | Written filters (`ext:mp4 size:>1GB modified:<2024 owner:… path:…`) and saved filters | v1 |
| Search | Command palette (Ctrl+K) and the list of shortcuts (Ctrl+\) | v1 |
| Actions | Open, show in Explorer, open a terminal here, Windows Properties | v1 |
| Actions | Copy full path, name, parent folder; copy the selection as a list | v1 |
| Actions | Delete to the Recycle Bin or permanently, with confirmation and protection of system files | v1 |
| Actions | Rename and move | v1 |
| Actions | Multiple selection and keyboard shortcuts | v1 |
| Clean-up | Guided clean-up: rebuildable developer folders (`node_modules`, `target`, npm, cargo and gradle caches), Temp, old Downloads, in one list | v1 |
| Clean-up | Empty folders and files untouched for a long time | after v1 |
| Clean-up | Duplicates (size from the MFT, then content compared on request; never the last copy) | v1 |
| Clean-up | Compare two scans: what grew (saved scan summaries) | v1 |
| Clean-up | Space per owner, files downloaded from the internet | v1 |
| Integration | CSV / JSON export, an HTML report, the map as an image, and the CLI | v1 |
| Integration | "Analyse with DiskDome" in Explorer's context menu, and a command line to open a folder | v1 |
| Settings | Scan mode (preselected, «Change» on the Start tab), dark / light theme, density, language (English / Italian), units (like File Explorer, GiB, decimal GB), visible columns | v1 |
| Actions | File actions run in parallel, each with its own progress in the status bar | v1 |
| Platforms | macOS | after v1 |

On the reference drive more than 6.5 million of the 12 million files are `.js` / `.ts`, mostly `node_modules`:
the developer clean-up is a concrete advantage over the existing tools.
