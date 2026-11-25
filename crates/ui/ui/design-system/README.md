# Design system

The Slint side of `docs/design-system.md`, which holds every value and the reasons.

- `index.slint` — the only file screens and `app.slint` import from.
- `tokens/` — typed globals, the only place with literal colours, lengths and durations:
  `Theme` (colours per theme), `Data` (file categories, age), `Size` (spacing, sizes, density, radii, borders,
  opacity, layers), `Type` (font and type steps), `Motion` (durations, easing).
- Run-time switches set by Rust: `Theme.mode`, `Size.compact`, `Data.palette`, `Data.style`, `Motion.reduced`.
- `components/` and `kit/` arrive with cards 4.2 and 4.3.

The rules are checked by `crates/ui/tests/design_conventions.rs` (§ 10); the token values are compared with the
tables of `docs/design-system.md` by `crates/ui/tests/design_tokens.rs`.
