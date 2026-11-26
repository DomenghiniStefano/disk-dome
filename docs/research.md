# Research and measurements

What was checked while defining the project (2026-11-24), with sources.

## WizTree, the reference to beat

- Proprietary software by Antibody Software (New Zealand). Free for personal use, paid for business use.
- Written in **Delphi 12 (Object Pascal) with the VCL**: `WizTree64.exe`, extracted from the 4.31 installer,
  carries "Embarcadero Delphi for Win64 compiler version 36.0 (29.0.52161.7750)" and `Vcl.*` units, including
  `Vcl.Direct2D`.
- Delphi's Win64 compiler is not LLVM-based: limited inlining, no auto-vectorisation.
- Its speed comes from the approach (reading the `$MFT`), not from the language.
- On the reference drive, first reading (2026-11-24, WizTree's own figures): 13,817,523 entries (11,974,106
  files, 1,843,417 folders), 23.97 s, 6,240 MB of memory (~473 bytes per entry). The `$MFT` is 13.7 GB, so
  23.97 s would mean ~600 MB/s. Not reproduced: timed from outside by `./scripts/proof.ps1` on 2026-11-26 (14.1
  million entries, scan and folder export), WizTree took 182 s with a peak of 2.0 GB.
- Also available for macOS (WizTreeMac) and Linux (WizTreeNix) since version 4.34 (2026-11-17).

## Reference drive

KIOXIA XG8 KXG80ZNV1T02: NVMe PCIe Gen4, 1 TB, 7,000 MB/s sequential read on paper, 4–5.5 GB/s measured in
laptops. The `$MFT` is not read at that speed: it is 14 GB in 64 extents, and `diskdome-cli mft-read` reads it in
4.5–5.5 s (2.6–3.1 GB/s) with eight threads and 4 MB chunks, 15–30 s with 32 threads (on mains power,
2026-11-26): more concurrent reads slow this drive down. The 3–4 s written here earlier came
from the sequential figure, never from a measurement.

## Rust tools that read the MFT

eDirStat, mathom (Rust + Tauri), HyperDU (`--mft` option), Diskraptor. To be tried on the reference drive
before step 4. Others: SquirrelDisk (Rust + Tauri), dua-cli, dust. DiskDome's differences: design, memory,
safety (non-elevated interface), licence.

## Why Slint

| | Slint | egui | Iced | Tauri + web |
| --- | --- | --- | --- | --- |
| API stability | stable 1.x since 2023, a company behind it | 0.x | 0.x | stable |
| wgpu integration | official (unstable feature) | paint callbacks | shader widget | WebGL only |
| Million-row lists | lazy `Model` | `show_rows` | new table widget | to build |
| Token design system | strong, checked by the compiler | hard | medium | strong (CSS) |
| Interface memory | minimal | minimal | minimal | +100–200 MB |

- Founders Olivier Goffart and Simon Hausmann come from Trolltech (Qt); one of them led the QML engine.
- Runtime under 300 KiB of RAM (runs on a Raspberry Pi Pico).
- Typed tokens: style mistakes found at compile time.
- Live preview: change the interface without recompiling Rust.
- In production: EV chargers (SK Signet), autonomous trains (OTIV), audio plug-ins (WesAudio).
- Against: a declarative language to learn; a smaller ecosystem than the web's; AI assistants know it less
  than Vue / CSS.

## Licence

GPL-3.0-or-later: DiskDome is an application, not a library, so companies use it freely (the GPL binds only
whoever distributes modified versions); nobody can turn it into a closed product; Slint under the GPL asks for
no attribution; as the copyright holder the owner can offer a commercial licence later (external contributions
then need a CLA). Same choice as EmuDome. Dependencies must carry compatible licences: `deny.toml`.

## Sources

- WizTree: https://diskanalyzer.com/donate · https://diskanalyzer.com/download · https://diskanalyzer.com/eula
- KIOXIA XG8: https://europe.kioxia.com/pt-pt/business/ssd/client-ssd/xg8.html ·
  https://www.notebookcheck.net/Kioxia-XG8-KXG8AZNV1T02-SSD-Benchmarks.753982.0.html
- Slint: https://www.slint.dev/blog/announcing-slint-1.0 · https://slint.dev/declarative-rust-gui ·
  https://slint.dev/blog/slint-1.13-released · https://docs.slint.dev/latest/docs/rust/slint/wgpu_28 ·
  https://raw.githubusercontent.com/slint-ui/slint/HEAD/LICENSE.md
- Slint's founders: https://devclass.com/2023/04/06/interview-the-story-behind-slint-1-0-a-new-cross-platform-gui-toolkit-coded-in-rust
- Slint in production: https://korben.info/en/slint-gui-toolkit-rust-cpp-js-python.html
- ntfs-reader: https://docs.rs/crate/ntfs-reader/0.4.5
- eDirStat https://docs.rs/crate/edirstat/2.0.0 · mathom https://libraries.io/cargo/mathom-scanner ·
  HyperDU https://raw.githubusercontent.com/automationjp/HyperDiskUsage/main/README.en.md ·
  Diskraptor https://alternativeto.net/software/diskraptor/
