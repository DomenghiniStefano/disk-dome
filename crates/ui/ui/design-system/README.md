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
  `SegmentedControl`, `ToggleButton`, `SearchField`, `ScrollBar` (vertical only for now).
- `primitives/` — the behaviour components share, never used by screens: `FocusTarget` (one keyboard stop),
  `Pressable` (pointer + keyboard, the states of § 5), `FocusRing`, `TextEntry` (the one editable text, which like
  `Label` sets font properties).
- `kit/` — the Kit window (§ 9): `cargo run -p diskdome-ui --features kit -- --kit`. `kit.slint` holds the window
  and the component sections (every component shows each state through `kit-state: KitState`, Kit only; screens
  leave it `live`); `kit-tokens.slint` draws every token (a new token gets its line in `KitTokenData`, shown in
  units of `Size.unit` and `Motion.unit`); `kit-review.slint` is the review board (`KitReview.issues`, «Now» and
  «Proposal» drawn with the real components, counted by `src/kit.rs` for the top bar); `kit-text.slint` is «Text and
  digits» (test 7: the steps at 12–13 px, a tree total in bold, a live tabular-digits check);
  `kit-parts.slint` the Kit's own layout pieces and `KitSearch`, the top bar's search over section titles (Rust
  answers the match, plain text or regular expression).

How a component is built: variants and sizes are enums; every look is a token or a pure function over tokens
(`ButtonLook`); behaviour comes from `primitives/`; text goes through `Label`, icons through `Icon`; disabled fades
the whole control with one `opacity`.

The rules are checked by `crates/ui/tests/design_conventions.rs` (§ 10); the token values are compared with the
tables of `docs/design-system.md` by `crates/ui/tests/design_tokens.rs`; the icon set by `design_icons.rs`; the Kit's
keyboard and pointer behaviour off screen by `kit_headless.rs` (`DISKDOME_KIT_SHOTS=<folder>` saves screenshots).
