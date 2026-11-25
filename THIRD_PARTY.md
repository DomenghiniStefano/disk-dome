# Third-party assets

Fonts, icons and other assets in the repository, with their licence. Every one is compatible with
GPL-3.0-or-later; check the licence before adding a new one (`CLAUDE.md`, rule 2). Rust crates are listed by
`cargo deny` and, in the app, by the generated third-party notice (`docs/design-system.md` § 6.19).

| Asset | Where | Licence | Notes |
| --- | --- | --- | --- |
| Sora 700 (The Sora Project Authors) | the «DiskDome» wordmark in `crates/ui/ui/assets/brand/logo-*.svg` and `social.png` | SIL Open Font License 1.1 (`crates/ui/ui/assets/brand/OFL-Sora.txt`) | letters converted to outlines; the font file is not shipped |
| Lucide icons (Lucide Icons and Contributors; some derived from Feather, MIT), lucide-static 1.71.0 | the SVGs in `crates/ui/ui/design-system/icons/` | ISC (`crates/ui/ui/design-system/icons/LICENSE-lucide.txt`, with Feather's MIT notice) | unchanged files, each with its licence header; the version is checked by `crates/ui/tests/design_icons.rs` |
