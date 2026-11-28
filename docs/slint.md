# Slint development

How DiskDome's interface and Dome UI (its design system, `crates/dome-ui`) are written in Slint 1.19: the layers,
the components, how screens talk to Rust, the Kit, the off-screen tests and the traps of Slint 1.19. Read it before
touching a `.slint` file, a Kit harness or a headless test. The project skill `/slint` (`.claude/skills/slint/`)
points here; the checklist at the end is the one to go through before calling Slint work done.

The look itself (tokens, components, screens) is `docs/design-system.md`; this file is how it is built.

## Before writing

- **Read the element's source first** in `~/.cargo/registry/src/*/i-slint-*-1.19.1/` (`builtin_elements.rs`,
  `widgets/`, `i-slint-core/window.rs`, `i-slint-renderer-software-1.19.1/`): what it draws, what it takes (focus,
  the pointer), what can be set, what the software renderer supports. Two days of guessing are one `grep`.
- **List the behaviour against Radix / shadcn on the card** before building a component (`docs/design-system.md`
  § 8.0): what exists, what is missing, what stays out and why. When the card and the spec disagree, or the spec
  is silent on a behaviour, ask the owner before building.
- **As configurable as possible**: where platforms or libraries differ on a behaviour (Windows 11's arrows and
  paging against Radix's jump), it becomes a parameter enum, with the spec's choice as default; every size, colour
  and delay is a property whose default is its token.

## Layers and files

- **Layers**: `tokens/` → `primitives/` → `components/` → screens. Each layer imports only the ones below it;
  screens and `app.slint` import only `@dome-ui` (the library's `index.slint`; its project file makes every other
  file private). Dome UI holds nothing specific to DiskDome: it is the design system of every Dome app.
- **Components**: one per file in `crates/dome-ui/ui/components/`, exported from `index.slint`, with its variants
  and sizes as enums in `enums.slint` (exported from `index.slint` too). The look for each variant comes from one
  pure-function global in the file (as `ButtonLook`), not from ternaries scattered over the elements. A type only
  the file uses (`ScrollBarPress`) stays unexported in it.
- **Names** follow `docs/design-system.md` § 8.4: a component's enums, row struct and Rust global carry its name
  (`SegmentedWidth`, `SegmentedItem`, `SegmentedNav`); indices end in `-index`; callbacks are events (`selected`,
  never `changed`); one idea has one name everywhere (`wrap-around`, `width-mode`, `tooltip`).
- Formatting is Slint's own formatter, `./scripts/slint-format.ps1` (the CI runs it with `-Check`); nobody formats
  by hand.

## Components

- **Properties**: `in` by default; `out` only for what the component computes; `in-out` and two-way bindings
  (`<=>`) only when both sides really write, with a comment saying why. Everything that is not the component's
  interface is a private `property` (no direction).
- **Behaviour is never rewritten**: clicks, hover, press, Enter / Space and the focus ring come from
  `primitives/` (`Pressable`, `FocusTarget`, `FocusRing`); the ring shows only when `InputMode.keyboard` is set.
  Disabled is one `opacity: Size.disabled` on the component's root.
- **Text only through `Label`, icons only through `Icon`** with `Icons.*`; colours through `InkTone`. A new
  icon: the SVG from the pinned Lucide version, its property in `icons.slint`, its entry in the Kit
  (`tests/design_icons.rs` checks all three). A label that may be cut tells its control through `cut`; the control
  says the whole text in its tooltip.
- **A value the spec does not give** becomes a token under `// Pending design`, listed on the card for the
  owner. Never a literal, never a guess presented as decided.
- **Accessibility on every component**: `accessible-role`, `accessible-label`, `accessible-enabled`, and
  `accessible-checked` where it applies; an icon-only button always has its `accessible-label`. Where Slint has no
  role for the thing (a scroll bar), say so in the spec and let the content carry it.
- **A pointer area takes every press under it** (an enabled `TouchArea` always grabs the press): never lay one over
  something that must stay clickable (a label in a row) just to watch the hover.

## Screens and Rust

- **Screens talk to Rust through globals**: one `export global` per area (`ScanApi`, `TreeApi`…) with
  properties and callbacks, re-exported by the build entry; one Rust adapter module per area wires it. Screens
  never get data through properties of the `Window`, so the Kit can show them with fake data.
- **Models are lazy**: a list or table implements `slint::Model` over `core` and builds only the rows asked
  for; no `VecModel` copy of the tree. Small fixed lists (tabs, menu items) may be a `VecModel`.
- **From another thread to the interface** only through `Weak::upgrade_in_event_loop` or
  `slint::invoke_from_event_loop`; their error (the event loop has ended) is logged, never dropped with `let _`.
- **What Slint cannot compute** (no loops, no recursion, no substring or regex) is answered in Rust by a
  `pure callback` on a global, wired on every window (`MenuNav`: `dome_ui::menu`, wired by `menu_nav!`;
  `SegmentedNav`, `RadioNav`, `TabNav`: `dome_ui::roving`, by `segmented_nav!`, `radio_nav!`, `tab_nav!`) and unit
  tested in Rust.

## The Kit

`cargo run -p diskdome-ui --features kit -- --kit` (`docs/design-system.md` § 9).

- **The Kit is the proof**: every component, variant and state appears in `kit/kit.slint` (test
  `kit-coverage`); states are shown with `kit-state`, which screens never set.
- **The Kit works, not only shows**: every interactive component in the Kit is wired so that using it does what it
  would in a screen (a check box in a menu flips, a choice is shown, a scroll bar scrolls real content), and the
  owner checks it by hand there. A sample says which option it shows (`KitScrollDemo`'s `name`).
- **A visual change is looked at in the off-screen shots** before it is called done:
  `DISKDOME_KIT_SHOTS=<folder> cargo test -p diskdome-ui --features kit --test kit_headless`. The shots are a local
  look only: never committed, never compared by a test. To find a new section, compare the shot with develop's:
  the first row that differs is where it starts.
- **The Kit shot covers the whole Kit**: a new section raises `HEIGHT` in `tests/kit_headless.rs` until the last
  section is in the shot (the last drawn row stays above `HEIGHT`); a cut shot hides the new work silently.

## Off-screen tests

UI tests run off screen (`crates/ui/src/headless.rs`: Slint's software renderer into a buffer), never in a window.

- **One test window per component**: `kit/kit-<component>-harness.slint`, exported from `dev-app.slint`, with the
  component placed at points the test can compute and the values it reads as `out` properties. Positions and sizes
  come from tokens; the test asserts them against its own constants before using them.
- Expected values come from the spec (`docs/design-system.md` gives the sizes), and a test that passes at once is
  proved by a mutation: both rules are in `CLAUDE.md` › Implementation rules.
- **Wait by the clock** (`Headless::wait`), never by counting turns: the macOS CI draws a frame in tens of
  milliseconds, so «16 turns of 25 ms» lasts over a second there and a 1 s delay has already run out. Whatever
  must be seen before a delay ends gets a delay several times longer in the test window (the ScrollBar's `rest`
  bar stays 3 × `scrollbar-linger`).
- A `Timer` or a `changed` handler is seen one frame later: let two frames run (`settle`) after an action.
- **Test an animation or a visibility by pixels** over frames (`Headless::pixel`), against the window's ground
  read away from everything; `press` / `release` / `hover` / `click_with` drive a drag, a held press or Shift+click.
- The headless platform is installed once per thread: one `#[test]` per file.

## Slint 1.19 traps

- An element inside a component without a layout is centred at its preferred size, so an overlay sets
  `width: 100%; height: 100%`; a helper that only holds a popup (`Menu`) does the same, or its coordinates start
  at the parent's centre.
- A conditional cannot mix a percentage and a length (`vertical ? thickness : 100%` does not compile): bind
  `preferred-width` / `preferred-height` and let the placement set the length.
- `&&` next to `||` needs parentheses.
- An enum value whose name matches a property is written in full (`AccessibleRole.text`).
- `Flickable` takes `content-height` and `content-y`, not the deprecated `viewport-*`.
- `visible: false` keeps the element's place in a layout: to take no room, bind its min / preferred / max height
  to 0 (as `KitSection`) or use `if`.
- Properties of an element inside an `if` cannot be read from outside it: copy what is needed to the parent in a
  `changed` handler and in `init` (as `Button` does with its label's `cut`).
- A component that reads its `width` in `init` or `changed width` lays the window out while it is built, before
  `design_system::install` wires the globals: a pure callback answered by Rust returns its default there. What a
  child hands over through one in `init` is handed again once the global says it is wired (`BreadcrumbFit.wired`).
- A property named like a built-in one cannot be declared (`row` in a layout child, `padding` on a component):
  pick another name (`inset`).
- Strings have no `contains`, prefix test or regex, and functions have no loops and no recursion: see «Screens and
  Rust».
- `Text` has no line height to set: a text stepping by lines uses the font's own line, measured by an undrawn `Text`
  in the same font (`Label`'s `line`). A literal `text:` that is never drawn still trips the `untranslated` rule:
  give it a named property.
- A `PopupWindow` takes the keyboard focus when it shows and gives it back when it closes, and while it is open
  the next pointer move goes to the popup layer: never put something that must not take the focus (a tooltip,
  a badge, a hint) on one; draw it in `DomeWindow`'s top layer (`TooltipLayer`).
- Inside a `PopupWindow`, `absolute-position` is relative to the popup, not the window: add the popup's own
  `x` / `y` (and its parent's `absolute-position`) to get window coordinates.
- Properties of elements inside a `PopupWindow` cannot be read from outside it: keep a popup's state on the
  component that owns it and let the popup read it.
- `accessible-role` must be a constant: an element that is sometimes an item and sometimes a title gets one
  `if` element per role.
- A size computed from itself through a layout is a binding loop, and some loops crash the compiler with a stack
  overflow instead of an error (a width that depends on whether a scroll bar shows): lay the scroll bar over the
  content; a row whose height must not follow its text gets a fixed `preferred-height` and its inner layout an
  explicit `x` / `y`.
- A text or a size that depends on an `out` computed from the layout it sits in (a label's `cut`) is a binding
  loop: show it without changing the layout (`opacity`, a colour).
- Slint animates a change only from a value it has already read: a property first read when drawn (an
  `opacity`) and changed in `init` appears at once. Read the start value in `init` before changing it
  (`PopupSurface`).
- The software renderer puts things on whole pixels: a move of a few pixels runs `linear`, or eased it jumps.
- The software renderer draws no rotation and no scale (`rotate` and `scale` are empty, `// TODO (#6068)` in
  `i-slint-renderer-software`): `transform-rotation` and `transform-scale-*` do nothing on screen. A chevron that
  turns is two icons; a growing element changes its size.
- Off-screen shots do not advance time: animations and `Timer`s stay where they started, so a colour caught
  mid-animation in a shot is not a bug.
- `Text` has no `selectable` in 1.19: only `StyledText` has it, without font weight, letter spacing, wrap modes or
  elision. Selectable text is a read-only `TextInput` with no caret (`SelectableLabel`); it cannot elide.
- A `FocusScope` takes the focus on a click unless `focus-on-click: false`: an area that wraps content (the
  `ContextMenu`) sets it, or its right click would take the focus from a selectable text and clear its selection.
- Segoe UI Semibold (600) has proportional digits; 400 and 700 are tabular, so the design uses only those (§ 3.3).
- A large window built in a debug build needs a deep stack: the Kit overflowed Windows' 1 MB main thread while the
  off-screen tests (2 MB test threads) passed. `crates/ui/build.rs` links the binary with an 8 MB stack on MSVC; a
  test passing off screen does not prove the window opens.

## Checklist before «done»

1. The behaviour was listed against Radix / shadcn on the card, and every open design value is a `Pending design`
   token named on the card.
2. The spec (`docs/design-system.md` § 5) has the component's interface table; `README.md` of Dome UI and
   `docs/decisions.md` follow any change of fact or decision.
3. The Kit shows every variant and state, working; `HEIGHT` covers it; the shots were looked at in dark / normal
   and light / compact.
4. A test window and a headless test prove each behaviour, with expectations from the spec, waits by the clock,
   and one mutation that made it fail.
5. `./scripts/check.ps1` passes (format, Slint format, clippy, every test, licences).
6. After the merge into `develop` and the push, its CI run is watched to the end, macOS included; a red run is
   fixed before the next card.
7. The card says what was done, what was measured, what waits for the owner.
