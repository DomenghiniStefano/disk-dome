---
name: slint
description: DiskDome's Slint development guide. Use before writing or changing any .slint file, a Dome UI component (crates/dome-ui), a Kit section or harness (crates/ui/ui/kit), or an off-screen UI test (crates/ui/tests/*_headless.rs), and before calling such work done.
---

# Slint work in DiskDome

1. Read `docs/slint.md` in full before the first edit: the layers, the component rules, how screens talk to Rust,
   the Kit, the off-screen tests and the Slint 1.19 traps. It is the single source; this skill only points to it.
2. For a Dome UI component, also read its section in `docs/design-system.md` § 5 and § 8 (8.0 library-grade,
   8.4 names), and list its behaviour against Radix / shadcn on its Trello card before building.
3. Before building on a Slint element, read its source in `~/.cargo/registry/src/*/i-slint-*-1.19.1/`.
4. Before saying the work is done, go through «Checklist before «done»» at the end of `docs/slint.md`, item by
   item, and say on the card which ones were met.

When a new trap or rule turns up while working, add it to `docs/slint.md` in the same commit, not here.
