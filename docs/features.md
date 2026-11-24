# Features

v1 is complete: not only seeing where the space goes, but acting on files as in a file manager. Windows only;
macOS after v1.

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
| Search | By name, wildcards and regular expressions | v1 |
| Search | Filters by size, date, extension and attributes, applied to every view | v1 |
| Actions | Open, show in Explorer, open a terminal here, Windows Properties | v1 |
| Actions | Copy full path, name, parent folder; copy the selection as a list | v1 |
| Actions | Delete to the Recycle Bin or permanently, with confirmation and protection of system files | v1 |
| Actions | Rename and move | v1 |
| Actions | Multiple selection and keyboard shortcuts | v1 |
| Clean-up | Rebuildable developer folders: `node_modules`, `target`, npm, cargo and gradle caches | v1 |
| Clean-up | Empty folders and files untouched for a long time | after v1 |
| Clean-up | Duplicates (name + size, then content hash) | after v1 |
| Clean-up | Compare two scans: what grew | after v1 |
| Integration | CSV / JSON export and the CLI | v1 |
| Integration | "Analyse with DiskDome" in Explorer's context menu | after v1 |
| Settings | Scan mode (chosen at first launch), dark / light theme, density, language (English / Italian), units (GB or GiB), visible columns | v1 |
| Platforms | macOS | after v1 |

On the reference drive more than 6.5 million of the 12 million files are `.js` / `.ts`, mostly `node_modules`:
the developer clean-up is a concrete advantage over the existing tools.
