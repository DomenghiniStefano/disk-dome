# Design system

The Slint side of `docs/design-system.md`, which holds every value and the reasons.

- `index.slint` — the only file screens and `app.slint` import from.
- `tokens/` — typed globals, the only place with literal colours, lengths and durations:
  `Theme` (colours per theme), `Data` (file categories, age), `Size` (spacing, sizes, density, radii, borders,
  opacity, layers), `Type` (font and type steps), `Motion` (durations, easing).
- Run-time switches set by Rust: `Theme.mode`, `Size.compact`, `Data.palette`, `Data.style`, `Motion.reduced`.
- `enums.slint` — the components' variants and sizes (`ButtonVariant`, `InkTone`…); `TypeStep` is in `tokens/type.slint`.
- `tones.slint` — `InkTone` → colour, shared by `Label` and `Icon`.
- `nav.slint` — `Nav.keyboard`: the focus ring shows only while the user moves with the keyboard (§ 3.7).
- `icons/` — the Lucide SVGs (version pinned in `icons.slint` and `THIRD_PARTY.md`) and `global Icons`, one
  property per file.
- `components/` — one file per component of § 5, each exported through `index.slint`: `Label`, `Icon`, `Button`,
  `SegmentedControl`.
- `primitives/` — the behaviour components share, never used by screens: `FocusTarget` (one keyboard stop),
  `Pressable` (pointer + keyboard, the states of § 5), `FocusRing`.
- `kit/kit.slint` — the Kit window (§ 9): `cargo run -p diskdome-ui --features kit -- --kit`. Every component
  shows each state through `kit-state: KitState` (Kit only; screens leave it `live`).

How a component is built: variants and sizes are enums; every look is a token or a pure function over tokens
(`ButtonLook`); behaviour comes from `primitives/`; text goes through `Label`, icons through `Icon`; disabled fades
the whole control with one `opacity`.

The rules are checked by `crates/ui/tests/design_conventions.rs` (§ 10); the token values are compared with the
tables of `docs/design-system.md` by `crates/ui/tests/design_tokens.rs`; the icon set by `design_icons.rs`; the Kit's
keyboard and pointer behaviour off screen by `kit_headless.rs` (`DISKDOME_KIT_SHOTS=<folder>` saves screenshots).
