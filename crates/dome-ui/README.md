# Dome UI

The design system of the Dome apps (`docs/design-system.md`, which holds every value and the reasons): a Slint library
with its Rust half. Nothing in it is specific to DiskDome; it moves to its own repository with the second Dome app
(card https://trello.com/c/9yiz5cgx), and the name is provisional.

Using it from an app:

- `build.rs`: `with_library_paths({"dome-ui": "<path>/crates/dome-ui/ui/slint-project.json"})`; the `.slint` files
  import `from "@dome-ui"`. The project file makes every other file private: the compiler refuses them.
- Rust: depend on `dome-ui`, then `dome_ui::menu_nav!(MenuNav, MenuEntry, MenuEntryKind);` in one module (with the
  app's generated types in scope) writes `install(window)`; every window calls it once.
- Translations: the components' `@tr` strings are compiled with the app and go in the app's `.po`.

The Slint half, in `ui/`:

- `index.slint` — the only file an app imports, as `@dome-ui`.
- `tokens/` — typed globals, the only place with literal colours, lengths and durations:
  `Theme` (colours per theme), `Data` (file categories, age), `Size` (spacing, sizes, density, radii, borders,
  opacity, layers), `Type` (font and type steps), `Motion` (durations, easing).
- Run-time switches set by Rust: `Theme.mode`, `Size.compact`, `Data.palette`, `Data.style`, `Motion.reduced`.
- `enums.slint` — the components' variants and sizes (`ButtonVariant`, `InkTone`…); `TypeStep` is in `tokens/type.slint`.
- `tones.slint` — `InkTone` → colour, shared by `Label` and `Icon`.
- `input-mode.slint` — `InputMode.keyboard`: the focus ring shows only while the user moves with the keyboard (§ 3.7).
- `icons/` — the Lucide SVGs (version pinned in `icons.slint` and `THIRD_PARTY.md`) and `global Icons`, one
  property per file.
- `components/` — one file per component of § 5, each exported through `index.slint`: `Label`, `Icon`, `Button`,
  `SegmentedControl`, `ToggleButton`, `SearchField`, `ScrollBar` (vertical or horizontal), `DomeWindow` (the window
  every screen inherits: it tells the popups its size), and the menu (§ 5.16): `MenuFrame` (the level-1 surface),
  `MenuList` (entries from `[MenuEntry]`), `Menu` (the popup, three levels), `ContextMenu` (right click, Menu key,
  Shift+F10), `MenuButton`, and the tooltip (§ 5.17): `Tooltip` (inside a control; `Button` and `ToggleButton`
  have it through `tooltip`) and `TooltipBubble`, drawn by `DomeWindow` from `primitives/tooltip-layer.slint`.
- `MenuNav` (in `menu-list.slint`) — what Slint cannot compute (no loops, no prefix test): the walk over a menu's
  entries, its type-ahead and `toggled`, answered by `src/menu.rs` and wired by `menu_nav!`.
- `primitives/` — the behaviour components share, never used by screens: `FocusTarget` (one keyboard stop),
  `Pressable` (pointer + keyboard, the states of § 5), `FocusRing`, `TextEntry` (the one editable text, which like
  `Label` sets font properties), `placement.slint` (`Viewport` and `Placement`: where a popup goes, § 3.11).
- The Kit window (§ 9) lives in the app, `crates/ui/ui/kit/`: `cargo run -p diskdome-ui --features kit -- --kit`. `kit.slint` holds the window
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
keyboard and pointer behaviour off screen by `kit_headless.rs` (`DISKDOME_KIT_SHOTS=<folder>` saves screenshots); the
menu on its test window `kit/kit-menu-harness.slint` by `menu_headless.rs`.

Slint traps met by the menu: an element of size 0 outside a layout is centred in its parent (a `Menu` covers its
parent instead); inside a `PopupWindow`, `absolute-position` is the popup's own; a `PopupWindow` takes the keyboard focus when it shows (so the tooltip is
not one); a size that depends on whether a
scroll bar shows overflowed the compiler's stack (the menu's scroll bar lies over the rows); `accessible-role` must
be constant (an `if` element per role).
