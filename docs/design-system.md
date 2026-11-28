# Design system

Step 4 of the plan. This is DiskDome's design system: the principles, the main window, every token with its
value, the data colours, the components with their variants and states, the patterns, the Kit window and the
automatic checks. It is written to be implemented in Slint without further design work.

**Status: decided by the owner on 2026-11-24, after the design sessions** (§ 11 lists what was decided). Every
value below is concrete. The few points still waiting are marked **⟨open⟩** and listed in § 11. The reference mockup is the Design
canvas «DiskDome — design system» (https://claude.ai/artifact/9uEkqjsB82zrQAAr9x97CD): the main window at real
size and interactive (dark / light, normal / compact), the accent
choice, the map styles (with «Folders» on a large window), the full and minimum windows, the Overview and the
other new views (§ 4.5), Statistics, the charts and the features (§ 4.6–4.8), the flows, every control, its states and the errors (§ 6.11–6.14), the
components, the data colours, the deletion dialogs and the first launch. Its values come from the
same source as the tables below.

Sources: the owner's other projects (EmuDome's structure, tokens and checks; NoteDome's principles; rapportino's
one-scale density; GameDome's `tokens.json`), `docs/features.md`, `docs/architecture.md`, and Slint 1.19's
documented capabilities (see «Implementation in Slint»).

How to read the tables: sizes are logical pixels (`px` in Slint: they follow the Windows scale factor, so
22 px is 33 physical pixels at 150 %). Colours are sRGB hex; where a colour is made from another one with
transparency, the hex is the result over its usual background, and the alpha form is given too.

## 1. Principles

1. **The data is the colour.** The interface is neutral graphite; colour comes from the files (the treemap and
   the category swatches). The accent never appears inside the data area.
2. **Dense, never cramped.** 22 px rows, 24 / 28 / 32 px controls, no empty bands, no decorative padding. Every
   pixel shows data or helps to act on it. Density is a setting (normal / compact) from one scale.
3. **Hierarchy by lightness, not by colour.** A darker ground sits deeper: the window chrome (toolbar, status
   bar, headers) is the deepest, the content panels one step up, what floats (menus, dialogs, tooltips) one step
   more.
4. **Subtle layers.** A floating element is a slightly lighter fill, a 1 px edge and a soft shadow. Panels
   never float: they are separated by a 1 px line.
5. **Typography as structure.** Groups are made with type (an overline label, a heading weight, ink vs muted
   ink), not with boxes and dividers.
6. **The accent in small doses.** The main action of a view, the selection, the focus, the active state. One
   primary button per view at most.
7. **Numbers are first-class.** Right-aligned, tabular figures, the same precision in a column, the unit in its
   own slot. A number that changes never makes its neighbours move.
8. **Safety is visible.** Destructive actions are red, use words (never an icon alone), say what will happen
   and ask for confirmation; system files carry a shield and ask twice (`CLAUDE.md`, rule 3).
9. **Keyboard first, Windows-native.** Everything works from the keyboard with Windows' own shortcuts and
   names; the native title bar, the system font, Explorer's verbs and their order.
10. **Speed is felt.** Interactions answer in one frame; motion is short (≤ 200 ms) and never on the data:
    sorting, zooming and redrawing the treemap are instant.

## 2. The main window

### 2.1 Anatomy

```
+------------------------------------------------------------------------------------------+
| (D) [C: Windows ==] [D: Dati ==] [E: USB] [emu-dome x] [+]                     _  []  X  | title bar with drive tabs
+------------------------------------------------------------------------------------------+
| [<] [Analizza di nuovo] | [Cerca un file o una cartella  Aa .*] [Filtri]                   [S] [...] | toolbar
+------------------------------------------------------------------------------------------+
| Panoramica  Esplora  Pulizia  Timeline  Statistiche  Attivita  670 GB  usati . 284 liberi | section bar
+----------------------------------------------------------+-------------------------------+
| [Albero|Elenco file] [Tabella|Entrambi|Mappa] [P]     # Data 115 GB  [Mostra] [Cestino] [...] | Explore bar
+----------------------------------------------------------+-------------------------------+
| Nome                 Dimensione   %          File  Modif. | # .pak    119 GB  17,8% 1.503 |
| v C:\                   670 GB  ===== 100%      ...       | # .vhdx  53,3 GB   8,0% 2.498 |
|   > Users               310 GB  ===   46,4%     ...       | # .dll   44,1 GB   6,6%  275 k |
|   > Program Files (x86) 123 GB  =     18,3%     ...       | ...                           |
+-------------------------------- splitter ----------------+-------------------------------+
| [home][up] C: > Users > stefa    docker_data.vhdx . 48,5 GB    [x] Spazio libero  [...] | map header
| +-------------+-----------+------+                                                       |
| |             |           |      |   map (treemap), full width                           |
| +-------------+-----------+------+                                                       |
+------------------------------------------------------------------------------------------+
| 3.789.066 file . analizzato oggi alle 21:14 | 1 selezionato . 48,5 GB  | o in tempo reale | GB  | status bar
+------------------------------------------------------------------------------------------+
```

Window: default 1280 × 800, minimum 960 × 600, size and splitter positions remembered.

| Region | Height normal / compact | Ground | Content |
| --- | --- | --- | --- |
| Title bar | 32 | `frame` (a step deeper than `bg`) | app mark, **drive tabs** (§ 5.27) and «+», drag area, caption buttons; the active tab takes `bg` and joins the toolbar. the tabs sit in the title bar (owner, 2026-11-24), if the custom title bar spike passes (§ 12, test 1); otherwise in the toolbar |
| Toolbar | 36 / 32 | `bg` | only what works everywhere: Back; «Scan again» (secondary; before the first scan the call to action is in the middle, § 2.2) or «Stop»; search («Search for a file or folder», case and regex toggles) and Filters once the drive is scanned; Settings; «⋯», the app menu (Export…, Command palette, Keyboard shortcuts, Settings, Help, Report a problem, About DiskDome; § 6.19) |
| Filter bar | 28 / 24, only when a filter is active | `bg` | filter chips, «Clear filters» |
| Info bar | 32 / 28, only when there is a message | state soft fill | one message with its action (scan mode, errors) |
| Section bar | 28 / 24, once the drive is scanned | `bg` | the six sections (§ 6.11) left; the drive's summary right, a link to Statistics |
| Explore bar | 28 / 24, Explore only, full width | `bg` | «Tree · File list», the layout «Table · Both · Map», the File types panel toggle; on the right the **selection**: swatch, name, size and «Show», «Move to the Recycle Bin», «⋯» (the right-click menu), or with nothing selected «Select a file or folder to see what you can do with it» in `ink-faint` |
| Content | rest | `surface` | the current section; in Explore the table and the side panel |
| Splitter | 1 (6 to the pointer) | `line` | resizes table vs treemap |
| Map header | 28 / 24 | `bg` | home / up, breadcrumb of the zoom root; on the right the hovered item (or the zoom root's summary), the «Free space» checkbox and the options menu (style, names, shape and colours, § 4.5) |
| Map (treemap) | ≥ 120 | `bg` | the treemap, full width |
| Status bar | 22 / 20 | `bg` | scan summary · selection · live state · scan mode · units |

The side panel (extensions / categories) sits on the right of the table, 300 px by default (220–480), with
its own header at the tab bar's height. Both splitters are remembered; the side panel can be hidden from the
toolbar and with Ctrl+Alt+P.

**Layouts of Explore** (owner, 2026-11-24: «give all the space to the squares, or only to the folders, so it is easy for
anyone»): a `SegmentedControl` with icon and word in Explore's bar, remembered:

| View | Shows | Use |
| --- | --- | --- |
| **Table** (Tabella) | Explore's bar, table, side panel; no map | lists, numbers, keyboard work |
| **Both** (Entrambi) | the table above, the map full width under it: 42 % of the content height (header included) until the user moves the splitter, then the splitter is remembered | the default |
| **Map** (Mappa) | the map fills the content beside the side panel (its legend); no table | the most intuitive view for someone who has never used a disk analyzer |

Shortcuts Ctrl+Alt+1 / 2 / 3. «Map» is the view to try first with non-technical people (§ 12).

What each window size gets (owner, 2026-11-24: «a full window and a reduced one, as examples»; computed by the
mockup's layout, artboards «Main», «MappaFull», «MappaRidotta»). Names counted with «Names: Many» (§ 4.4) on the
mockup's C: drive.

| Window | Density | «Both»: table rows · map | Names in «Both» | «Map»: map beside the side panel | Names in «Map» |
| --- | --- | --- | --- | --- | --- |
| 1920 × 1032 (maximized at 1080p, taskbar visible) | normal | 22 · 1920 × 344 | 94 | 1619 × 857 | 206 |
| | compact | 27 · 1920 × 358 | 98 | 1619 × 875 | 217 |
| 1280 × 800 (default) | normal | 16 · 1280 × 247 | 62 | 979 × 625 | 94 |
| | compact | 20 · 1280 × 261 | 62 | 979 × 643 | 100 |
| 960 × 600 (minimum) | normal | 10 · 960 × 163 | 34 | 659 × 425 | 56 |
| | compact | 13 · 960 × 177 | 30 | 659 × 443 | 55 |

(With the section bar and Explore's full-width bar of § 6.11.)

Below 1100 px of width the search field narrows to 200 px; above 1600 px it grows to 400 px. Nothing else
changes with the size: the same regions, more rows and a larger map.

### 2.2 Window states

| State | What the window shows |
| --- | --- |
| **Drive not scanned yet** (its tab is open) | **one call to action**, centred, 560 px wide: drive icon 32, «Find out what takes up space on E:» (`display`), label · file system · used of total (from Windows, no scan) and a 6 px usage bar; the **hero** button «Scan E:» (`primary`, 40 px, § 5.2), the only primary on screen, with «It only reads: nothing is changed.» beside it; one line on the scan mode with «Change» (§ 6.7); then «Other drives» (one row each, «Scan» or «Open» secondary) and «Scan a folder…» with the drop target. No section bar; the toolbar shows only Settings and «⋯». The scan shows its progress in place, with «Stop» |
| **Empty** (no scan yet) | the **Start** tab (§ 6.16): «What do you want to analyze?», every drive of this PC as a card with Windows' numbers, the recommended one with the only primary button, «Choose a folder…», «Network path…», the drop target, the scan mode line and «How it works» in three steps (artboard «Inizio») |
| **First launch** | the empty state above, no dialog (§ 6.7, proposal) |
| **Scanning** | a drive never scanned: the content becomes the **progress view** (artboard «Analisi»): «Analysis of Windows (C:)» and the percentage, an 8 px bar, what it is doing now («Reading the drive's index: 8.2 of 13.7 GB») with the time left on the right, three counters growing (files found, folders, space counted), the phases each with its state and its own bar, «You can keep using the PC. At the end C:'s Overview opens.», «Stop». A drive already scanned: its last scan stays on screen, a 2 px progress line under the toolbar and the same words in the status bar. With a folder walk Explore fills while it reads |
| **Ready** | everything shown; the status bar says when the drive was scanned («scanned today at 21:14») |
| **Live** | the status bar's dot is `success` with «Live»; an outside change updates rows in place, no animation |
| **Live updates off or paused** | dot `ink-faint` with what applies: «Live updates paused» (by the user, or the journal overflowed), «Live updates off» (Settings), «No live updates» (not NTFS, network path, folder walk, a folder scanned apart); a click opens a popover with the reason and its one action (§ 6.17) |
| **Scanning another drive** | the current tab stays usable; the scanning tab shows a progress ring instead of its icon and the status bar lists «Analysis of E:» among the running operations, with «Stop» (§ 5.29, § 6.17) |
| **Scan stopped** | folder walk: what was read stays, with a `warning` info bar «Analysis of E: stopped at 42%: you see what was read, totals are lower than real.» and «Resume»; drive's index: the previous scan stays (§ 6.17) |
| **Error** | an info bar (danger) under the toolbar with what happened and what to do; the previous scan stays visible if there is one |

## 3. Tokens

### 3.1 How tokens work in Slint

- Tokens are Slint globals, typed: `length`, `color`, `brush`, `duration`, `int`. A wrong token does not compile.
- Families, one file each in `crates/dome-ui/ui/tokens/`:
  `theme.slint` (`global Theme`: colours per theme), `data.slint` (`global Data`: file-category colours),
  `size.slint` (`global Size`: spacing, sizes, radii, borders; density), `type.slint` (`global Type`: font and
  the type steps), `motion.slint` (`global Motion`: durations and the easing).
- Run-time switches, set by Rust from the settings: `Theme.mode: ThemeMode` (`dark`, `light`; also
  follows the OS when the setting is «System»), `Size.compact: bool`, `Data.palette: DataPalette` and
  `Data.style: MapStyle` (§ 4.4). Every value that depends on them is a conditional binding in
  the token file, so switching re-renders without restarting.
- Names say the role (`row`, `control`, `panel-pad`, `surface`, `ink-muted`), not the value. A grid step has a
  numeric name only in the spacing scale (`space-8`).
- A value nobody has decided yet is still a token, under a `// Pending design` comment with what it is for. A
  literal in a component or a screen is never the way.
- Variants are `enum`s (`ButtonVariant.primary`), never strings.
- The accent is DiskDome's own, **Laguna** (owner, 2026-11-24); there is no user accent setting in v1.

### 3.2 Colour

#### Neutrals (graphite)

A cool graphite, hue 255° in OKLCH with almost no chroma (0.005), so the file colours of the treemap stay the
only colour on screen. Dark is the default theme; the light themes follow the same roles. **The light theme is a
soft grey, never near-white** (owner, 2026-11-24: the first light theme, white surfaces at L 0.995, «hurts the
eyes»): surfaces at L 0.94, the window ground at L 0.905. **Two themes, Dark and Light** (owner, 2026-11-24); the
dimmer «Light dim» tried in the mockup was dropped. `Theme.mode: ThemeMode { dark, light }`.

| Token | Dark | Light | Role | `ink` on it (dark / light) |
| --- | --- | --- | --- | --- |
| `bg` | `#0f1113` | `#dee0e2` | deepest ground: toolbar, tab bar, headers, treemap area, status bar | 15.7 / 12.9 |
| `surface` | `#161719` | `#eaebed` | content panels: tables, side panel | 14.9 / 14.3 |
| `raised` | `#1f2123` | `#f2f3f5` | floating: menus, tooltips, dialogs, toasts | 13.4 / 15.4 |
| `field` | `#1c1e20` | `#f1f2f4` | text fields | 13.9 / 15.3 |
| `control` | `#232528` | `#d7d9dc` | secondary button, segmented control, chips | 12.7 / 12.1 |
| `control-hover` | `#2c2e31` | `#cdd0d3` | hover of controls and menu items | 11.3 / 11.0 |
| `control-pressed` | `#1a1c1f` | `#c3c6ca` | pressed controls | 14.2 / 10.0 |
| `line` | `#282a2d` | `#cbced1` | 1 px separators between panels, field edges |  |
| `line-strong` | `#6b6e73` | `#717579` | checkbox and radio boxes (≥ 3:1 on every ground at rest: `bg`, `surface`, `raised`, `field`, `control`, `hover`) |  |
| `edge` | `#35383c` | `#bbbec2` | edge of popups and dialogs |  |
| `ink` | `#e8eaec` | `#191c20` | text, icons that act | on surface 14.9 / 14.3 |
| `ink-muted` | `#abaeb3` | `#494d54` | secondary text, column headers, status bar, folder icons | on surface 8.1 / 7.1 |
| `ink-faint` | `#7e8288` | `#65696f` | placeholders, disabled text, missing-value dash | on surface 4.6 / 4.6 |

#### Accent: Laguna (decided 2026-11-24)

The accent is a new Dome colour: a hue no other Dome app uses. The family, by OKLCH hue:

| App | Primary | OKLCH hue |
| --- | --- | --- |
| IsaacDome | `#901800` deep red | 33° |
| PokeDome | `#f77245` coral | 39° |
| EmuDome | `#faa40f` amber | 71° |
| PostDome | `#d4a84b` Dome gold | 84° |
| SoundDome | `#1db954` green | 149° |
| domehub | `#47c2ff` sky blue | 235° |
| GameDome | `#7c5cff` violet | 286° |
| NoteDome | `#a57bd4` amethyst | 305° |

Three gaps of ~65–90° were free: yellow-green (~117°), teal (~192°) and pink-magenta (~349°). **Laguna**, a
cyan-teal at 190°, takes the widest one: calm and technical («storage, system tool»), far from the red of deletion
and the amber of warnings; its closest Dome neighbours are SoundDome (41° away) and domehub (45°). Text on it and
the accent as text pass WCAG AA in both themes:

| Accent | Theme | `accent` | `accent-hover` | `accent-pressed` | `on-accent` | `accent-text` | `accent-soft` | `focus` | on-accent / accent | accent-text / surface |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Laguna (190°) | dark | `#37c2bb` | `#4dd2cb` | `#12aea8` | `#0f1113` | `#37c2bb` | `#37c2bb38` | `#37c2bb` | 8.6 | 8.2 |
| Laguna (190°) | light | `#007c79` | `#006f6d` | `#006360` | `#ffffff` | `#006663` | `#007c7929` | `#006663` | 5.0 | 5.7 |

Lightness (owner, 2026-11-24): the first Laguna (L 0.80, `#3dd7cf`) was too bright; the accent is one step darker,
**L 0.74, C 0.115 (`#37c2bb`)** on the dark theme; the light theme keeps `#007c79`.

Compared on the mockup and not chosen: Volt (yellow-green, 118°: the tightest gap, 31° from SoundDome) and Fucsia
(pink, 350°: next to `danger`, which matters in an app that deletes files).

The app icon takes the accent: the «Pie» mark (owner, 2026-11-25, `docs/decisions.md`), drawn in the title bar at
16 px on every artboard. The two data categories nearest to it moved away (§ 3.2 «Data»): Dati to 176° and
darker, Programmi to 222°.
Accent roles: `accent` (fill of the primary button, switch on, progress), `accent-hover`, `accent-pressed`,
`on-accent` (text and icons on the fill), `accent-text` (accent as text or icon on a surface: links, active tab
text, sort arrow), `accent-soft` (selection fill of rows, chosen segment), `focus` (the focus ring).

#### States

| Token | Dark | Light | Use |
| --- | --- | --- | --- |
| `danger` | `#f66d67` (6.6:1 on bg) | `#b32322` (5.0:1) | deletion, errors, invalid input |
| `danger-soft` | `#f66d6724` | `#b3232224` | fill of danger info bars and badges |
| `warning` | `#f6b84d` (10.7:1 on bg) | `#8a5100` (4.9:1) | system items, permission and space warnings |
| `warning-soft` | `#f6b84d24` | `#8a510024` | fill of warning info bars and badges |
| `success` | `#5fcf83` (9.7:1 on bg) | `#10703a` (4.7:1) | live updates, completed actions |
| `success-soft` | `#5fcf8324` | `#10703a24` | fill of success info bars and badges |

A state colour always comes with an icon or a word (never colour alone). The `*-soft` fills are the state at
14 % (`24` alpha), for info bars and badges.

#### Data: file categories

The treemap, the swatches of the side panel and the file icons in the tables are coloured by **category**
(owner's decision, 2026-11-24). Twelve families, each one hue, alternating lightness so neighbours on the
colour wheel differ in lightness too (minimum distance between two categories: ΔE OK ≥ 0.096). The same
colours in both themes: a file's colour never changes with the theme.

| Category (it / en) | Enum | Base | Lighter | Darker | `on-data` | OKLCH (L C h) | Icon | Extensions (first version) |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Video / Video | `video` | `#d9704f` | `#f18664` | `#c15a3a` | `#16181b` (5.4) | 0.66 0.140 38 | `file-video` | mp4 mkv mov avi wmv webm m4v mpg ts flv |
| Immagini / Images | `image` | `#e68ebb` | `#fea4d2` | `#ce78a5` | `#16181b` (7.6) | 0.75 0.120 348 | `file-image` | jpg jpeg png gif webp heic avif tif tiff bmp psd raw cr2 nef arw dng svg ico |
| Audio / Audio | `audio` | `#a275c9` | `#b88ae0` | `#8d60b2` | `#16181b` (5.0) | 0.64 0.130 308 | `file-audio` | mp3 flac wav aac ogg opus m4a wma aiff |
| Documenti / Documents | `document` | `#5a86ce` | `#6f9ce5` | `#4671b7` | `#16181b` (4.9) | 0.62 0.120 260 | `file-text` | pdf doc docx xls xlsx ppt pptx odt ods odp txt md rtf epub csv |
| Giochi / Game data | `game` | `#5f53ae` | `#7268c5` | `#4c3e97` | `#ffffff` (6.3) | 0.50 0.140 286 | `gamepad-2` | pak vpk bundle assets uasset ucas utoc bsa ba2 forge wad |
| Programmi / Programs | `app` | `#4caccb` | `#64c3e2` | `#3296b4` | `#16181b` (6.8) | 0.70 0.100 222 | `app-window` | exe dll msi msix appx sys drv ocx efi so dylib |
| Dati / Data | `data` | `#197a67` | `#358f7b` | `#006453` | `#ffffff` (5.2) | 0.52 0.090 176 | `database` | db sqlite mdf ldf pst ost dat bin log dmp bak tmp cache etl evtx |
| Codice / Source code | `code` | `#5cb572` | `#73cb87` | `#459f5d` | `#16181b` (7.0) | 0.70 0.130 150 | `file-code` | js mjs cjs ts tsx jsx json map rs py c cpp h cs java go rb php css scss html vue toml yaml lock |
| Compilati / Build output | `build` | `#788a37` | `#8d9f4d` | `#647520` | `#16181b` (4.6) | 0.60 0.110 120 | `package` | o obj rlib rmeta pdb lib a d class pyc jar wasm node |
| Archivi / Archives | `archive` | `#d1a255` | `#e8b86b` | `#ba8c3e` | `#16181b` (7.6) | 0.74 0.110 78 | `file-archive` | zip 7z rar tar gz xz zst bz2 cab iso img vhd vhdx vmdk wim esd |
| Sistema / System | `system` | `#7d6e63` | `#928377` | `#695b50` | `#ffffff` (4.9) | 0.55 0.025 60 | `cpu` | pagefile.sys hiberfil.sys swapfile.sys $MFT |
| Altro / Other | `other` | `#55585d` | `#696c70` | `#424549` | `#ffffff` (7.1) | 0.46 0.008 255 | `file-question` | (everything else, no extension) |

- **Palettes** (owner, 2026-11-24: all three ship): besides the standard colours, two variants as a setting («Colours of the map»):
  **Soft** (Tenue: every colour 42 % toward its own grey, luminance kept) and **Vivid** (Vivace: 30 % away from
  it). `on-data` is recomputed for each colour (dark ink where it reaches 4.5:1, else white). The transform is
  applied by the treemap crate and by `Data` in Slint from the same numbers.
- **Shades per extension.** Within a category, the three largest extensions of the scan get the base colour,
  a lighter (+0.07 L) and a darker (−0.07 L) shade, in that order; the others take the base. The side panel
  shows the exact shade of each extension.
- **`on-data`**: the text drawn on a category colour (treemap labels): dark ink `#16181b` where it reaches
  4.5:1, white otherwise (Giochi, Dati, Sistema, Altro).
- **Where a category comes from.** `diskdome-core` maps the lower-case extension to a `FileCategory` (the
  table above is its first version; the full map lives in `core` with tests); a few names win over the
  extension (`pagefile.sys`, `hiberfil.sys`, `swapfile.sys`, `$MFT` and the other NTFS metafiles → System).
  Folders have no category: a folder's colour in the treemap is its files'. The enum is shared with Slint
  (`export enum FileCategory` in `data.slint`), so Rust and the interface use the same names.
- Accent: no category sits closer than ΔE OK 0.12 to the accent (Programmi, the nearest, is 32° and a step darker
  away), and the accent is never drawn inside the data area.

#### Data: age

For «Colours: Age» and the Timeline section (§ 4.5): one hue (58°, amber-brown, far from the accent), **bright = recent,
dim = old**, six buckets by the file's last-modified date. Lightness falls 0.08–0.10 per step, so the order reads
without the legend and in grey-scale; the same colours in every theme, like the categories.

| Token | Bucket (it / en): last modified within | Colour | `on-data` | OKLCH (L C h) |
| --- | --- | --- | --- | --- |
| `age-d7` | 7 giorni / 7 days | `#ffb984` | `#16181b` (10.6) | 0.84 0.150 58 |
| `age-m1` | 1 mese / 1 month | `#e79551` | `#16181b` (7.5) | 0.74 0.130 58 |
| `age-m6` | 6 mesi / 6 months | `#ba7c4c` | `#16181b` (5.1) | 0.64 0.100 58 |
| `age-y1` | 1 anno / 1 year | `#8e6445` | `#ffffff` (5.2) | 0.54 0.070 58 |
| `age-y3` | 3 anni / 3 years | `#664d3a` | `#ffffff` (7.8) | 0.44 0.045 58 |
| `age-older` | più vecchi / older | `#453b33` | `#ffffff` (10.9) | 0.36 0.020 58 |

#### Other colour tokens

| Token | Dark | Light | Use |
| --- | --- | --- | --- |
| `hover` | `#222325` | `#e0e1e3` | row hover (ink 5.5 % / 4.5 % over surface) |
| `selected-inactive` | `#2b2c2e` | `#d7d8db` | selected rows when the table has no focus (ink 10 % / 8 %) |
| `bar-track` | `#2b2c2e` | `#d5d6d9` | percent and usage bar track |
| `bar-fill` | `#6e7072` | `#96989b` | percent and usage bar fill (neutral) |
| `scroll-thumb` | `#e8eaec3d` | `#191c2038` | scrollbar thumb (ink at 24 % / 22 %) |
| `scroll-thumb-hover` | `#e8eaec66` | `#191c205c` | scrollbar thumb, hovered or dragged |
| `scrim` | `#0000008c` | `#1a1c2059` | over the window behind a dialog |
| `shadow` | `#00000099` | `#1a1c2033` | drop shadows of levels 1 and 2 |
| `data-gap` | = `bg` | = `bg` | the gaps between treemap folders (3, 2, 1 px) |
| `data-hover` | = `ink` | = `ink` | 1 px hover outline in the treemap |
| `data-selection` | = `ink` + `bg` halo | = `ink` + `bg` halo | 2 px selection outline in the treemap |

### 3.3 Typography

**Font: Segoe UI** (Windows' system font; nothing bundled, nothing to license). Its default figures are
tabular (all digits the same width), which matters because Slint's `Text` has no OpenType feature switch
(no `tnum`): the font's default figures are what the columns get. **The design uses 400 and 700 only**
(decision 8). Measured in the Kit and in the font files (test 7): Segoe UI 400 and 700 have
tabular digits, Semibold 600 does not (its «1» is 28 % narrower), so a bold total in 600 would not line up with
the rows above it. Segoe UI Variable is out: its «1» is narrow at every weight, and Slint 1.19 does not apply its
«Text» and «Small» optical sizes.

macOS (after v1): SF Pro's default figures are proportional, so numbers there will need a font with tabular
defaults; decided when the macOS port starts.

| Step | Size / line | Weight | Tracking | Use |
| --- | --- | --- | --- | --- |
| `display` | 20 / 26 | 700 | -0.2 px | empty states, first-run title |
| `title` | 15 / 20 | 700 | 0 | dialog and panel titles |
| `body` | 13 / 18 | 400 | 0 | default text, table cells (normal) |
| `body-strong` | 13 / 18 | 700 | 0 | primary button, emphasised values |
| `caption` | 12 / 16 | 400 | 0 | secondary text, column headers, status bar, table cells (compact) |
| `caption-strong` | 12 / 16 | 700 | 0 | treemap name + size labels, badge text |
| `map-name` | 11 / 13 | 700 | 0 | treemap names on small rectangles (Names: Many, All); map only |
| `map-name-xs` | 10 / 11 | 700 | 0 | treemap names on the smallest rectangles (Names: All); map only |
| `overline` | 11 / 14 | 700 | +0.6 px | uppercase section labels, two words at most |
| `mono` | 12 / 16 | 400 | 0 | regular expressions, hashes (Cascadia Mono, Consolas) |

- One `Label` component draws every text (§ 5.1): it takes a `TypeStep` and a colour role; nothing else sets
  `font-size`, `font-weight` or `font-family`.
- `map-name` (11 px) and `map-name-xs` (10 px) exist **only for names inside the map** (§ 4.4): there a name is a
  bonus on top of the hover line, which always shows the full name at `caption`. Nowhere else does text go below
  `overline`.
- Line height is the step's: a single-line `Label` is exactly that tall, text centred vertically.
- **Truncation.** Names and paths are elided, never wrapped, in tables and bars: names at the end («progetto_fin…»),
  paths in the middle («C:\Users\stefa\…\node_modules»); the full text is in the tooltip. Wrapping only in
  dialogs, info bars, toasts and empty states.
- **Italian is ~25 % longer than English**: no fixed widths for text; buttons hug their label with a minimum
  width (§ 5.2); columns are resizable.
- Text rendering: grey-scale anti-aliasing (Slint does not do ClearType), drawn by Slint's **software renderer**
  (decision 9, test 7). Compared in the Kit with femtovg and Skia: Skia draws light text on dark fuller,
  femtovg and software alike draw it thinner; software uses a fraction of the memory (15 MB private against
  femtovg's 104 and Skia's 453, debug builds) and no GPU.

### 3.4 Spacing

Grid: `space-0 2 4 6 8 10 12 16 20 24 32 48`. 10 is kept (EmuDome uses it for card padding); nothing else off
the grid.

Role tokens (density-dependent ones in § 3.5):

| Token | Value | Use |
| --- | --- | --- |
| `card-pad` | 10 | inside a card (drive cards, clean-up groups) |
| `dialog-pad` | 16 | inside a dialog; also its gap between title, body and footer |
| `dialog-gap` | 12 | between blocks of a dialog's body |
| `menu-pad` | 4 | inside a menu, around its items |
| `item-pad-x` | 8 | a menu or list item's side padding |
| `field-pad-x` | 6 | a field's side padding |
| `button-pad-x` | 10 | a button's side padding with text (8 with an icon on that side) |
| `label-gap` | 6 | icon ↔ text inside a control or a row |
| `group-gap` | 8 | between groups of the toolbar (with a separator in the middle) |
| `section-gap` | 16 | between sections of a page (settings, empty state) |
| `tooltip-pad` | 4 × 8 | inside a tooltip (vertical × horizontal) |

### 3.5 Sizes and density

One scale, two densities. Normal is the owner's compact baseline (EmuDome, 2026-11-22: «giant buttons, make
everything more compact»); compact is WizTree's density, for power users. Dialogs and their controls keep the
normal sizes in both densities (they are read, not scanned).

| Token | Normal | Compact | Use |
| --- | --- | --- | --- |
| `control-sm` | 24 | 20 | small control: actions in the treemap and side panel headers, chips, row actions |
| `control` | 28 | 24 | default control: buttons, fields, selects, segmented |
| `control-lg` | 32 | 28 | large control: dialog main actions, empty-state actions |
| `row` | 22 | 18 | table and list row |
| `header-row` | 24 | 20 | table column header |
| `toolbar` | 36 | 32 | main toolbar (control + 2 x space-4) |
| `tabbar` | 28 | 24 | view tabs bar, side panel header, treemap header |
| `statusbar` | 22 | 20 | status bar |
| `menu-item` | 24 | 22 | menu and select list item |
| `icon` | 16 | 14 | icon in controls and rows |
| `indent` | 16 | 14 | tree indentation per level |
| `cell-pad-x` | 6 | 4 | table cell side padding |
| `panel-pad` | 8 | 6 | panel and toolbar padding |
| `tool-gap` | 4 | 2 | gap between toolbar controls |
| `bar-h` | 6 | 4 | percent bar height in a row |
| table text | `body` 13 / 18 | `caption` 12 / 16 | cells of every table |

Fixed sizes:

| Token | Value | Use |
| --- | --- | --- |
| `side-panel` | 300 | side panel default width |
| `side-panel-min` | 220 | side panel minimum |
| `side-panel-max` | 480 | side panel maximum |
| `treemap-min` | 120 | treemap minimum height |
| `table-min` | 120 | table area minimum height |
| `splitter-hit` | 6 | splitter hit area (the line is 1) |
| `scrollbar` | 10 | scrollbar area |
| `scroll-thumb` | 6 | scrollbar thumb (8 hovered) |
| `swatch` | 10 | category swatch (square) |
| `bar-w` | 56 | percent bar width |
| `search-w` | 320 | search field in the toolbar (grows to 400) |
| `drive-picker-w` | 232 | drive picker |
| `dialog-sm` | 400 | small dialog width |
| `dialog-md` | 520 | medium dialog width |
| `dialog-lg` | 680 | large dialog width |
| `menu-min` | 200 | menu minimum width |
| `menu-max` | 360 | menu maximum width |
| `tooltip-max` | 320 | tooltip maximum width |
| `toast-w` | 360 | toast width |

### 3.6 Radii

Almost square, as in EmuDome (owner, 2026-11-22: «più squadrato», buttons «leggermente più rotondi»).

| Token | Value | Use |
| --- | --- | --- |
| `radius-badge` | 2 | badges, key caps |
| `radius-control` | 2 | fields, selects, checkboxes, chips, segmented |
| `radius-button` | 4 | buttons |
| `radius-card` | 3 | cards, option cards, drive cards |
| `radius-panel` | 4 | menus, tooltips, dialogs, toasts |
| `radius-swatch` | 2 | category swatches |
| `radius-bar` | 1 | percent and usage bars |

No pills. Round only where the shape is round by nature: the live-state dot, the switch's knob and track.

### 3.7 Borders and focus

| Token | Value | Use |
| --- | --- | --- |
| `border-hairline` | 1 | separators, field edges, popup edges |
| `border-focus` | 2 | focus ring |
| `border-selection` | 2 | treemap selection outline |
| `border-cursor` | 1 | cursor row frame in tables |

- **Focus ring**: 2 px `focus`, 1 px outside the control's edge, following its radius (+1). In a table the focus
  is the **cursor row**: a 1 px `focus` frame drawn inside the row, on top of the selection fill.
- The ring shows only while the user moves with the keyboard (Windows' own rule): a `InputMode.keyboard` flag, set by
  any navigation key and cleared by a pointer press, decides it.
- Edges are 1 px `line` between panels and around fields (`line-strong` for checkbox and radio boxes, which
  need 3:1); popups and dialogs use `edge`.

### 3.8 Elevation

| Level | What | Fill | Edge | Shadow (Slint `drop-shadow-*`) |
| --- | --- | --- | --- | --- |
| 0 | panels, toolbar, tables | `bg` / `surface` | 1 px `line` between panels | none |
| 1 | menus, select lists, tooltips, toasts | `raised` | 1 px `edge` | offset 0 4, blur 16, `shadow` |
| 2 | dialogs | `raised` | 1 px `edge` | offset 0 16, blur 48, `shadow`; `scrim` over the window |

In the dark theme shadows are kept light (NoteDome's lesson: heavy shadows become mud on near-black): the edge
does most of the work. No glass: Slint has no backdrop blur, and the dense interface does not want it.

### 3.9 Opacity

| Token | Value | Use |
| --- | --- | --- |
| `disabled` | 0.40 | a control that cannot be used now (with its reason in the tooltip) |
| `pending` | 0.55 | a row being deleted or moved, until the file system confirms |
| `dimmed` | 0.30 | treemap rectangles outside the current highlight (blended toward `bg`) |

### 3.10 Motion

| Token | Value | Use |
| --- | --- | --- |
| `fast` | 90 ms | hover and press colours, leaving |
| `base` | 140 ms | menus, select lists, tooltips appearing; progress line |
| `slow` | 200 ms | dialogs appearing |
| `tooltip-delay` | 500 ms | mouse rest before a tooltip (0 on keyboard focus) |

- Easing: `ease-out` = `cubic-bezier(0.2, 0.8, 0.2, 1)` (EmuDome's) for everything that enters; leaving is a
  plain fade at `fast`. The 4 px rise is the exception, `linear` (`rise-easing`): Slint's software renderer puts
  things on whole pixels, and eased the rise came in two jumps in the first frames, where the opening must be as
  fluid as possible; at an even pace it comes one pixel at a time under the fade.
- What moves: hover and press colours (`fast`); menus, select lists and tooltips appear with a fade and 4 px of
  rise (`base`); dialogs fade and grow from 98 % (`slow`); the scan progress line eases to its new value
  (`base`).
- What never moves: the data. Sorting, expanding, zooming the treemap, filtering and live updates are instant.
  No spring animations (Slint 1.18 has them; not here).
- **Reduced motion**: when Windows' «Animation effects» is off (`SPI_GETCLIENTAREAANIMATION`), Rust sets
  `Motion.reduced` and every duration is 0.

### 3.11 Layers and collisions

Nothing covers something else by accident (owner, 2026-11-25: «a popup that covers other popups or loading»). Five
layers, from the bottom. When two would meet, the higher one wins, and something the app shows on its own (a
notice, the end of an operation) never covers what you opened or are doing:

| Layer | What | How | Rule |
| --- | --- | --- | --- |
| Content | tables, map, panels, cards; inside it the sticky table header (1), the splitters' hit areas (2), the drag ghost (3) | `Size.z-*` tokens | — |
| Bars | title bar, toolbar, filter bar, info bars, section bar, Clean-up's action bar, status bar | in the layout | the notice never covers them; a popup covers the bars only around its own anchor while it is open (a menu opened from the toolbar, the command palette at the top); the title bar and the status bar are never covered except by a dialog's scrim (the status bar) |
| Notice | the toast (§ 5.20) | `Size.z-notice` (5) | one at a time, over content only; waits, hidden, while a popup or a dialog is open |
| Popup | menus, select lists, popovers (Filters, the status bar's popovers, «compared with…», hidden selection), search suggestions, tooltips | Slint `PopupWindow` | one at a time: opening one closes the other; it closes on a click outside, Esc, a dialog opening, typing in search, focus moving elsewhere |
| Dialog | the scrim and the dialog; Settings is a dialog | `Size.z-dialog` (10) | modal; a notice about what happens in Settings appears inside it, in its footer |

Windows' own windows (UAC, credentials, file pickers, its notifications) are outside the app: DiskDome waits and
says so (§ 6.17).

- **Long operations do not float.** Moving, deleting, comparing and scanning another drive sit in the status bar
  (§ 5.29), so the only thing that appears over content without being asked is the notice, which is short-lived,
  closable and never on a bar.
- **Placement.** Every popup and the notice stay inside the window, 8 px from every edge, never over the title bar
  or the status bar.
  A menu opens down from its anchor (the pointer or the button), aligned on the anchor's left edge, or on its right
  edge for buttons at the right of the window; if it does not fit above the status bar it opens upward; if neither
  fits it keeps 8 px at the top and at the bottom and scrolls. Popovers of the status bar open upward, 4 px above
  it, aligned on their item's right edge. The notice sits in the bottom-right corner, 16 px from the right edge
  and 16 px above the bar under it (the status bar, or Clean-up's action bar).
- **Esc** closes the top layer only: the dialog, then the popup, then the notice.
- **Narrow windows.** The bars give way before anything overlaps. In the status bar the texts on the left shorten
  first (ellipsis, the whole text in the tooltip), then the operation's label; its bar and percentage, «Live», the
  mode and the unit stay. In cards the values never wrap: labels and notes shorten (the whole text in the tooltip),
  charts scale down to the card's width, columns that are not the name hide before the name shortens below 140 px.
  The map header keeps the zoom's totals whole: the hint goes first, then the breadcrumb from its start (the folder
  you are in stays; the whole path is in the tooltip).

Checked on the screens by a test (§ 10, overlap check) in every state where two layers can meet.

### 3.12 Icons

- **Lucide** (ISC licence, compatible with GPL-3.0-or-later; listed in `THIRD_PARTY.md` with the first icon),
  the same set as EmuDome and NoteDome. SVG files in `crates/dome-ui/ui/icons/`, exposed as `global Icons { out
  property <image> folder: @image-url("folder.svg"); … }`, drawn by one `Icon` component that sets the size and
  tints with `colorize` (the colour role is a property, default `ink-muted`).
- Sizes: `icon` (16 / 14), `icon-sm` 12 (inside badges and chips). Lucide's 2 px stroke at 24 px renders 1.33 px
  at 16 px, matching Segoe UI's stems at 13 px.
- No emoji. Folder icons `ink-muted`; file icons tinted with their category colour; the shield of protected
  items `warning`.

| Icon | |
| --- | --- |
| Interface | `source` (one of `Icons`), `size` (`small` 12, `normal` `icon`, `hero` 18), `side` (any side; by default the token of `size`; Lucide's 24 grid softens a little at other sides), `tone`, `category` (for `data`: the category's own colour, a file's icon; for `on-data`: the ink on that colour), `label` |
| Accessibility | with a `label`, an image of that name; without one, decorative (beside a text that says it already), skipped by screen readers |
| Fading | a faded icon fades its container (one `opacity`), never its colour: Lucide's strokes overlap and a translucent tint darkens every joint |
| Direction | an icon does not turn: Slint 1.19's software renderer draws no rotation or scale, so a chevron that opens is two icons (`chevron-right`, `chevron-down`) |

- The set for v1 (Lucide names, to confirm against the pinned version): `hard-drive`, `folder`, `folder-open`,
  `file`, `file-video`, `file-image`, `file-audio`, `file-text`, `file-archive`, `file-code`, `package`
  (build output), `app-window`, `gamepad-2`, `database`, `cpu` (system), `file-question` (other), `play`,
  `square` (stop), `refresh-cw`, `search`, `case-sensitive`, `regex`, `filter`, `x`, `chevron-right`,
  `chevron-down`, `arrow-up`, `arrow-down`, `corner-left-up`, `house`, `ellipsis`, `panel-right`,
  `layout-dashboard`, `settings`, `external-link`, `folder-search`, `terminal`, `copy`, `clipboard-list`,
  `pencil`, `folder-input`, `trash-2`, `info`, `triangle-alert`, `circle-alert`, `shield-alert`, `recycle`
  (rebuildable), `link` (reparse point), `check`, `sun`, `moon`, `monitor`; for exporting (§ 6.18) `share`,
  `sheet` (CSV), `braces` (JSON), `file-text` (HTML report), `image` (the map as an image); for the app menu and the
  palette (§ 6.19) `command`, `keyboard`, `circle-help`, `bug`, `history`, `layers`, `bookmark`.

## 4. Showing the data

### 4.1 Numbers and units

- **Sizes**: three significant figures, the unit switching at 1000 of the current unit: `512 B`, `1,23 KB`,
  `12,3 MB`, `123 GB`, `1,02 TB` (Italian) / `1.23 KB` (English). Zero is `0 B`.
- **Units** (owner, 2026-11-24): a setting with three choices: **like File Explorer** (binary values labelled `KB MB
  GB TB`, the default: the same numbers the user sees in Windows, 931 GB for a «1 TB» drive), binary with honest
  labels (`KiB MiB GiB TiB`), decimal (`KB MB GB TB`, ×1000).
- **Size cell**: the number right-aligned and the unit in its own 24 px slot, left-aligned, `ink-muted`; so
  numbers align on their last digit whatever the unit.
- **Counts**: grouped by the interface language (`13.817.523` / `13,817,523`); in the narrow side panel and the
  status bar, counts ≥ 100 000 are shortened to three figures (`13,8 M`, `4,1 M`) with the full value in the
  tooltip.
- **Percentages**: one decimal (`42,7%` / `42.7%`), `<0,1%` for a non-zero value under 0.05 %, `100%` without
  decimals.
- **Dates**: the OS short date and 24-hour time (`24/11/2026 18:05` on an Italian Windows); ISO
  (`2026-11-24 18:05`) if the OS format cannot be read. Folders show their own modification time.
- **Missing values**: `—` in `ink-faint` (a folder's allocated size before the scan finishes, a date the file
  system does not have).
- **Attributes**: Explorer's letters in `caption`, `ink-muted`: R (read-only), H (hidden), S (system), A
  (archive), C (compressed), E (encrypted), L (reparse point: links, junctions, OneDrive placeholders), P (sparse);
  each letter spelled out in the tooltip («H: hidden · S: system»). No attribute: an empty cell.
- **Shown, not counted** (owner, 2026-11-25, card «Albero ed Elenco file»): folder totals, the map, the
  percentages and sorting by size count only the space on this drive, once. A size that is shown but not counted
  is in `ink-faint`: an OneDrive online-only file (its declared size; Allocated 0), the second and later names of a
  hard link (the clusters count once, on the first name, as WizTree). A link or junction shows `—`: its target is
  counted where it is. A folder whose content is all online shows its online size in `ink-faint`; a folder that
  also holds local files shows the local size, with the rest in the tooltip («Another 12.6 GB online only, not
  counted»).
- Formatting lives in the interface crate (Rust), from the structured values the engine sends; the engine
  never builds a sentence.

### 4.2 Percent bar

In the `%` column: a 56 px bar (`bar-h` tall: 6 / 4) with the number on its right. Track `bar-track`, fill
`bar-fill` (neutral: the accent is not for data). The bar shows the share of the **parent** folder.

### 4.3 Sorting

Default: size, largest first. A click on a header sorts by that column (numbers and dates descending first,
text ascending first), a second click reverses; Ctrl+Shift+1…9 does the same for the n-th visible column. The
sorted header is `ink` with a 12 px arrow (`accent-text`): before the label in right-aligned (number) columns,
after it in left-aligned ones, so the label stays aligned with the values; the others `ink-muted`.

- **Ties** sort by name. **Missing values** (a link has no size, a file no file count) go last in both directions.
- **Name**: folders first, then natural order like File Explorer («file2» before «file10»), case ignored.
- **Size** and **%** sort by the counted size (§ 4.1): online-only files and second hard-link names go to the
  bottom of a size sort, which is what someone freeing space wants.
- **Folder** (File list) sorts by the whole path; **Modified** and **Created** newest first.
- Sorting is instant and keeps the selection and the cursor in view; the tree sorts every level the same way.
- Each table (Tree, File list) keeps its own sort, saved with its columns (§ 5.10).

### 4.4 The map (treemap)

Drawn by the CPU into a pixel buffer (`diskdome-treemap`), shown by an `Image`; the interface draws only
what changes on top (hover, selection, labels). Redrawn only on resize, zoom, rescan, theme, style
or option change. In the interface it is called **«Mappa» / «Map»**, a word anyone understands; «treemap» stays in
the code and the docs.

- **Layout**: squarified, files only. Rectangles under 2 px are merged into their folder's remainder, coloured
  with the folder's largest category.
- **Folder separation** (owner, 2026-11-24: «in Vignette the folders don't show»): **every style** leaves gaps in
  `bg` of 3, 2 and 1 px between the folders of the first three levels under the zoom root; deeper folders have no
  gap (it would turn into noise). A style only changes how a file's rectangle is painted.
- **Styles** (owner, 2026-11-24: «more options for the coloured squares, so I choose the best or we keep them
  all»): a setting «Map style», all drawn by the same renderer and compared on the canvas. Tiles, Outline and
  Folders were dropped by the owner the same day; Folders came back for large windows, at the owner's request.
  All ship as a setting (T4: every style redraws in under 2 ms). **Default: Vignette** (owner, 2026-11-24, chosen
  on the artboard «Treemap» with the folder gaps visible in every style).

| Style | How each file rectangle is drawn | Character |
| --- | --- | --- |
| **Vignette** (Vignetta), default | radial gradient: the full category colour up to 35 % of the radius (centre at 50 % / 42 %), then darker toward the corners, 32 % toward black | depth without lightening the colours |
| **Diagonal light** (Luce diagonale) | linear gradient at 135°: 10 % toward white at the top left, the full colour at 45 %, 22 % toward black at the bottom right | soft relief, colours almost full |
| **Mosaic** (Mosaico) | flat colour with a 1 px inner border of the same colour 35 % toward black; no `bg` lines | every file visible, like stained glass |
| **Structure** (Struttura) | flat colour, a 1 px right / bottom edge 18 % toward black; gaps in `bg` between folders of 3, 2 and 1 px at the first three levels | folders readable without headers |
| **Flat** (Piatto) | flat colour, a 1 px `bg` line on the right and bottom edges | the densest, no effect |
| **Cushion** (Cuscino) | van Wijk & van de Wetering cushions: light from the top left, ambient 0.40, diffuse 0.60, height 0.75, 0.85 per level | the classic WinDirStat / WizTree depth; lightens colours toward the centre |
| **Relief** (Rilievo) | flat colour, a 1 px top-left edge 38 % toward white and a 1 px bottom-right edge 38 % toward black | «key» look, retro |
| **Folders** (Cartelle), a setting like the others at every window size (owner, 2026-11-24) | every folder down to 4 levels that is at least 96 × 60 gets a frame: an 18 px header with its name (`caption-strong`) and size (`caption`, `ink-muted`) on `bg` mixed 6 % toward `ink` (+2.5 % per level), its children 2 px inside; smaller folders are drawn flat with 1 px gaps. Names of files never cover a header. In a small map only the folders that fit get a header; the rest are drawn as in Vignette | SpaceSniffer-like: folders named everywhere. Dropped at 1280 × 800 (too many small frames); at 1920 × 1032 the Map view gets 34 headers and still 184 names (artboard «MappaCartelle») |

- **Options menu** (owner, 2026-11-25, decision 32): the `ellipsis` button at the right of the map header opens a `Menu`
  (§ 5.16) holding three `SegmentedControl` rows (§ 5.6), labelled with an overline: **Shape** Rectangles · Rings · Layers
  (§ 4.5, § 4.7), **Colours** File type · Age (§ 4.5), **Names** None · Few · Many · All; then one line «Style:
  Vignette» with «Change in Settings». Rings and Layers have no style and no free space: the line says so and the
  «Free space» checkbox is disabled with that reason. Under a separator: «Save as image…» (the export dialog on
  «Map image», § 6.18) and «Copy as image» (to the clipboard at the size on screen, with a notice). The choices are
  kept per window, like the layout.
- **Names** (owner, 2026-11-24: «as many names as possible, even small ones»): option «Names» in the map
  header's menu: **None**, **Few**, **Many** (default), **All**. Placed greedily, never overlapping:
  first the folders whose files are drawn as one piece, then every file, largest rectangle first. A name needs
  room for at least four characters (plus the folder icon for a folder), else the rectangle gets none. Three
  tiers, top left of the rectangle, elided at the end, in `on-data` of the colour as drawn (for Vignette the
  colour 20 % toward black, for Cushion 10 % toward white):

  | Tier | Rectangle at least | Text | Padding | Names |
  | --- | --- | --- | --- | --- |
  | name + size | 72 × 30 | name `caption-strong`, size `caption` under it | 4 px | Few, Many, All |
  | name | 36 × 15 | `map-name` (11 / 13, 700) | 3 px | Many, All |
  | small name | 22 × 11 | `map-name-xs` (10 / 11, 700) | 2 px | All |

  At most 64 names (Few), 260 (Many), 700 (All). The free-space block, when shown, keeps its own label. Placement
  is computed in Rust on the layout (`diskdome-treemap`) whenever the layout changes; Slint draws the names over
  the image, so the treemap crate needs no text rendering (test 10, § 12). In the mockup the placement of 933
  rectangles took 8–23 ms in JavaScript.
- **Hover**: a 1 px `ink` outline; the right side of the map header (`caption`) shows the category swatch,
  name, size, % of the zoom root, category and extension. No floating card: nothing covers the data.
- **Selection**: a 2 px `ink` outline with a 1 px `bg` halo outside, readable on every category colour; the
  same items are selected in the table (§ 6.1).
- **Zoom**: double-click on a rectangle, or Enter, zooms to its folder; the header's breadcrumb shows the zoom
  root; Backspace or Alt+↑ goes up one level, the house button to the drive. Instant.
- **Highlight**: choosing an extension or category in the side panel, or an active search, dims every other
  rectangle (`dimmed`, blended toward `bg`); the rest keeps its colour.
- **Free space** (owner, 2026-11-24: «see the free space, or hide it with a tick»): a **«Free space» checkbox** in
  the map header, off by default, remembered. Only for drives (disabled on a folder tab). When on, at the drive's
  root the map is split into two blocks, squarified by size: all the used data in one, the free space in the
  other, so the data keeps its shape and the free space reads as one area. The free block is hatched (`surface`
  with a 1 px `line` stripe every 7 px at 135°, a 1 px `line` edge),
  labelled «Free space · 284 GB · 29.8 % of the drive» in `ink-muted`, hoverable, not selectable, not zoomable.
  Zoomed into a folder, free space is not shown. The value is the drive's free space (`GetDiskFreeSpaceExW`,
  total free bytes); with disk quotas the user's own free space goes in the tooltip. The tab bar's summary always
  says «670 GB used · 284 GB free».

### 4.5 More views from the same scan

Owner, 2026-11-24: «more views: we could go well beyond WizTree with the information we collect». A proposal,
drawn in the mockup (artboards «Panoramica» and «Viste»). **All of them ship in v1** (owner, 2026-11-24: «and as
many features as possible»).

Every view below uses **only what the scan already reads**: from the `$MFT`, for every file, size, allocated
size, last-modified date, attributes (hidden, system, compressed, sparse, reparse point) and link count; from
the USN journal, what changes while DiskDome is open. No extra pass over the disk, no file is opened, nothing is
written. A view whose data is missing in a scan mode (no-admin walk, a drive that is not NTFS) says so with an
`EmptyState` and what to do; it never shows invented numbers.

| View | Where in the app | Data | Answers |
| --- | --- | --- | --- |
| **Overview** (Panoramica) | first section | totals, categories, folders, dates, system files | «what is going on with this drive», in one screen |
| **Rings** (Anelli) | map options menu › Shape: Rectangles / Rings / Layers (§ 4.4, § 4.7) | the folder tree | the same as the map, for people who do not read rectangles |
| **Colours: Age** (Colori: Età) | map options menu › Colours: File type / Age, with every shape | last-modified date | «what big things have I not touched in years?» |
| **Timeline** | section | last-modified date, saved scan summaries | «when did the space arrive?», «what changed since Tuesday?» |
| **Activity** (Attività) | section | USN journal | «what is growing right now?» |
| **Windows' space** (Spazio di Windows) | a card in Overview, system drive only | system files and folders | «what does Windows keep for itself, and where do I manage it?» |

WizTree has none of them; WinDirStat has none; DaisyDisk and Filelight have rings only.

#### Overview

The first section (Ctrl+1; open decision 16: the first view after scanning a drive). A grid of `Card`s on
`surface`, 12 px apart, each with an overline title and, on the right of the title, its key number in
`caption-strong`. It scrolls if the window is short; no card has a fixed height.

| Card | Shows | Acts |
| --- | --- | --- |
| **Headline** (no card) | one sentence: «C: is 70 % full: 670 GB used, 284 free. The biggest things are games (129 GB) and archives (104 GB). You can free up to 182 GB without touching personal files.», and the usage bar | — |
| **The map** (first card, 484 px wide) | a small map of the drive in the chosen style, names on the largest rectangles, one caption line on how to read it | a click, or «Open the map», opens Explore › Map |
| **File types** | one stacked bar of the twelve categories (category colours, 1 px `bg` gaps), the legend with size and % | a click on a category highlights it in the map (§ 4.4) and switches to the map view |
| **Where the space goes** | the six folders where the space really is: a folder is listed only when none of its sub-folders holds more than 35 % of it, and never together with its ancestor or descendant (so «Registrazioni OBS», not «C:\Users») | a click selects the folder in the tree |
| **You can free** (Puoi liberare), beside the map, with the screen's primary button «Free up space…» → Clean-up | the same groups as the Clean-up view (§ 6.8): rebuildable folders, temporary files (`%TEMP%`), Downloads not modified for over a year, then the Recycle Bin, the hibernation file and Windows Update (Windows' tools); total «up to 186 GB», the same as Clean-up's header («up to»: they are suggestions) | buttons only navigate («Open Clean-up», «Show») or open Windows' own tool (external-link icon). **Nothing is deleted from this card**: deleting stays in the views and goes through § 6.4 |
| **Modified in the last 7 days** (moved to Timeline by the new-user walk-through) | bytes of the files whose last modification falls on each of the last seven days, one bar per day | a click filters the tree by that day |
| **Largest files** | the five largest files, name, folder (elided in the middle), size | «All» opens Explore › File list |
| **Since the last scan** (no card: one line under the headline, from the second scan of the drive) | «Since yesterday: +3.2 GB. Mostly docker_data.vhdx (+2.4 GB) and Temp (+0.6 GB).» | «See what changed» opens Timeline › What changed |
| **Windows' space** (last row, system drive only) | the rows of § 4.5 «Windows' space», total and share of the drive | each button opens Windows' own tool |

«Modified in the last 7 days» is not growth: a single scan has no earlier state. Real growth is the Activity
view's, when live updates are on.

#### Rings (map shape)

The same hierarchy as the map, drawn as a sunburst: the zoom root in the centre (name `title`, size `caption`
`ink-muted`), then one ring per folder level, four rings; the angle is the size. A folder's ring takes its
dominant category, its depth shown by lightness: level 1 `darker`, 2 base, 3 `lighter`, 4 `lighter` 18 % toward
white. 1 px `surface` lines between arcs; arcs under 0.35° are not drawn (they are inside their parent's span).
Names on level-1 arcs of at least 16° and level-2 arcs of at least 14°, `map-name`, `on-data`. Hover, selection,
double-click to zoom and Backspace work as on the map. Drawn by the CPU renderer like the map (a second layout in
`diskdome-treemap`).

#### Colours: Age

The map's rectangles coloured by the file's **last-modified** date with the age ramp (§ 3.2) instead of the
category. Last-access dates are not used (Windows updates them lazily or not at all); creation dates are not used
(a copy resets them). The side panel's «Categories» list becomes the six age buckets with their totals; hover and
selection as usual. Works with every map style and with Rings.

#### Timeline

A section (Ctrl+4). Order (owner, 2026-11-25, decision 32): first what changes between scans, then what the file dates say —
What changed and History and forecast (§ 4.7; from the second scan), Modified in the last 7 days, When the space
arrived, Calendar. Before the second scan the first two are replaced by one card at the bottom, «From the next
scan», saying what they will show and that DiskDome keeps a summary of every scan (a few MB, no file contents).
The blocks from the file dates:

- **When the space arrived**: a bar chart of GB by month of last modification, the last 24 months plus one bar
  «earlier», coloured with the age ramp. Values in the tooltip; a click on a bar filters the tree and the map to
  that month (a filter chip, § 6.6).
- **Sizes and lost space** (moved to Statistics, § 4.6): the files in six size buckets (under 4 KB, 4–64 KB, 64 KB–1 MB, 1–100 MB, 100 MB–1 GB,
  over 1 GB), each with two bars side by side: how many files and how much space (the small files are millions
  but weigh little). Under them three `StatTile`s: **lost to rounding** (allocated minus size, every file takes at
  least one cluster), **saved by NTFS compression** (size minus allocated, for compressed files), **hard links**
  (as WizTree, every link shows the file's size and its clusters are counted once, on the first name,
  `docs/decisions.md`; the tile says how much more a tool that counted every link's clusters would show).

#### Activity

A section (Ctrl+6), live: what changed on the open drives while DiskDome is open, from the USN journal. One row per
folder (open decision 17: the level at which changes are added up): folder, a sparkline of the last 10 minutes,
added (`+`) and removed (`−`) bytes, the time of the last change. The header repeats the live dot and the
totals («+530 MB and −1.20 GB in the last 10 minutes»). No animation: rows update in place at most once a
second. With live updates paused (status bar, § 2.2) the view is an `EmptyState` saying why and what to do.

#### Windows' space

What Windows keeps for itself, explained: the page file, the hibernation file, restore points
(`System Volume Information`), updates and components (`WinSxS`, `SoftwareDistribution`, `Installer`: partly hard
links, so the real space is smaller, and the card says so) and the Recycle Bin. Each row: `shield-alert` icon in
`warning`, what it is in one line, the size, and a button that **opens Windows' own tool** (performance options,
power options, system protection, Disk Cleanup, the Recycle Bin), with an external-link icon.

DiskDome never deletes, moves or changes these (rule 3: system files are protected) and never elevates to open a
tool (rule 4); a tool that needs administrator rights shows its own prompt. The commands live in
`crates/platform`. In no-admin mode `System Volume Information` cannot be read: its row says «Not readable
without the scanner» and points to the scan mode setting.

#### New pieces these views need

Built in the Kit window first (§ 9), from tokens only, drawn with Slint `Rectangle`s and `Path`s; no chart
library:

- `StackedBar` (the File types bar), `BarChart` (vertical bars with a click filter, a baseline and a value
  tooltip), `Sparkline` (1.5 px line, no axes), `StatTile` (a number at `title` and a `caption` under it on `bg`).
- The `Card` of § 5.23 with an optional right-hand header slot.
- Tokens: the age ramp (§ 3.2) and the two map name steps (§ 3.3).

### 4.6 Statistics

A section (Ctrl+5) of cards in three columns, every row `label · detail · value` at row height, the value
right-aligned in tabular figures, the detail in `caption` `ink-faint`. **Every row is a link**: a click opens
Explore › File list with that row as a filter chip (hidden files, empty folders, owner, path over 260
characters…); a size band filters by size. «Copy the numbers» puts them all on the clipboard as text. Mockup:
artboard «Statistiche».

| Card | Rows | From |
| --- | --- | --- |
| Content | files, folders, mean and median file size, empty files, empty folders, the folder with most files, distinct extensions, files without extension | the model |
| Sizes | the Concentration curve (§ 4.7) above six size bands, each with a bar of bytes (`chart-space`) and the file count; header «4 % of the files hold 78 % of the space» | size |
| Space on disk | size of the files, space really allocated, lost to rounding, saved by NTFS compression, sparse files (allocated of declared), hard links (clusters counted once), OneDrive online-only (no space here) | allocated size, attributes, reparse tag |
| Attributes | hidden, system (`shield-alert`, `warning`), compressed, encrypted (EFS), read-only, temporary: size and count | attributes |
| Links | symbolic links, junctions, app aliases, OneDrive placeholders, alternate data streams. «DiskDome does not follow them: nothing counted twice» | reparse points, named `$DATA` |
| Owners | ranked bars of bytes per owner (the user, SYSTEM, TrustedInstaller, Administrators, others) | security descriptor (`$Secure`) |
| Names and paths | longest path, files over 260 characters (`triangle-alert`: File Explorer may fail on them), deepest folder, names in non-Latin scripts, most common name | names |
| From the internet | files downloaded (Windows' «mark of the web»), those older than a year, the main source | the `Zone.Identifier` stream, resident in the MFT record |
| NTFS | cluster size, MFT size and records in use, USN journal size and the days it covers, files in more than 16 fragments («matters only on hard disks») | volume data, data runs |

### 4.7 Charts

Seven charts on the same data (artboard «Grafici»), each placed where it answers a question:

| Chart | Form | Reads |
| --- | --- | --- |
| **Concentration** | cumulative curve, log x (share of files, largest first) vs share of space; the key point as a dot with its value on hover, the sentence in the card header | how few files hold the space |
| **Calendar** | 53 × 7 heatmap of GB modified per day, five steps of `chart-space` from `bar-track` | the days the space arrived |
| **Big and old** | scatter of the largest folders, log x (days since the newest file) × log y (size); the quadrant «over a year, over 10 GB» on `hover`, its dots in `ink`, the rest `bar-fill` | space you probably no longer need |
| **Layers** | icicle: the zoom root as a 22 px bar, then one row per folder level, width = size, the Rings colours | the hierarchy with readable names |
| **History and forecast** | line of the used space at every scan, the capacity as a hairline, the trend of the last 10 weeks as a lighter line | «when will the drive be full?» (a projection, said so) |
| **What changed** | diverging bars per folder against a scan the user picks (by default the previous one): grown right (`delta-up`), shrunk left (`delta-down`), net total in the header | what grew since Tuesday |
| **All drives** | one row per drive on one shared axis: the five largest categories plus Other, free space hatched; a drive not scanned shows used (`bar-fill`) and free only, from Windows | every drive at once |

**Where every chart lives, when it appears, what stands in for it** (owner, 2026-11-25, decision 32; artboard
«GraficiSchermate»; in the «Main» artboard the map options menu and the Tweak «storico» show both states):

| Chart | Where | Appears | Before, or without the data | Click |
| --- | --- | --- | --- | --- |
| All drives | Start, in place of the drive cards (artboard «StartDopo») | once at least one drive has been scanned | the first time, the drive cards with used and free from Windows (§ 6.16) | opens the drive's Overview |
| Since the last scan (line) | Overview, under the headline | from the second scan of the drive | not shown | Timeline › What changed |
| Small map | Overview, first card | after every scan | — | Explore › Map |
| File types | Overview; Explore's side panel | after every scan | — | highlights the category in the map |
| Windows' space | Overview, last row | system drive only | not shown on other drives | opens Windows' own tool |
| Map: Rectangles | Explore › Map, the default shape | always | — | selects; double-click enters |
| Rings, Layers | Explore › Map › options › Shape | by choice | — | Rings: enters the folder; Layers: selects, double-click enters |
| Colours: Age | Explore › Map › options › Colours, with every shape; the side panel becomes «File ages» with the six buckets | by choice | without dates (rare), the option is disabled with the reason | highlights that age |
| Big and old | Clean-up, right column «To review» (§ 6.8) | at least one folder of 10 GB untouched for over a year | the column says «Nothing big untouched for over a year» | «Show» in Explore; nothing is selected or deleted here |
| What changed | Timeline, first card, with «compared with…» (§ 4.8, comparing with a saved scan) | from the second scan of the same drive or folder | the card «From the next scan» at the bottom | the folder in Explore |
| History and forecast | Timeline, beside it | history from the second scan; forecast with at least 4 weeks of scans and a steady trend | the same card; then «Not enough scans for a forecast yet» | a point: compare with that scan |
| Modified in the last 7 days, When the space arrived, Calendar | Timeline | always (file dates) | — | that day or month in Explore |
| Concentration | Statistics › Sizes, above the bands | at least 1,000 files | the six bands only | the largest files in Explore |
| Owners | Statistics › Owners | scans from the drive's index (NTFS) | «Not available folder by folder» | filter by owner |
| Live (sparklines) | Activity | NTFS drive, live updates on | «Here you see what changes while DiskDome is open. Nothing yet.»; paused: why and how to resume | the folder in Explore |

A chart without its data is never drawn empty: its place says what it will show and from when. No chart deletes
anything; every bar, point or cell opens Explore with the filter set.

Rules for every chart (the `dataviz` method, with DiskDome's tokens): one y-axis, never two; marks thin, a 1–2 px
`surface` gap between touching marks, 2 px lines, dots ≥ 8 px with a 2 px `surface` ring; hairline solid grid in
`line`; text always in `ink` / `ink-muted`, never in a series colour; a legend for two series or more; every mark
has its value on hover and its row in a table view; every bar, cell or dot is clickable and becomes a filter in
Explore. Bytes are `chart-space`, counts `bar-fill`, growth and shrinkage `delta-up` / `delta-down`; status
colours are never series. Drawn with Slint `Path` and `Rectangle` (the heatmap and the scatter by the CPU
renderer if they pass a few thousand marks).

| Token | Dark | Light | Use |
| --- | --- | --- | --- |
| `chart-space` | `#e79551` (7.5) | `#ae6626` (3.7) | bytes in every chart (lines, bars, heatmap steps) |
| `delta-up` | `#e79551` (7.5) | `#ae6626` (3.7) | grown (diverging bars) |
| `delta-down` | `#4caccb` (6.9) | `#1c87a5` (3.5) | shrunk (diverging bars) |
| counts | = `bar-fill` | = `bar-fill` | file counts, context dots |

### 4.8 Features beyond WizTree

Twenty proposals (artboard «Funzioni»). The owner chose the four groups marked v1 below on 2026-11-24, plus the
File Explorer menu entry and the command line (§ 6.16); the rest stays «after v1» until chosen. `docs/features.md`
changes only then.

| Group | Feature | Data | Proposal |
| --- | --- | --- | --- |
| Understand | Compare scans: what changed since a past scan | saved scan summaries | v1 |
| | History and forecast | saved summaries | v1 |
| | Written filters: `ext:mp4 size:>1GB modified:<2024 attr:hidden owner:stefa path:Downloads`, saved filters | MFT | v1 |
| | OneDrive and online-only files: declared vs local space | MFT | v1 |
| Discover | Links and junctions: where they point, never counted twice | MFT | v1 |
| | Space per owner | MFT (`$Secure`) | v1 |
| | Downloaded from the internet, and from where | MFT (`Zone.Identifier`) | v1 |
| | Duplicates: a group of Clean-up (§ 6.8); candidates by size from the MFT; content compared only on request, in the background, with the user's rights; the last copy can never be ticked | MFT + reading files | v1 |
| Act | Guided clean-up: rebuildable folders, Temp, old Downloads in one list, always to the Recycle Bin | MFT | v1 |
| | Move to another drive, with progress; across drives it is a copy plus a delete, and says so | file actions | v1 |
| | Compress with NTFS (LZX) for games and programs, reversible | file actions | after v1 |
| | Free up OneDrive space (make files online-only) | OneDrive | after v1 |
| Watch | Live activity (the Activity section) | USN journal | v1 (owner, 2026-11-24) |
| | Alerts: a drive under 10 % free, a folder growing over 5 GB a day (Windows notification) | USN journal | after v1 |
| | Scheduled scan through the service, summary only | service | after v1 |
| Convenience | Export: CSV, JSON, a self-contained HTML report, the map as PNG | the model | v1 |
| | Command palette (Ctrl+K) | — | v1 |
| | Shortcuts list (Ctrl+\) | — | v1 |
| | Bookmarks: favourite folders as fixed tabs | — | after v1 |
| | Compare two folders side by side | MFT | after v1 |

**Saved scan summaries** (what Compare, History and the instant view at the next launch use): after every scan
DiskDome writes the folders down to 6 levels or above 0.01 % of the drive, each with its MFT file reference (record
and sequence, so a moved or renamed folder is still the same folder), the totals per extension and category,
and the drive's used and free space; 1–3 MB per scan, in `%LOCALAPPDATA%\DiskDome\scans\<volume serial>\`, never a
file's content. Kept: the last 30 and one per month (owner, 2026-11-24).

**Comparing with a saved scan** (owner, 2026-11-25, card «Analisi salvate e confronto»; working in «Main» › Timeline):

- **«compared with…»** in What changed's header: the net change («+3.5 GB»), «compared with» and a ghost button with
  the scan's date («17 Nov, 21:14» with `chevron-down`). Its menu lists the saved scans of the same drive or folder,
  newest first, in two groups, «LAST SCANS» and «ONE A MONTH»: date and time, then «the scan before · 7 days ago · 666
  GB used»; the chosen one has `check`. The default is the scan before. At the bottom «Manage saved scans…» opens
  Settings › Saved summaries with this drive open.
- Under the header one line: «From 17 Nov, 21:14 (7 days ago) to today: 666 → 670 GB used. Folders are recognised by
  their number in the MFT, even if you moved or renamed them.»
- **Rows**: the folders that changed most, grown right and shrunk left of the axis, each with its parent path in
  `ink-faint` and the value; «Other folders» takes the rest so the rows add up to the net change. A click on a row
  opens it in Explore. A **moved** folder is one line, not a minus and a plus: «ISO moved: Downloads → Documents ·
  5.40 GB, counts neither more nor less» (`folder-input`); renamed folders keep their new name. Folder walks have no
  MFT numbers: they match by path, and the line says «matched by path: a moved folder shows as shrunk here and grown
  there».
- **History**: every point of History and forecast is a target («17 Nov: 666 GB · click: compare with this scan»);
  the compared one has an `accent` ring, and the caption says which.
- **Saved summaries** in Settings (§ 6.10): each drive's row opens to its scans, one per row (date, «the scan of
  now», «among the last 30» or «one a month», used space) with «Delete» on each but the current one; the notice has
  «Undo». Deleting the compared scan moves the comparison to the scan before it; with no scan left the Timeline shows
  «From the next scan».

**Memory.** Performance is the product, so every per-entry field is paid for 14 million entries. Done on
2026-11-24 (card 4.T13, `docs/architecture.md`): size on disk as `u32` clusters (`alloc`), last modified as `u32`
Unix seconds (`mtime`, equal to what File Explorer shows), attributes `u16` (`attrs`), the security descriptor's
number `u16` (`security`, the owner resolved from it later), link and fragment flags in the existing `flags` byte.
The model stays at 61.5 bytes per entry; peak 0.89 GB on 14.3 million generated records, 0.39 GB on the owner's C:.
Rare data (link targets, alternate streams, download sources, very fragmented files) goes in side tables sorted by
entry index, never a hash map.

## 5. Components

Every component is one `.slint` file in `crates/dome-ui/ui/components/`, exported through
`index.slint`, the only file of the `@dome-ui` library an app can import. Variants and sizes are enums. Each one appears in the Kit window in every variant ×
state × theme × density before a screen uses it. Screens never restyle a component: a missing look is a new
variant, decided with the owner.

Not every entry below belongs to Dome UI: the ones that know a drive, a scan or a view of DiskDome (`DrivePicker`,
`DriveCard`, `DriveTabs`, `ViewSwitch`, `ActionProgress`, the `Treemap`) live in the app, built from Dome UI's
generic components (§ 8.4).

States, with the same meaning everywhere:

| State | Look |
| --- | --- |
| rest | as specified per component |
| hover | the next lighter fill (`control-hover`, `hover`); `fast` |
| pressed | `control-pressed` / `accent-pressed`; no scaling (desktop, dense) |
| focused (keyboard) | focus ring (§ 3.7) |
| disabled | `disabled` opacity on the whole control, no hover, reason in the tooltip |
| checked / active / selected | `accent-soft` fill or `accent-text`, per component |
| busy | the control keeps its size; its icon becomes a 2 px progress arc in its text colour |

### 5.1 Label

The only text primitive. Properties: `text`, `step: TypeStep` (`display`, `title`, `body`, `body-strong`,
`caption`, `caption-strong`, `overline`, `mono`, `map-name`, `map-name-xs`), `tone: InkTone` (`normal`, `muted`, `faint`, `accent`, `danger`,
`warning`, `success`, `on-accent`, `on-danger`, `data` (a category's own colour), `on-data` (the ink on it)), `align`, `elide: Elide` (`none`, `end`, `middle`). Overline
upper-cases its text (the translation stays lower-case in the `.po`). Middle elision is computed in Rust for
paths (Slint elides only at the end) and passed in ready.

| Label | |
| --- | --- |
| Wrap `LabelWrap` | `none` (default: one line, exactly the step's line tall; names and paths in tables and bars), `word` (between words: dialogs, info bars, toasts, empty states), `anywhere` (mid-word too: a long path in a dialog). A wrapped text starts at the top of its box and steps by the font's own line |
| `max-lines` | a wrapped text stops after that many lines, the last one cut with «…»; 0 (default) for no limit |
| `cut` (out) | the text does not fit its box. The control holding the label then says it whole in its tooltip (§ 3.3): a pointer area on the label itself would take the clicks of the row under it. `Button` and `SegmentedControl` read it |

**SelectableLabel**: a label whose text can be selected and copied (details, messages, dialogs; never where a click
belongs to a row, the map or a control). Same `text`, `step`, `tone`, `category`, `align` and `wrap` as `Label`.

| SelectableLabel | |
| --- | --- |
| Pointer | drag selects, double click a word, triple click a line; I-beam pointer; the selection is `accent-soft` under the label's own ink; no caret |
| Keyboard | Tab reaches it, as Windows' read-only fields; Ctrl+A selects it all, Ctrl+C copies the selection |
| Context menu | `has-selection` (out), `copy()`, `select-all()`, `clear-selection()`: the screen's `ContextMenu` offers «Copy» (disabled with nothing selected) and «Select all»; a right click keeps the selection |
| Overflow | it wraps (`word`, `anywhere`) or clips at its edge: never «…», so a long one-line text (a path) wraps `anywhere` |

It is a component of its own, not an option of `Label`, so the many labels of tables and the map stay a plain text
without the weight of an input.

### 5.2 Button

| | |
| --- | --- |
| Variants `ButtonVariant` | `primary` (accent fill, `on-accent`, `body-strong`), `secondary` (`control` fill, `ink`), `ghost` (transparent, `ink-muted`, hover `control-hover`), `danger` (`danger` fill, text `bg`, `body-strong`), `danger-ghost` (transparent, `danger` text; destructive items outside dialogs) |
| Sizes `ButtonSize` | `small` (control-sm), `normal` (control), `large` (control-lg), `hero` (40 px, `title` text, icon 18, 20 px padding, min 160 px: only the call to action of an empty state, one per screen) |
| Content | text; icon + text (icon 16, `label-gap`); text + trailing icon (`icon-end`, as the chevron of a button that opens a list; the padding on its side is `button-pad-x-icon`); icon only (square, `label` mandatory: accessible name + tooltip) |
| Width | `ButtonWidth.hug` (default): hugs the content, minimum 64 px with text (Italian labels grow); short of room it gives way down to that minimum and cuts the label with «…», and the tooltip says the full text. `ButtonWidth.fill`: the whole width its layout gives (a dialog's or a panel's full-width button). Icon-only is square |
| Radius | `button` (4) |
| Rules | one `primary` per view; destructive actions never icon-only; a dialog's quiet way out («Cancel») is `secondary` |
| Keyboard | Enter and Space activate; the dialog's default button takes Enter from anywhere in the dialog |

### 5.3 ToggleButton

A button with `checked`: checked = `accent-soft` fill, icon and text `accent-text`; unchecked `ink-muted`. Used for
the panel toggles and the search's `Aa` / `.*`. Accessible as a toggle (`accessible-checked`).

| ToggleButton | |
| --- | --- |
| Interface | `text`, `icon`, `checked` (written by both the button and the screen), `variant`, `size`, `enabled`, `tooltip`, `tooltip-shortcut`; `toggled(checked)` when the user flips it |
| Variants `ToggleVariant` | `ghost` (default: transparent, hover `control-hover`), `outline` (the same with a 1 px `line` edge, a toggle standing alone on a panel) |
| Sizes `ToggleSize` | `small` (control-sm, inside a field), `normal` (control, a toolbar), `large` (control-lg, beside a large button) |
| Content | text; icon + text (icon 16, `label-gap`, the icon's side padded `button-pad-x-icon`); icon only (square, `accessible-label` mandatory: accessible name + tooltip). The content is the Button's (`ButtonContent`) |
| Width | hugs its content; short of room it gives way down to a square and cuts the label with «…», and the tooltip says the full text |
| Keyboard | Enter and Space flip it; Esc puts the tooltip away |

### 5.4 TextField and SearchField

- `TextField`: height `control`, `field` fill, 1 px `line`, radius `control`, `field-pad-x`; placeholder
  `ink-faint`; focused: 1 px `focus` edge plus the ring when the keyboard brought the focus there; error: 1 px
  `danger` edge and a `caption` message under it in `danger` with `circle-alert`.
- `SearchField`: `search` icon on the left, a clear `x` when not empty, and two `ToggleButton`s inside on the
  right: `case-sensitive` (match case) and `regex` (regular expression). Width 320 in the toolbar (grows to 400 when
  there is room). A wildcard (`*`, `?`) pattern needs no toggle. An invalid regular expression turns the field's
  edge `danger` and says why in the tooltip, without a dialog. Enter applies immediately; typing applies after
  150 ms of rest. Esc clears, a second Esc leaves the field.
- `mono` step for the field's text while regex is on.

| TextField | |
| --- | --- |
| Interface | `text` (written by both the field and the screen), `placeholder`, `label`, `label-position`, `help`, `error`, `icon`, `step` (default `body`), `input-type` (Slint's `InputType`: `password` hides the characters), `read-only`, `enabled`; `has-focus`, `box-y` and `box-height` out (where the box sits under its label, for a list opened under it); `edited(text)` on every change the user makes, `accepted(text)` on Enter |
| Label `FieldLabelPosition` | `top` (default: `FieldLabel` above the box, `field-gap` under it), `hidden` (the place already says what the field is for: a toolbar, a table cell). Either way it is the accessible name; with no label, the placeholder is |
| Help and error | `help` (what to type) and `error` (what is wrong and what to do) under the box, each a `FieldMessage`, `field-gap` apart, the help first and the error under it (shadcn's order); they wrap between words at the field's width. An error also turns the edge `danger`, over the focus; screen readers read the error, or else the help, as the description |
| Pointer | a click anywhere on the icon, the text or the label puts the focus in the text (text cursor over the box) and turns the focus ring off |
| Keyboard | Tab out of the field turns the keyboard flag on (§ 3.7), so the next control shows its ring; Enter reports `accepted`. No Esc of its own: a control built on the field (a combobox) takes the keys it needs around it |
| Content | an optional `icon` leads the text (`ink-faint`); the field's children go at the box's end, centred by the row (a unit, a clear button), `label-gap` from the text and `field-pad-x` from the edge |
| States | hover changes nothing (as shadcn's input); disabled fades the whole field, label and messages; read-only keeps the look and lets the text be selected and copied |
| Width | `field-w` where the layout sets none, never under `field-w-min`; it stretches to fill a dialog's column. A long label is cut with «…» |
| Accessibility | the text is the `text-input`, named by the label and described by the error or the help, with the placeholder as its placeholder text. Slint 1.19 has no «labelled by», «described by» or «invalid» state: the label's text is copied as the name, the message as the description, and the invalid state is said only by the message |

`FieldLabel` is a field's name (`body`, `ink`), cut with «…»; a click on it reports `clicked` so its owner focuses
the field, as a Radix Label does. `FieldMessage` is the line under a field, `kind: FieldMessageKind`: `help`
(`caption`, `ink-muted`) or `error` (`caption`, `danger`, led by a 12 px `circle-alert` on its first line, `label-gap`
from the text). `TextField` draws both; another control (a select, a group of checkboxes) uses them on its own.
`TextField` and `SearchField` share their box (`FieldFrame`: ground, edge and ring).

| SearchField | |
| --- | --- |
| Interface | `text`, `match-case`, `regex` (written by both the field and the screen), `placeholder`, `label` (the accessible name; the placeholder when empty), `error`, `toggles`, `rest-delay` (default `search-rest`, 150 ms), `enabled`; `has-focus` out; `search-requested(text)` after the rest, on Enter, on a toggle and on clearing |
| Toggles `SearchFieldToggles` | `match-case-and-regex` (default: the files' search), `match-case`, `none` (the command palette, the shortcuts dialog, § 6.19) |
| Error | `error` is the reason, never empty when the pattern is wrong: the edge turns `danger` and the reason shows in a tooltip under the field, aligned to its start (§ 6.6), at once while the field has the focus and after the delay on hover over the text; screen readers read it as the description |
| Pointer | a click anywhere on the icon or the text puts the focus in the text (text cursor over both) |
| Clear | the `x` shows with text on an enabled field; its tooltip says «Clear the search» with the key «Esc» |
| Width | 200 / 320 / 400 (`search-w-min`, `search-w`, `search-w-max`); a screen sets other widths where the field has another place |

### 5.5 Select and DrivePicker

- `Select`: looks like a `secondary` button with a `chevron-down` on the right; opens a level-1 list
  (`menu-item` rows, the chosen one with `check`). Arrow keys move inside, Enter chooses, Esc closes; typing
  jumps to the first match.

| Select | |
| --- | --- |
| Interface | `entries` (`[MenuEntry]`: plain items, with separators, section titles and disabled items as in a menu), `current-index` (-1 for none; written by both the select and the screen), `placeholder`, `size`, `width-mode`, `enabled`, `tooltip`; for the list `side`, `align`, `list-min-width`, `list-max-width`, `wrap-around`; `is-open` out; `selected(index)` when the user chooses another item; `open(start)` |
| Sizes `SelectSize` | `small`, `normal` (default), `large`: the button's heights of the same name |
| Width `SelectWidth` | `longest` (default): as wide as its widest item or placeholder would make it, so it never moves when the choice changes; `chosen`: hugs the item it shows; `fill`: the width its layout gives. Short of room it gives way down to the button's minimum, cuts the text with «…» and says it whole in the tooltip |
| Placeholder | shown while nothing is chosen (`current-index` -1), in the button's own ink; also the accessible name when none is given |
| List | a `Menu` under the select (flipping above it near the window's bottom), never narrower than the select; every item is drawn as a check box item, checked on the chosen one (an item's own icon is not drawn: the slot holds the `check`) |
| Opening | a click, Enter, Space, ↓ or ↑; it opens on the chosen item, highlighted; with nothing chosen, on the first item (↓, Enter, Space from the keyboard), the last (↑) or none (the pointer) |
| Keys in the list | as a menu (§ 5.16): ↑ ↓ (stopping at the ends unless `wrap-around`), Home, End, PgUp, PgDn, type-ahead, Enter and Space choose, Esc and Tab close; a disabled item is highlighted, never chosen. Closing gives the focus back to the select |
| Closed type-ahead | typing on the focused, closed select chooses the next enabled item starting with the letters, without opening the list; one letter repeated goes through the items starting with it; the letters reset after `typeahead-reset` |
| `selected` | only when the choice changes: choosing the chosen item again closes the list without it |
| Accessible | role `combobox`, its value the shown text, expandable and expanded while the list is open |
| Not done | Radix's `item-aligned` position (the list laid over the select with the chosen item on it, as Windows' combo box): the list always opens under or above it; ↑ ↓ on the closed select changing the choice without opening (Windows); Space typed during a type-ahead counts as a letter (Radix): here it opens the list |
- `DrivePicker`: the menu of the drive tabs' «+» (§ 5.27): the drives that have no tab (network drives,
  drives hidden by the user) with a 4 px usage bar (`bar-fill`, `warning` above 90 % full), the file system and
  whether the fast scan applies («NTFS · fast»); then «Analyze a folder…» and «Network path…».

### 5.6 SegmentedControl

2–4 options in one `control-sm` / `control` frame, `control` fill, radius `control`; the chosen segment
`raised` fill with `ink`, the others `ink-muted`. Arrow keys move the choice. Used for «Extensions | Categories»,
and in the settings (theme, density, units).

| | |
| --- | --- |
| Interface | `items: [SegmentedItem]`, `current-index` (written by both the control and the screen), `selected(index)` when the user chooses |
| `SegmentedItem` | `text` (always the accessible name), `icon`, `icon-only`, `tooltip`, `disabled` |
| Content | text; icon + text (icon 16, `label-gap`, the icon's side padded `button-pad-x-icon`); icon only (square, its text said by the tooltip and screen readers) |
| Width | `SegmentedWidth.hug` (default): hugs its segments; short of room they give way down to `segment-min-w` (40, pending design) and cut their labels with «…», and the tooltip says the full text. `SegmentedWidth.fill`: the whole width its layout gives, split equally between the segments («Extensions \| Categories» in the side panel) |
| Disabled segment | faded (`disabled` opacity), not clickable, skipped by the keyboard; its `tooltip` says why |
| Keyboard | one Tab stop; ← ↑ / → ↓ choose the previous / next enabled segment, Home / End the first / last; `wrap-around` (default on, as Radix and the Menu) goes round the ends; Esc puts the tooltip away |
| Tooltip | a segment's `tooltip`, or the text of an icon-only or cut one; on hover, and at once for the chosen segment under keyboard focus |
| Nothing chosen | `current-index` out of range: no segment is chosen; an arrow chooses the first (→) or last (←) enabled one |

### 5.7 Tabs

Text tabs (`body`, `ink-muted`) in the tab bar; the active one `ink` with a 2 px `accent` underline drawn
inside the bar's bottom. Ctrl+1 / 2 / 3 switch them (Ctrl+Tab belongs to the drive tabs, § 5.27). No icons.

| | |
| --- | --- |
| Interface | `items: [TabItem]`, `current-index` (written by both the tabs and the screen), `selected(index)` when the user chooses; `activation`, `overflow-mode`, `wrap-around`, `enabled`; out: `focus-index`, `tooltip-index`, `overflowing`, `shift` |
| `TabItem` | `text` (always the accessible name), `tooltip`, `disabled` |
| Bar | `tabbar` tall; each tab its label with `tab-pad-x` on both sides, the underline as wide as the label (pending design); hover `hover` fill, pressed `control-pressed`; the tabs hug their labels and the screen places them (the section bar's summary sits right) |
| Keyboard | one Tab stop, on the current tab; ← / → the previous / next enabled tab, Home / End the first / last, ↑ ↓ left to the screen (a horizontal list, as Radix); `wrap-around` (default on) goes round the ends; Esc puts the tooltip away. `TabActivation.automatic` (default, as Radix): the arrows choose; `manual`: they move the focus ring and Enter / Space choose, and leaving the tabs puts the focus back on the current one |
| Shortcuts | Ctrl+1 … belong to the screen, which sets `current-index` from its own key handling: the tabs hear keys only while focused |
| Disabled tab | faded (`disabled` opacity), not clickable, skipped by the keyboard; its `tooltip` says why |
| Tooltip | a tab's `tooltip`, or the text of a cut label; on hover, and at once for the focused tab under keyboard focus |
| Overflow | `TabOverflow.shrink` (default): the tabs give way down to `tab-min-w` (40, pending design) and cut their labels with «…», and the tooltip says the full text. `TabOverflow.menu`: the labels stay whole; when they do not fit, the bar ends with a small ghost «⋯» `MenuButton` («More tabs», its own Tab stop) listing every tab as a radio item (the current one checked, a disabled one with its reason), and the tabs slide so the focused one is whole in view |
| Nothing current | `current-index` out of range: no tab is underlined; an arrow chooses the first (→) or last (←) enabled one |
| `TabPanel` | the content the screen shows for the current tab (`if tabs.current-index == 1: TabPanel { … }`): a column with the `tab-panel` role, named by its `accessible-label` |
| Accessibility | the bar is a `tab-list` with its item count; each tab a `tab` with its index, selected when current, its tooltip as description |
| Not done | vertical tabs (no vertical tab bar in the design), closable tabs and icons (the drive tabs, § 5.27, are their own component on the same keyboard walk), a link from a panel to its tab for screen readers (Slint 1.19 has no relation property), scroll buttons |

### 5.8 Checkbox, Radio, Switch

- `Checkbox`: 14 px box, radius `control`, 1 px `line-strong`; checked: `accent` fill with a `check` in
  `on-accent`; indeterminate: a 2 px bar. Label on the right (`body`), the whole row clickable.
- `Radio`: the same circle-shaped; used inside `OptionCard` (scan mode).
- `Switch`: track 28 × 16, knob 12; off: track `control`, knob `ink-muted`; on: track `accent`, knob
  `on-accent`. For settings that apply at once; a checkbox for choices confirmed by a button.

The three draw the same row: the mark (box, circle, track) beside the first line of a `body` label, `label-gap`
between them, `choice-h` tall on one line (pending design: `control-sm`). The whole row is the pointer target; hover
and press change the mark (`control-hover` / `control-pressed`, or `accent-hover` / `accent-pressed` when marked); the
focus ring follows the mark. Disabled: `disabled` opacity on the whole row, its `tooltip` says why.

| Common | |
| --- | --- |
| Label `wrap: LabelWrap` | `none` (default): one line, cut with «…» and said whole in the tooltip; `word` / `anywhere`: more lines, the mark beside the first |
| `tooltip` | said on hover and keyboard focus (§ 5.17); on a disabled control, why; Esc puts it away |
| Accessibility | `accessible-role` `checkbox` / `radio-button` (in a `radio-group`) / `switch`, the label as name, `accessible-checked`, `accessible-enabled`, the tooltip as description. Slint 1.19 has no «mixed» state: an indeterminate box reads as not checked |

| Checkbox | |
| --- | --- |
| Interface | `text`, `state` (written by both the box and the screen), `wrap`, `enabled`, `tooltip`; `toggled(state)` when the user flips it |
| `CheckboxState` | `unchecked`, `checked`, `indeterminate` (some of what the box stands for is ticked: a group's box over its rows). A click or Space from `indeterminate` ticks it; `toggled` never carries `indeterminate` |
| Keyboard | Space flips it; Enter does not (it belongs to the dialog's default button), as WAI-ARIA and Radix |

| RadioGroup | |
| --- | --- |
| Interface | `items: [RadioItem]`, `current-index` (written by both the group and the screen; -1, the default, chooses none), `orientation` (`vertical` default, `horizontal` with `choice-gap-x` between items), `wrap-around` (default on), `wrap`, `enabled`; `selected(index)` when the user chooses; `focus-index` out |
| `RadioItem` | `text` (shown, the accessible name), `tooltip`, `disabled` |
| Keyboard | one Tab stop, on the chosen item or, with none, the first enabled one; every arrow chooses the previous / next enabled item (as Radix, in either orientation), `wrap-around` goes round the ends; Space chooses the focused item; Enter does nothing |
| Disabled item | faded, not clickable, skipped by the arrows; its `tooltip` says why |

| Switch | |
| --- | --- |
| Interface | `text`, `checked` (written by both the switch and the screen), `label-side`, `wrap`, `enabled`, `tooltip`; `toggled(checked)` when the user flips it |
| `SwitchLabelSide` | `right` (default, as a check box), `left`: the label first and the switch at the end of the width its layout gives (a settings row) |
| Keyboard | Space and Enter flip it (a switch is a button, as Radix) |
| Motion | the knob slides in `fast`, at an even pace (on whole pixels an eased slide jumps) |

### 5.9 OptionCard

A card-sized radio (scan mode, first launch): `surface` fill, 1 px `line`, radius `card`, `card-pad`; title
`body-strong`, description `caption` `ink-muted`, optional badge («Recommended»). Chosen: 1 px `accent` edge plus
`accent-soft` fill. Arrow keys move the choice.

### 5.10 Table (the virtual table)

The heart of the app: Tree, File list and Clean-up are the same component with different columns. Built on Slint's
`ListView` (dense rows, full keyboard), with our own virtual scrolling: the scrollbar maps to the row index, so
14 million rows scroll precisely to the last one (owner, 2026-11-24, from test 3c). Columns, sorting, resizing,
reordering and the special entries: owner, 2026-11-25 (card «Albero ed Elenco file»; artboard «Tabella», and working
in «Main»).

- **Rows** come from a Rust `Model` that serves only the visible rows (`docs/architecture.md`): 14 million
  rows scroll as smoothly as 10. Row height `row`; no zebra stripes; no lines between rows.

#### Columns

| Column | Tree | File list | Width (normal density) | Shows |
| --- | --- | --- | --- | --- |
| Name | always, first | always, first | Tree: the rest, ≥ 200 · File list: 300 | `NameCell` |
| Size | yes | yes | 100 | `SizeCell` (§ 4.1); `ink-faint` when not counted |
| % of the folder | yes | — | 120 | `PercentCell` (§ 4.2) |
| Allocated («Spazio occupato») | yes | yes | 112 | the clusters really used: less when compressed or sparse, 0 when online only |
| Files | yes | — | 80 | files inside, at every level |
| Modified | yes | yes | 128 | `DateCell` |
| Folder | — | yes | the rest, ≥ 160 | `PathCell`, the containing folder, middle-elided, whole in the tooltip |
| Folders | hidden | — | 72 | folders inside |
| Type | — | hidden | 72 | the extension |
| Created | hidden | hidden | 128 | `DateCell`; needs the created time in the model (`u32`, 4 bytes per entry, measured with `./scripts/proof.ps1`) |
| Attributes | hidden | hidden | 76 | `AttrCell` (§ 4.1) |
| Owner | hidden | hidden | 120 | the account name («stefa»); the full one («DELLDOMENGHINI\stefa») in the tooltip |

The defaults are the owner's of 2026-11-24 (Tree: Name, Size, %, Allocated, Files, Modified). Widths include the
cell padding; a column's header label fits its default width with the sort arrow. The File list holds every file
of the tab, largest first, narrowed by search and filters; its menu adds «Show in the tree».

#### Header

- `header-row` tall, `bg`, sticky, a 1 px `line` under it. Labels `caption`: `ink-muted`, the sorted one `ink`
  with its arrow (§ 4.3). Right-aligned columns have right-aligned labels. Hover: `hover` fill.
- **Sort**: click (§ 4.3).
- **Resize**: drag the 6 px handle on a header's right edge; a 1 px `line` shows when the header is hovered,
  `line-strong` when the handle is, and the cursor becomes ↔. Double-click the handle: fit the content. Each column
  has a minimum (numbers 72, counts 48, dates 88, Name 200 in the Tree, 160 in the File list); the widest text column
  (Name in the Tree, Folder in the File list, Name if Folder is hidden) takes the space left.
- **Reorder**: drag a header; it follows the pointer on `control` with `shadow`, its slot dims to `ink-faint`, and
  a 2 px `accent` line shows where it lands. Esc cancels. Name stays first (it carries the tree's indent).
- **Columns…**: right-click on the header, or the 22 px ghost button `columns-3` at its right end (above the
  scrollbar). A menu titled «Tree columns» / «File list columns»: one checkable item per column (Name checked and
  disabled, «always shown»), then «Fit the column widths» and «Restore the default columns». It stays open while
  columns are ticked; Esc or a click outside closes it. A shown column is added at the right end.
- **Narrow window**: the table scrolls sideways, header and rows together; Name stays pinned on the left.
- **Saved** per table (Tree, File list), for every tab: visible columns, order, widths, sort.
- **Keyboard**: Ctrl+Shift+1…9 sorts (§ 4.3); Tab from the rows reaches the header (one stop) with a 1 px `focus`
  frame on a header cell: ← → move between columns, Enter or Space sorts, Ctrl+← → moves the column, Shift+← →
  narrows or widens it by 8 px, Shift+F10 or the Menu key opens «Columns…». Narrator reads «Size, column header,
  sorted largest first».

#### Cells

`NameCell` (indent `indent` × depth, `chevron-right` / `chevron-down` 16 px for folders, icon, name `body`, then
what follows the name), `SizeCell` (§ 4.1), `PercentCell` (§ 4.2), `CountCell`, `DateCell`, `AttrCell`,
`PathCell` (middle-elided, `ink-muted`). Numbers right-aligned, text left. Side padding `cell-pad-x`; a 12 px
space after the last column.

#### Row states

| State | Look |
| --- | --- |
| rest | transparent on `surface` |
| hover | `hover` fill |
| selected (table focused) | `accent-soft` fill, text stays `ink` |
| selected (focus elsewhere) | `selected-inactive` fill |
| cursor (keyboard) | 1 px `focus` frame inside the row, over the fill |
| pending (delete / move running) | `pending` opacity; a 16 px progress ring (2 px, `accent` on `line-strong`) instead of the icon; not selectable until the file system confirms; the tooltip says the action and its percentage |
| protected (system item) | `shield-alert` badge «system» in `warning` after the name; nothing else changes until an action touches it |
| rebuildable (`node_modules`, `target`, caches) | `recycle` badge «rebuildable» in `ink-muted` |
| filtered-out ancestor (search) | name in `ink-faint`, kept to show the path to a match |

#### Special entries

| Entry | Icon | After the name | Size · Allocated | Behaviour |
| --- | --- | --- | --- | --- |
| Junction, folder symbolic link | `folder-symlink`, `ink-muted` | «→ C:\Users» in `ink-faint`, shrinks first | `—` · `—` | no chevron; never followed (nothing counted twice); Enter, double-click or «Go to the target» (§ 6.3) selects the target, opening its drive's tab if needed |
| File symbolic link | `file-symlink` | «→ target» | `—` · `—` | as above |
| OneDrive online-only file | `cloud`, `ink-muted` | — | declared size and 0, both `ink-faint` | not counted (§ 4.1); tooltip «Online only: 7.82 GB on OneDrive, nothing on this drive» |
| Folder with online-only content | `folder` | a 12 px `cloud` in `ink-faint` | the local size | tooltip «Another 12.6 GB online only, not counted»; only on the folder that holds them |
| Hard link, second name | the file's icon | `link-2` badge «hard link» | size in `ink-faint` | counted once, on the first name; tooltip names the first |
| Compressed (NTFS) | normal | — | Allocated smaller | letter C; Allocated tooltip «Compressed by NTFS: saves 1.70 GB» |
| Sparse | normal | — | Allocated smaller | letter P; tooltip «Sparse file: declares 48.5 GB, uses 29.2» |
| Unreadable folder (no-admin mode, network) | `folder-lock` | `warning` badge «unreadable» | `—` · `—` | no chevron; tooltip «Access denied: we don't know how much it holds. Scan as administrator to read it»; the status bar: «2 folders unreadable: the totals leave them out · Scan as administrator» |

- **Empty table** (no match): an `EmptyState` in the table area.

### 5.11 Treemap

The view of § 4.4: an `Image` of the pixel buffer plus overlays (hover outline, selection outlines, labels).
Inputs: `buffer: image`, `hover-rect`, `selected-rects: [Rect]`, `labels: [TreemapLabel]`, `dimmed: bool`. Outputs:
`pointer-moved(x, y)`, `clicked(x, y, modifiers)`, `double-clicked(x, y)`, `context-menu(x, y)`. Hit-testing is
done in Rust on the layout, never in Slint. Its header: `house` and `corner-left-up` (ghost, small), the
`Breadcrumb`, then on the right the hovered item (swatch, name, size, %, category; the zoom root's summary when
nothing is hovered), the «Free space» `Checkbox` (§ 4.4) and an `ellipsis` menu with «Map style», «Names», «Shape» and «Colours» (§ 4.5).

### 5.12 Side panel: extensions and categories

Header at the tab bar's height: overline «FILE TYPES» and the `SegmentedControl` «Extensions | Categories». With the map
coloured by age (§ 4.5) the overline is «FILE AGES», the control is hidden and the rows are the six age buckets.
While a highlight is active, a small ghost button «× Highlight» clears it. A table with:
swatch (10 × 10, radius `swatch`, the exact shade), extension (`.js`, `body`) or category name, size, %, files.
Category mode groups rows under the category with its total; a category row expands to its extensions. A
click highlights in the treemap and the table (§ 4.4); a double-click adds it as a filter chip. The footer
(row height, `bg`): «312 extensions · 931 GB».

### 5.13 Breadcrumb

Segments `caption` `ink-muted`, the last `ink`, separated by `chevron-right` 12 px `ink-faint`; each segment
is a ghost button; when it does not fit, the middle segments collapse into «…», a menu.

### 5.14 Badge

Height 16: `caption` (12 / 16), 4 px side padding, radius `badge`.
`BadgeVariant`: `neutral` (`control` fill, `ink-muted`), `accent` (`accent-soft`, `accent-text`), `warning`,
`danger`, `success` (their `-soft` fill and text). Icon 12 optional. Used for «Recommended», «rebuildable»,
«system», counts on the Filters button.

### 5.15 Chip (filters)

`control-sm` tall, `control` fill (`control-hover` on hover), radius `control`, text `caption` «Extension: .mp4»,
an `x` to remove (16 px, its own hit area, `label` «Remove filter»). Lives in the filter bar; a click opens the
Filters panel at its group, Del or Backspace on a focused chip removes it. In the panel, toggle chips share the
shape: off `control` / `ink-muted`, on `accent-soft` / `accent-text`, with an optional 8 px swatch (§ 6.6).

### 5.16 Menu and ContextMenu

Level-1 popup: `raised`, `edge`, radius `panel`, `menu-pad`, width 200–360. `MenuItem`: `menu-item` tall,
icon 16 (or empty slot, so labels align), label `body`, shortcut `caption` `ink-faint` right-aligned, submenu
`chevron-right`; hover / keyboard highlight = `control-hover` fill (not the accent: menus are not selection);
`danger` variant = `danger` text and icon; disabled = `disabled`. `MenuSeparator`: 1 px `line` with 4 px above
and below. `MenuSection`: `overline` title.

Slint's `ContextMenuArea` / `Menu` draw with the widget style, not our tokens (test 8: the Fluent
style's colours are fixed, the menu stays light in our dark theme, radius 7, at least 280 wide, items 32 tall; no
shortcut text outside a menu bar, no `danger` item, no section title). Decision 11: `Menu` is ours on a `PopupWindow`, with the same keyboard behaviour (arrows, Enter, Esc,
→ / ← for submenus, the Menu key and Shift+F10 to open).

`MenuFrame` is the level-1 surface with free content (the map's
options, the DrivePicker); `MenuList` draws `[MenuEntry]` (`kind`: item, checkbox, radio, separator, section; `icon`,
`hint`, `shortcut`, `variant`, `disabled` with its `reason`, `checked`, `submenu`), hides leading, trailing and
repeated separators, elides long labels and scrolls; `Menu` is the popup, three levels deep, placed by `Placement`
(§ 3.11: `side`, `align`, `gap`, `margin`, flip, shift, cut to the room), with `list-min-width` / `list-max-width`,
`match-anchor-width`, `wrap-around`, `row-height`, `empty-text`, `keep-open-on-check`; `ContextMenu` asks for a menu
on a right click (where its content does not take the pointer itself), the Menu key and Shift+F10; `MenuButton`
opens one under a button (↓ on the first item, ↑ on the last) and takes the focus back when it closes. Keys: ↑ ↓
(skipping separators and titles, going round), Home, End, PgUp, PgDn, type-ahead, Enter and Space, → and ← for
submenus, Esc (the top level only) and Tab; a disabled item is highlighted but not chosen, as in Windows. The menu
never changes its entries: a screen keeps them with `entries = MenuNav.toggled(entries, entry.id)` in `activated`
(a check box flips; a radio item is checked and the radio items next to it cleared), as every Kit menu does. Not done:
the tooltip with a disabled item's reason and a cut label's full text (a tooltip inside a popup is drawn under it,
§ 5.17), the exit fade (Slint closes a popup at once), touch long press.

### 5.17 Tooltip

`raised`, `edge`, radius `panel`, `tooltip-pad`, `caption` `ink`, max 320 wide, wraps. After 500 ms of rest
with the mouse, at once on keyboard focus; never on rows of the table (they have the status bar) or on the treemap
(it has its header). Slint 1.19's `Tooltip` element takes our tokens when it is given our content instead of a
`text` (test 8), but it shows on hover only (never on keyboard focus), after a fixed 500 ms, beside the
pointer, without the fade and rise; with a `text` it draws the style's own light box. Decision 11: the tooltip is
ours, with the keyboard behaviour above.

Not on a `PopupWindow`: Slint gives every popup but its own tooltip
the keyboard focus, so ours would take it from the control it describes and from a field being typed in. A
`Tooltip` inside the control writes the window's one tooltip (`TooltipLayer`), and `DomeWindow` draws it as its
last layer, over every content, with no pointer area. It opens after `delay` (`tooltip-delay`), at once on keyboard
focus and at once again within `skip-delay` of another one closing (`tooltip-skip`, as Radix); it closes when the
pointer leaves and the focus goes, on a press and with Esc; over a disabled control it watches the pointer itself,
to give the reason. Placed by `Placement` (`side`, `align`, `gap`, `margin`, flip and shift; bottom and centred by
default), at most `max-bubble-width` wide, wrapping. `TooltipBubble` is the bubble: text, an optional shortcut, free
content. `Button` and `ToggleButton` take `tooltip` and `tooltip-shortcut`; an icon-only one says its
`accessible-label` by default, and the tooltip is also its `accessible-description`. Still open: a tooltip inside a
popup (a menu item's reason) is drawn under the popup; scrolling does not close it yet.

### 5.18 Dialog

Modal, centred: `raised`, `edge`, radius `panel`, level 2, `scrim` behind. Sizes `DialogSize`: `small` (400),
`medium` (520), `large` (680). Parts: title (`title`), body (`body`, wraps), footer with the buttons
right-aligned, **primary first** (Windows order: «Move to Recycle Bin» then «Cancel»). Buttons are `large`
(32) in both densities: dialogs keep the normal sizes. Esc and the window's close = «Cancel». The default button (Enter) is declared per dialog; for an
irreversible action it is «Cancel».

### 5.19 InfoBar

A full-width message under the toolbar (32 / 28): `*-soft` fill of its state, a 16 px state icon, text
`body` `ink` (one sentence: what happened, what to do), an action as a `ghost` button, a close `x` when it can
be dismissed. Kinds: `info` (accent-soft and `info`), `warning`, `danger`, `success`.

### 5.20 Toast

The result of something you did or that ended by itself («3 items moved to the Recycle Bin · 4.2 GB freed»). In the
bottom-right corner, 16 px from the right edge and 16 px above the bar under it (§ 3.11), 360 wide, level 1: icon,
up to three lines of `body` (the whole text in the tooltip), an optional action (`ghost`, «Undo», «Show details»)
and a 20 px `x` to close it. «Undo» sits right of the text; other actions, at most two («Show in File Explorer»,
«Open», «Choose where to save…», «Retry»), sit in a row under it in `accent-text`.

- **One at a time.** A new one replaces the one shown; results that arrive together become one notice («2 actions
  done · …»). Ctrl+Z still undoes an action whose notice has gone (§ 6.5).
- **Time.** As long as it takes to read: 2.5 s plus 60 ms per character, at least 4 s (8 s with «Undo»), at most
  10 s; it stays while hovered or focused. A failure («Show details») stays until closed.
- **Waits.** While a popup or a dialog is open it is not shown and its time does not run; it appears with its full
  time when they close. In Settings it appears inside the dialog, above the buttons.
- **Covers.** Only content, and only for those seconds: never a bar, a popup or a dialog. Esc or `x` closes it.

### 5.21 ProgressBar and UsageBar

- `ProgressBar`: the 2 px scan line under the toolbar (`accent` on transparent), determinate when the MFT size is
  known, else an indeterminate 30 % segment sliding (`slow` × 6, linear; static at 30 % with reduced motion).
- `UsageBar`: drive usage, 4 px (picker) or 6 px (drive cards), radius `bar`, `bar-track` / `bar-fill`, `warning`
  fill above 90 %.

### 5.22 EmptyState

Centred in its area: a 32 px icon `ink-faint`, `display` title, `body` `ink-muted` one-line explanation, one action.
Examples: «No matches» / «Nothing matches “*.psd” on C:» / «Clear search».

### 5.23 Card and DriveCard

`Card`: `surface`, 1 px `line`, radius `card`, `card-pad`. `DriveCard` (empty state): 240 wide, `hard-drive` 20 px,
«C:» `title` + label `body`, «NTFS · fast scan» `caption`, `UsageBar` 6 px, «931 GB of 953 GB used» `caption`,
the action button.

### 5.24 Splitter and ScrollBar

- `Splitter`: 1 px `line`, 6 px hit area, `col-resize` / `row-resize` cursor, `ink-faint` line while hovered or
  dragged; double-click restores the default position.
- `ScrollBar`: 10 px area over the content's edge, thumb 6 px (8 while hovered) radius 3, `scroll-thumb` /
  `scroll-thumb-hover`, no arrows, no track. Shows while scrolling and while the pointer is over the area.

| ScrollBar | |
| --- | --- |
| Interface | `orientation` (Slint's `Orientation`, vertical by default), `value` (the content's offset, as Flickable's `content-y` / `content-x`: written by both the bar and the content), `content-length`, `visible-length`, `enabled`; `scrolled()` when the user moves the content with the bar, never for a change made by the screen |
| Placement | laid over one edge of the scrolled area; its thickness is `thickness` (default `scrollbar`), its length is given where it is placed |
| Thumb | as long as the track × visible / content, never shorter than `thumb-min` (24, pending design); `thumb-thickness` at rest, `thumb-thickness-hover` while the bar is under the pointer or pressed; `thumb-color`, `thumb-hover-color` |
| Visibility `ScrollBarVisibility` | `scroll` (default): while the content moves and while the pointer is over the bar, then it fades `hide-delay` later (default `scrollbar-linger`, 1 s, pending design); `auto`: always while the content overflows; `always`: always, the track and arrows even when the content fits (no thumb then) |
| Track | drawn while the bar is under the pointer or pressed, and always with `always`; `track-color` (default `scroll-track`: transparent, so no track) |
| Arrows `ScrollBarArrows` | `none` (default), `on-hover` (shown while the bar is under the pointer or pressed, as on Windows 11), `always`. Each is a square of the bar's thickness at an end, a filled triangle of `arrow-size` (`scroll-arrow`, 6, pending design) in the thumb's colour; the arrows keep their room while hidden, so the thumb does not move when they appear. A press moves `step` (`scroll-step`: one `row`, pending design) |
| Track press `ScrollBarTrackClick` | `page` (default, as Windows): one visible length towards the pointer; `jump` (as Radix and macOS): the thumb's centre comes under the pointer, ready to drag. Shift+click always jumps |
| Holding | a held arrow or page repeats after `repeat-delay` (400 ms), every `repeat-interval` (50 ms), both pending design; paging stops once the thumb has reached the pointer |
| Pointer | dragging the thumb moves the content with it; the wheel over the bar scrolls along its axis, and a vertical wheel over a horizontal bar scrolls it too (Windows) |
| Disabled | faded (`disabled` opacity), takes no pointer |
| Accessibility | none of its own: Slint 1.19 has no scroll bar role, so screen readers and the keyboard scroll the content itself (as with Radix) |

### 5.25 ToolBar, StatusBar, PanelHeader

Structure components that only lay out others with the right height, ground and padding; `ToolBarSeparator` is
1 × 16 px `line` with `group-gap` / 2 on each side. Status bar items: `caption` `ink-muted`, separated by 1 × 12
`line`; the live dot is 6 px round. From the left: the drive's counts and when it was scanned (or the scan's
progress), the selection; then, at the right, the running operations (§ 5.29), «Live», the scan mode, the unit.
Nothing in it wraps: at narrow widths the left texts shorten first, then the operations' label (§ 3.11).

### 5.26 Kbd

A key cap for help and tooltips: `caption`, 1 px `line`, radius `badge`, 4 px side padding, `ink-muted`. Italian
key names on an Italian Windows: Ctrl, Maiusc, Alt, Canc, Invio, Esc, Tab, Backspace (Windows' own Italian
labels).

### 5.27 DriveTabs

One tab per **local drive, always present** (they are few and fixed; a USB drive appears when plugged in and
leaves when removed, its scan kept until then), plus one **closable** tab per folder or network path the user
opened. Switching tab is instant: each scanned tab keeps its model, selection, expanded folders and zoom.

- **Content**: icon 14 (`hard-drive`, `folder` for a folder tab, `network` for a network path), the letter (`body-strong`, or the folder's name),
  the label (`caption`, `ink-faint`; `ink-muted` when active), a 24 × 4 usage bar (`bar-fill`, `warning` above 90 %
  full; none for folders and for drives not scanned yet), and for a closable tab an `x` (16 × 16).
- **States**: active (joins the toolbar: `bg` fill), rest (`ink-muted` on the frame), hover (`control-pressed`
  fill), not scanned (letter and icon `ink-faint`; selecting it shows the «not scanned yet» state of § 2.2),
  scanning (a 2 px progress arc instead of the icon), live updates paused (the bar in `ink-faint`). The tooltip
  says letter, label, file system, used of total.
- **Start** (owner, 2026-11-24; called «Start» in Italian too): a tab (`house` and the word) before the drive tabs, never closed: every drive and recent folder at a glance and the ways to choose what to analyze (§ 6.16). The app opens there the first time; afterwards it opens where the user left it. Once a drive has been scanned, the drive cards become the All drives chart (§ 4.7; artboard «StartDopo»): one row per drive on a shared scale, with «Open» or «Scan», then the recent folders.
- **«+»**: a small ghost icon button after the tabs; opens the `DrivePicker` menu (§ 5.5). Ctrl+T opens it.
- **Keyboard**: Ctrl+Tab / Ctrl+Shift+Tab next / previous tab, Ctrl+W closes a folder tab (drive tabs do not
  close), Ctrl+Shift+T reopens the last closed one.
- **Placement** (owner, 2026-11-24): the title bar, if test 1 passes; otherwise the toolbar. The three options stay in
  the mockup («schede» in the Tweaks):
  - **title bar** (proposal): as Explorer, Terminal and browsers on Windows 11. No extra height. Needs DiskDome's
    own title bar (§ 12, test 1). The window title «DiskDome — C:» stays the window's name for the taskbar.
  - **toolbar**: at its left, letters only (labels in the tooltip), search narrows to 220 px. No custom title
    bar; tight at 960 px.
  - **own row**: a 30 px strip under the native title bar. Simplest; costs one table row.
- **Memory**: every scanned tab keeps its model (C: is ~0.9 GB, `docs/architecture.md`). If the total passes a limit
  (to be set after test 2, § 12), the least recently used tab releases its model and shows «Scanned 10 min ago ·
  Rescan (2 s)»; an MFT rescan is that fast.

### 5.28 ViewSwitch

The `SegmentedControl` of the three layouts in Explore's bar (§ 2.1), 22 px tall there: `list-tree`
«Table», `panel-bottom` «Both», `layout-dashboard` «Map». Words, not icons alone: the views must be obvious to
anyone. Ctrl+Alt+1 / 2 / 3.

### 5.29 ActionProgress

Every running operation (owner, 2026-11-24: actions are never disabled; each shows its progress): moving, deleting,
copying, comparing identical copies, scanning another drive. It lives **in the status bar**, never floating over
the window (owner, 2026-11-25, § 3.11):

- **The item**, at the right of the status bar before «Live»: an icon (`trash-2` deleting, `folder-input` moving or
  copying, `hard-drive` scanning or comparing, `share` exporting, `layers` for several), what it does («Moving 3 items to the Recycle
  Bin») or «3 operations», a 48 × 4 px bar and the percentage (several together: weighted by size). Its tooltip
  lists each one with its percentage.
- **A click** opens a popover upward (level 2, 360 px, 4 px above the status bar, aligned on the item's right
  edge): «RUNNING», then one row per operation with its icon, what it does, «Stop», a 4 px bar, the numbers
  («1.2 of 4.2 GB · 40%») and the time left; at the bottom «When they end a notice appears in the bottom right.
  You can keep working.» Esc or a click outside closes it.
- **When one ends** its row leaves and a notice reports the result (§ 5.20: «3 items moved to the Recycle Bin ·
  4.2 GB freed», with «Undo»; a failure stays until closed, with «Show details»). The item leaves with the last one.
  The model and the parents' totals update when each item is done; the rows being moved or deleted are
  `pending` in the table (§ 5.10).

## 6. Patterns

### 6.1 Selection

Owner, 2026-11-25 (card «Selezione multipla»; artboard «Selezione», and working in «Main»).

- **One selection shared by every view**: selecting in the tree selects in the map and in the file list (when the
  item is there), and the reverse. Each tab (drive or folder) keeps its own; Back restores it.
- **Table** (Tree, File list): click selects one; Ctrl+click adds or removes; Shift+click selects from the anchor
  (the last plain or Ctrl click) to here; Shift+arrows extend; Ctrl+arrows move the cursor without selecting and
  Ctrl+Space toggles the cursor's row (Windows' rule; the cursor is not the selection); Ctrl+A selects the items of
  the cursor's level; Esc clears.
- **Map**: click selects one; Ctrl+click adds or removes; **no area selection** (owner: dragging stays free to drag
  files to File Explorer). Every selected rectangle has its own outline (§ 4.4).
- **Right-click** on a selected item opens the menu for the whole selection; on another item it selects that one.
- **Nested items are counted once** (owner): with a folder and something inside it both selected, the inner item
  keeps a lighter highlight (`accent-soft` at 45 %, name in `ink-muted`) and the tooltip «Already included in
  «Users»: counted once»; totals and actions take the folder only.
- **Explore bar with several items** (§ 2.1): no swatch, «2 items · 349 GB», then a `Badge` «1 already included»
  when nested items exist, a `warning` badge with `shield-alert` «1 system item» when some are system items, and
  a ghost `x` «Clear selection (Esc)»; the actions stay the same and apply to all.
- **Menu with several items** (§ 6.3): a header line «2 items · 349 GB»; the verbs that need one item (Open
  terminal here, Zoom here in the map, Rename) are disabled with the reason «one item at a time» on the right;
  «Copy full path» copies one path per line; «Show in File Explorer» opens one window per folder.
- **System items** stay in the selection but every deletion leaves them out until included (§ 6.4).
- **After an action**: what succeeded leaves the selection; what failed stays selected and the report says why.
- **Filters and search that hide selected items** (owner, 2026-11-25: «nothing hidden, nothing the user does not
  understand»): the items **stay selected** and are shown apart, never silently dropped. The Explore bar adds a
  `neutral` badge with a `filter` icon after the name, «+1 hidden by the filters» (or «by the search»); its tooltip
  names them with their sizes and says «Actions leave it out; when deleting you can include it». The badge opens a
  menu: «Remove from the selection», «Clear the filters» (or «Clear the search»). A notice says it once, when it
  happens: «"docker_data.vhdx" is selected but hidden by the filters: it stays selected and actions leave it out».
  The status bar adds «1 hidden, left out of actions»; the map keeps its outline on the (dimmed) rectangle. Every
  action takes only what is visible; deleting asks (§ 6.4). With only hidden items selected, the bar says «Nothing
  visible» with the badge. Clearing the filters brings them back into view, still selected.
- **Status bar and Narrator**: «2 selected · 349 GB · 1 already included, counted once»; Narrator reads «2 items
  selected, 349 GB, plus 1 already included».

### 6.2 Keyboard

| Keys | Action |
| --- | --- |
| ↑ ↓ PgUp PgDn Home End | move in the table |
| → / ← | expand / collapse, or go to the first child / the parent |
| Num * | expand the whole subtree |
| Enter | open the file; zoom the treemap to a folder when the treemap has the focus |
| F2 | rename (inline) |
| Canc / Del | move to the Recycle Bin (confirmation) |
| Maiusc+Canc / Shift+Del | delete permanently (confirmation) |
| Ctrl+C | copy the files (as Explorer) |
| Ctrl+Maiusc+C / Ctrl+Shift+C | copy full path (Windows 11's «Copy as path») |
| Ctrl+X, Ctrl+V | move (cut, then paste into the selected folder) |
| Ctrl+Maiusc+M / Ctrl+Shift+M | Move to… (§ 6.5) |
| Ctrl+Z | undo the last file action (§ 6.5), up to 10 in the session |
| Ctrl+A | select the items of the cursor's level (§ 6.1) |
| Ctrl+Maiusc+1 … 9 / Ctrl+Shift+1 … 9 | sort the table by its n-th visible column; again reverses (§ 4.3) |
| Tab (from the rows) | the table's header: ← → columns, Enter sorts, Ctrl+← → moves, Shift+← → resizes, Shift+F10 «Columns…» (§ 5.10) |
| Esc | clear the selection; cancel a cut; close a menu or a dialog |
| Alt+Invio / Alt+Enter | Windows Properties |
| Ctrl+E | show in Explorer |
| Ctrl+Maiusc+E / Ctrl+Shift+E | Export… (§ 6.18) |
| Ctrl+F | search |
| F5 / Ctrl+F5 | rescan the folder / the drive |
| Ctrl+1 … 6 | sections: Overview, Explore, Clean-up, Timeline, Statistics, Activity (§ 6.11) |
| Ctrl+Alt+1 / 2 / 3 | Explore's layout: Tree / Both / Map |
| Ctrl+Alt+T / L | Explore's table: Tree / File list |
| Alt+← / Alt+→ | back / forward (section, filter, selection, zoom) |
| Ctrl+K | command palette (§ 6.19) |
| Ctrl+\ | keyboard shortcuts (§ 6.19) |
| F1 | Help, in the browser (§ 6.19) |
| Ctrl+O | scan a folder |
| Ctrl+Shift+F | the Filters panel (§ 6.6) |
| Ctrl+Tab, Ctrl+Shift+Tab | next / previous drive tab |
| Ctrl+T, Ctrl+W, Ctrl+Shift+T | «+» menu, close a folder tab, reopen the last closed one |
| Backspace, Alt+↑ | treemap: up one level |
| Ctrl+Alt+P | show or hide the side panel |
| Ctrl+, | settings |
| Menu key, Maiusc+F10 | context menu |
| F6 | move the focus between toolbar, table, side panel, treemap |

Shortcuts are declared with Slint's `KeyBinding` (1.16) and shown in menus with the menu item's `shortcut`. They use
keys that need no Shift on the Italian keyboard (owner, 2026-11-25): «/», «?» and «=» are Shift+7, Shift+' and
Shift+0 there, so the shortcuts list is Ctrl+\ (the key left of 1), not Ctrl+/. A binding matches the character,
not the key's position, so Ctrl+\ works on other layouts too (above Enter on a US keyboard).

### 6.3 Context menu (tree, file list, map, lists)

In Explorer's order, the verbs Windows uses (Italian on an Italian Windows):

1. **Open** (Enter) · Show in Explorer (Ctrl+E) · Open terminal here · Windows Properties (Alt+Enter); for a link or
   a junction, **Go to the target** (Enter) comes first (§ 5.10)
2. Copy (Ctrl+C) · Copy full path (Ctrl+Shift+C) · Copy name · Copy parent folder · Copy the selection as a list ·
   Export the selection… (§ 6.18)
3. Rescan this folder (F5) · Zoom here in the map · Show in the tree · Filter: this extension, this folder
4. Rename (F2) · Move to… (Ctrl+Shift+M) · Cut · Paste
5. Move to the Recycle Bin (Del) · **Delete permanently** (Shift+Del, `danger`)

For several items, the verbs that need one item (Rename, Open terminal) are disabled with their reason.

The copy verbs with several items (owner, 2026-11-25): «Copy full path», «Copy name» and «Copy parent folder» put
one line per item (each parent once). «Copy the selection as a list» puts a tab-separated table with a header row —
Name, Folder, Size (as shown, in the units setting), Bytes (exact), Allocated, Modified (ISO 8601 date and time) —
one row per item in the order on screen, nested items counted once, so it pastes into Excel or a document; the
notice says «12 rows copied». Paths over 260 characters are copied whole.

### 6.4 Deleting (rule 3)

Nothing is deleted without a confirmation; the Recycle Bin is the default; system items ask twice.

1. **Move to the Recycle Bin** (Del): dialog `medium`. Title «Move 3 items to the Recycle Bin?». Body: up to 5
   rows (icon, name, size; «and 12 more»), the total «4.2 GB». If some items are too large for the Recycle Bin,
   Windows would delete them permanently: a `warning` InfoBar inside the dialog says so and lists them. They are
   left out (owner, 2026-11-25): the primary button acts on the others and says so («Move the other 2 to the
   Recycle Bin»), and the InfoBar's ghost button «Delete these permanently…» opens the permanent-delete
   confirmation (point 2) for them alone, with «Cancel» as the default.
   Buttons: «Move to Recycle Bin» (`primary`, default), «Cancel».
2. **Delete permanently** (Shift+Del): title «Delete 3 items permanently?», body as above plus «You won't be able
   to recover them.» Buttons: «Delete permanently» (`danger`), «Cancel» (default: Enter does not delete).
3. **System items in the selection** (`Windows`, `Program Files`, `pagefile.sys`, `hiberfil.sys`, `System Volume
   Information`, the NTFS metafiles…): the first dialog shows a `warning` InfoBar «2 items are system files:
   Windows may stop working.», and they are **left out** unless the user ticks «Include system items»; unticked,
   the button acts on the others and says so («Move the other 11 to the Recycle Bin»), and with only system items it
   is disabled. Ticked,
   the button reads «Continue» and a second dialog follows: title «Delete system files?», only the system items
   listed, «Delete system files» (`danger`), «Cancel» (default). For `pagefile.sys` and `hiberfil.sys` the
   dialog says they are usually locked while Windows runs and points to Windows' own settings (virtual memory,
   hibernation) to shrink them.
4. **Selected items hidden by filters or the search** (§ 6.1, owner, 2026-11-25): as system items, they are
   **left out** unless included. A neutral InfoBar (`filter` icon) under the list: «"docker_data.vhdx" (48.5 GB) is
   selected but hidden by the filters: it is left out, unless you include it.», and a checkbox «Include the hidden
   one too» («Include the 2 hidden ones too»). Ticked, they join the list with a «hidden» badge and the title and
   total count them; if one is a system item, point 3 applies. With only hidden items and the box unticked, the
   title says «Nothing visible to move to the Recycle Bin» and the primary button is disabled.
5. **Special entries in the selection** (owner, 2026-11-25): a neutral InfoBar (`info`, on `control`) between the
   list and the body, one line, the first that applies:
   - a link or junction: «"Progetti DaVinci" is a link: the action touches only it, not "D:\Video\Progetti DaVinci",
     which it points to.»;
   - an online-only file: «"Backup telefono 2023.zip" is online only: you also remove it from OneDrive, and you free
     no space on this drive.»;
   - a second hard-link name: «"Montaggio vacanze 2025.mp4" is another name of "Videos\Montaggio vacanze 2025.mp4":
     the space is freed only by deleting that one too.»
   The buttons and the Recycle Bin rule do not change; the notice reports what was really freed («0 bytes freed»).
6. While the file system works: the rows are `pending` (§ 5.10); then the model and every parent's totals update at once
   (`docs/architecture.md`), and a toast reports the result («3 items moved to the Recycle Bin · 4.2 GB freed»)
   with «Undo» (§ 6.5).
   A failure for some items: a toast `danger` with «Show details», listing each item and the reason.

### 6.5 Rename, move, copy, drag, undo

Owner, 2026-11-25 (card «Azioni sui file»; artboard «AzioniFile»; Rename, Move to…, Open above 10 and Undo work in
«Main»). Rule 3 everywhere: nothing is lost without the user knowing; rule 4: every action runs with the user's
rights.

- **Rename (F2)**: in the table the name cell becomes a `TextField` in place, the name selected without its
  extension; in the Map layout a small `Popover` with the same field opens over the rectangle. Enter confirms,
  Esc cancels, a click elsewhere confirms. Errors in the field, which stays open: the characters `\ / : * ? " < > |`,
  Windows' reserved names (CON, PRN, AUX, NUL, COM1–9, LPT1–9), a trailing dot or space, a name already used in the
  folder (case ignored). Errors after Enter (file in use, permissions): an `InfoBar` under the row with the reason,
  the old name back. The model, the tree's order and the map update at once.
- **Move to… (Ctrl+Shift+M)**: dialog `medium` with the items (up to five, «and 7 more», the total), «Move to» with
  a path field and «Browse…» (Windows' folder picker), the last 5 destinations with their free space, and one line
  on what will happen: on the same drive the move is immediate; on another drive or a network folder DiskDome
  copies each file and then deletes the original, and if interrupted what is not yet copied stays where it is.
  When the destination has not enough space, a `danger` InfoBar says how much is free and how much is needed, and
  «Move» is disabled. Progress in `ActionProgress` (§ 5.29) with «Stop».
- **Conflicts**: «A file with the same name», with both files (size, modified, «newer»), and the choices «Keep
  both» (`primary`: adds « (2)»), «Replace» (the existing one goes to the Recycle Bin, and the dialog says so),
  «Skip», «Cancel all»; «Do the same for the other 6 conflicts». For folders «Merge» replaces «Replace» and the
  files inside go through the same choices.
- **Copy, Cut, Paste**: Ctrl+C puts the files on the clipboard as File Explorer does (paste them in Explorer, on the
  desktop or in another DiskDome tab); Ctrl+X cuts: the rows stay at 55 % opacity until pasted, Esc cancels;
  Ctrl+V pastes into the selected folder, or the selected file's folder, from DiskDome or from Explorer. A copy in
  the same folder takes Windows' name («Photos - Copy»). Ctrl+Shift+C copies the path, not the files.
- **Drag and drop**: dragging the selection shows a ghost «2 items · 73.5 GB». Windows' rules: same drive moves,
  another drive copies, Shift always moves, Ctrl always copies. Over a folder row, or a drive tab, the row is
  highlighted with the verb in `accent-text` («Move into "Archive"», «Copy to "D:"») before the drop. A folder
  dropped from Explorer outside the rows shows a dashed `accent` frame «Analyze "Projects 2021"» (§ 6.16). Dragging
  works between DiskDome and Explorer because the interface is never elevated (Windows blocks it across integrity
  levels).
- **Open above 10 items** (decision 24): dialog `small` «Open 12 items?», «Each one opens with its program: there
  may be 12 windows.», «Open 12 items» (`primary`), «Cancel». Up to 10, no question.
- **Partly done**: a `warning` toast «8 of 10 moved to Z:\Archive · 2 not», with «Show details»: a dialog listing
  each item with its reason in plain words and what to do («In use: OBS Studio is writing it. Close OBS and try
  again.»), «Retry» or «Show in File Explorer» per row, «Retry all». What failed stays selected and in place.
- **Undo**: every file action's notice has «Undo» for at least 8 seconds (§ 5.20); Ctrl+Z undoes the last action, then the previous,
  up to 10 in the session. Recycle Bin: restores to the original folder; Move, Cut and Paste: brings back; Rename:
  the old name; Copy and Paste: the copy goes to the Recycle Bin. Delete permanently cannot be undone (that is why
  its default button is «Cancel»). When the files changed in the meantime, Undo does nothing and says why.

### 6.6 Search and filters

Owner, 2026-11-25 (card «Filtri»; artboard «Filtri», and working in «Main»).

**Search** filters every view to the items whose name matches; the tree keeps their ancestors (in `ink-faint`),
expanded. The tab bar's summary says «1.204 matches · 96 GB». Wildcards, regex and case as in § 5.4.

**Filters** narrow what every view shows, by the files' properties.

- **The Filters panel** (toolbar «Filters», Ctrl+Shift+F): a 640 px panel under the button, not a dialog (`raised`,
  `edge`, level-2 shadow). Changes apply at once: there is no «Apply», and the table, the map and the numbers change
  behind it. Esc or a click outside closes it; the filters stay. Groups, two columns:
  - **Saved filters** (when there are any), on top: one `Chip`-like button each (`bookmark` icon), the active one
    `accent-soft`; on hover a `pencil` to rename (in place) and an `x` to delete (the notice has «Undo»).
  - **Type**: the twelve categories as toggle chips with their swatch (`control`, on: `accent-soft` /
    `accent-text`); «Extensions» field (`.mp4, .mkv`, Enter adds).
  - **File size**: Any · ≥ 100 MB · ≥ 1 GB · ≥ 10 GB · < 1 MB, and «From … to» fields («500 MB», «2 GB»).
  - **Path**: «Inside» (a folder name or a piece of path) and «Exclude» (Enter adds; each excluded folder a chip).
  - **Modified**: the six age buckets of the map (§ 4.5) as toggle chips with the age swatches, and «From … to»
    dates.
  - **Attributes**: Hidden, System, Online only (OneDrive), Links, Compressed by NTFS, each a
    `SegmentedControl` «All · Only · Without».
  - **Owner**: All and the owners found on the drive; «Not available folder by folder» in that scan mode.
  - Footer: «4 files · 27.9 GB of 668 GB» (live), «Clear filters» (ghost), «Save these filters…» (secondary,
    disabled with no filter), which turns into a name field with «Save» and «Cancel».
- **Rules**: within a group one choice is enough (Video *or* Images); across groups all must hold (Video *and*
  ≥ 1 GB). Size, dates, type and owner are the file's: a folder stays when at least one file inside passes.
  Hidden, System, Compressed and Path also apply to folders and everything they hold.
- **What changes**: folders show the total of what passes, and % is of the filtered folder; the map keeps its
  shape and dims what does not pass (nothing moves, § 1); the File types panel, the status bar («With filters:
  1,739 of 3,803,268 files · 55.1 GB») and every section of the tab count only what passes (Overview, Explore,
  Clean-up, Timeline, Statistics, Activity). The free space stays Windows'. While a scan reads, filters apply to
  what was read.
- **Filter bar** (§ 2.1, under the toolbar, only when a filter is active): a `filter` icon, one `Chip` per
  filter («Type: Video», «Size: ≥ 1 GB», «Modified: more than 1 year ago», «Without system items», «Owner: stefa»,
  «Path: Downloads», «Without: Windows»); a click opens the panel at that group, `x` removes it. On the right
  «1,739 files · 55.1 GB of 668 GB» (`ink-muted`), «Save…» and «Clear filters» (ghost). The Filters button shows
  the count as a `neutral` badge.
- **Selection**: selected items that a filter hides stay selected, shown apart in the Explore bar («+1 hidden by the
  filters»), and actions leave them out unless included when deleting (§ 6.1, § 6.4).
- **Per tab**: each tab keeps its filters; Back restores them.
- **Nothing passes**: an `EmptyState` «Nothing with these filters» · «No file on this drive passes every filter.»
  with «Change the filters» and «Clear filters».

**Written filters** in the search field, with the name: `ext:mp4 size:>1GB vacanze`. A space means «and», a comma
«or» inside a value, a minus excludes (`-attr:system`, `-path:Windows`).

| Key | Examples | Filters |
| --- | --- | --- |
| `ext:` | `ext:mp4,mkv` | extension |
| `type:` | `type:video` | category (the twelve, in the interface language or English) |
| `size:` | `size:>1GB`, `size:<4KB`, `size:100MB..1GB` | file size; B, KB, MB, GB, TB |
| `modified:` | `modified:<2024`, `modified:>2026-11-01`, `modified:2025` | modification date: before, after, during; also 01/11/2026 |
| `age:` | `age:>1y`, `age:<7d` | how long ago: d, w, m, y |
| `attr:` | `attr:hidden`, `-attr:system` | hidden, system, online, link, compressed |
| `owner:` | `owner:stefa` | owner (MFT scans only) |
| `path:` | `path:Downloads`, `path:Users\stefa`, `-path:Windows` | a folder with that name, or a piece of path |

- Keys are English in both languages (as in Everything and WizTree); Italian aliases work too (`est:`, `tipo:`,
  `dim:`, `modificato:`, `età:`, `proprietario:`, `percorso:`).
- A complete filter becomes a chip when Space or Enter follows it; the name stays in the field. Backspace in an
  empty field removes the last chip.
- **Suggestions** while typing, in a list under the field (`raised`, level-1 shadow): the keys with an example,
  then the values really on the drive (extensions with their size and count, categories, owners, attributes,
  common sizes and ages); click or Enter picks one.
- **Errors** block nothing: the field's edge turns `danger` and a bubble under it says what is wrong and what to
  write («"sise:" is not a filter. Did you mean "size:"?», «size: a number with its unit, e.g. size:>1GB»); the
  rest of the search works.

**Saved filters**: «Save these filters…» in the panel or «Save…» in the bar, with a name proposed from the chips.
They are listed on top of the panel and in the command palette (Ctrl+K), for every drive and folder; a click
replaces the active filters. Three come ready at first launch, to keep or delete: «Large files» (≥ 1 GB), «Large and
old» (≥ 100 MB, more than a year ago), «Videos and photos».

**Where filters come from** (§ 6.11, drill-down): every bar, row, dot or cell outside Explore opens Explore with the
matching chip: Statistics' «Hidden files» → «Only hidden»; a size band → «Size: 100 MB–1 GB»; a Timeline month →
«Modified: November 2026»; a double-click on `.mp4` in File types → «Extension: .mp4»; an owner → «Owner: SYSTEM»;
«Show» on «Large and old» in Clean-up › To review → its two chips. Back returns with the filters of before.

**Keyboard**: Ctrl+Shift+F opens and closes the panel; inside, Tab between groups, arrows between chips, Space
chooses, Esc closes. In the bar, Tab moves between chips, Del or Backspace removes one, Enter opens the panel at its
group. Narrator: «Filters, 3 active: type video, size at least 1 GB, without system items. 4 files, 27.9 GB».

### 6.7 First launch: the scan mode

**Decided (owner, 2026-11-24), replacing the first-launch dialog**: no dialog at first launch. The recommended mode
is already chosen and said in one line under the call to action («DiskDome reads the drive's index: Windows will
ask for permission» · «Change»); «Change» opens three `OptionCard`s below the line, in place:

1. **Drive's index, asks each time** · Recommended — «Reads the drive's index (MFT). Windows asks for permission
   (UAC) at every scan.»
2. **Drive's index, never asks** — «Installs a small service once that reads the index. Asks for permission only now.»
3. **Folder by folder** — «No permissions. It takes longer and some system folders can't be read.»

«You can change it in Settings.» under them. The UAC prompt appears only when the user presses «Scan». Artboards
«Start» and «PrimaVolta»; the old dialog stays on the canvas as «Avvio», no longer used.

### 6.8 The Clean-up view

Owner, 2026-11-25 (card «Pulizia guidata completa»; working in «Main», section Clean-up). The Clean-up section (Ctrl+3)
is the guided clean-up of § 4.8: one list in groups, the drive of the tab, nothing preselected (owner, 2026-11-24).

- **Header** (recycle icon, `accent-text`): «You can free up to 186 GB: 167 GB here, 19.1 GB with Windows' tools» and
  one caption: «Nothing is chosen for you: tick what to remove. Everything goes to the Recycle Bin.» The total is the
  same as Overview's «You can free» card, which lists the same groups.
- **Table** columns: checkbox (34), Name (240, icon), Where (the folder, middle-elided, whole in the tooltip), Size,
  Modified, and a ghost «Show» on hover (opens Explore with the item selected). Rows `row`, sorted by size.
- **Groups**, each a 34 px sticky header on `bg` with a chevron (collapse), a tri-state checkbox for the whole group
  (`minus` when some are ticked), the group's icon in `accent-text`, its name in 700, «why it is safe» in `caption`
  `ink-muted` (whole in the tooltip), the ticked sum in `accent-text` and «24 items · 147 GB» on the right:
  1. **Rebuildable folders** (`recycle`): «They are rebuilt by the next build or install: node_modules, target, the
     npm, pnpm, cargo and gradle caches.»
  2. **Temporary files** (`clock`): the user's `%TEMP%`. «Programs create them and forget them. Files in use and files
     modified in the last 24 hours are skipped.»
  3. **Downloads not modified for over a year** (`arrow-down`): one row per file or folder in Downloads.
     «Downloaded and never touched again: look at the list before removing them, they do not come back by
     themselves.»
  4. **Identical copies** (`copy`, owner 2026-11-25: a group of Clean-up, card «Duplicati»), below.
  A group shows its first 8 rows and «Show the other 16 · 44.8 GB». An empty group keeps its header with «Nothing on
  D:» on the right, so the user sees what was looked for.
- **Identical copies**: files of the same size (at least 1 MB) found at once from the MFT; hard links (the same
  file) and online-only files (comparing them would download them) never count.
  - **Before comparing**: the header says «File with the same size: maybe copies. Compare the content first, then
    choose which to remove», «4 sets · up to 28.6 GB» and a secondary button «Compare the content». Each set is a
    row (`copy` icon, the name in 700, «2 files of 17.0 GB», a `neutral` badge «to compare», the space it would free
    and «to free»), followed by its copies indented by 20 px with their folder and date, the oldest marked «the
    oldest». The checkboxes are disabled («Compare the content first»): nothing that might differ can be ticked.
  - **Comparing**: a background job with its `ActionProgress` («Comparing the content of 4 sets · 26.2 GB»,
    «Stop»), reading the files with the user's rights: first a sample of the start and the end, then the whole
    content; the app stays usable. The notice at the end: «Comparison done: 3 identical sets, 1 different, removed
    from the list».
  - **After**: the badge turns `success` «identical, verified»; sets that differ leave the list, and the group's
    caption says so («1 set had the same size but different content: it is not in the list»). The checkboxes work;
    **the last copy of a set can never be ticked**, in this group or in any other (`cbx` at 35 %, tooltip «It is
    the last identical copy: you keep it»), so a copy that is also an old Download cannot be lost twice. On hover
    each copy offers «Keep this one» (ticks the others) and «Show». The group's checkbox ticks every copy but the
    oldest of each set. The confirmation adds «One identical copy always stays: "C:\Users\stefa\Videos\…"».
  - **Errors**: a file that cannot be read leaves its set with «unreadable»; a file that changes during the
    comparison marks the set «changed during the comparison: compare again».
  - The header's total counts only verified copies, once even when a copy is also in another group; before
    comparing it says «plus the copies to compare». Overview's «You can free» counts them the same way.
- **With Windows' tools** (no checkboxes, `external-link` header): the Recycle Bin («Open the Recycle Bin»), hibernation
  (`shield-alert` in `warning`, «Power options»: «if you don't use hibernation, Windows can remove it»), Windows Update
  and Windows' Temp («Disk Cleanup»: «needs administrator rights, which DiskDome does not use to touch files», rule 4).
  Each row has a secondary button with `external-link`; DiskDome only opens the tool.
- **Footer** (44 px, `bg`): «3 chosen · 43.7 GB» (or «Nothing chosen: tick what you want to remove»), «Clear the ticks»
  (ghost) and «Move to the Recycle Bin…» (`primary`, disabled with nothing ticked). It opens the usual confirmation
  (§ 6.4) with the ticked items; for Temp the dialog adds «Temporary files in use and modified in the last 24 hours
  are skipped: they stay where they are.» Cancel keeps the ticks.
- **While and after**: the rows are `pending` (§ 5.10); the notice says what was really freed and what was skipped:
  «4 items moved to the Recycle Bin · 48.7 GB freed · 214 files in use skipped (420 MB): they stay in Temp». Temp
  then shows what is left, with «in use: they stay while a program keeps them open». Items too large for the Recycle
  Bin follow § 6.4.
- **Nothing at all**: the header says «Nothing to clean on D:» and the caption «No rebuildable folder, no temporary
  file, no old download. For the largest files look at "To review" or the File list.», with «Open the File list».
- Filters (§ 6.6) apply here too.

**To review** (owner, 2026-11-25, decision 32): a 380 px column on the right, on `surface` with a left `line`: overline «TO
REVIEW» and the total, the Big and old chart (§ 4.7) and the five largest folders of its quadrant, each with its
age, size and «Show» (to Explore). One caption: they do not rebuild themselves, look at what they hold, delete them
from Explore with the usual confirmation. Nothing here has a checkbox. Without such folders the column says
«Nothing big untouched for over a year»; under 960 px it moves under the table.

### 6.9 Messages and tone

Interface text (both languages): address the user directly («tu» in Italian), short sentences, verb first on
buttons, no exclamation marks. Errors say what happened and what to do: «Can't read the MFT of D:: the drive is not
NTFS. Scan it folder by folder.» with the action «Scan folder by folder». No codes in sentences (the engine sends
codes; the interface words them; details go to the log).

**Global terms where Italian users use them** (owner, 2026-11-24): «Start», «Home», «Timeline» stay in English in
the Italian interface; Italian everywhere else.

**Say what DiskDome does and what it found, not how well it does it** (owner, 2026-11-24). No timings or speed to
show off in the interface: not «Scanned in 6.8 s», «a few seconds», «fast». While something works, a progress bar
tells how it is going (percentage, what it is doing, numbers growing, time left); when it ends, a short notice says
what was found or what changed («Analysis of C: complete · 3,789,066 files»). Scan modes are named by what they do
(«Drive's index», «Folder by folder»), not by speed; the speed difference is said once, where the user chooses
(«it takes longer»). Timings belong in the log and in `diskdome-cli`. Each screen's purpose, and what it says first:
artboard «Scopo».

### 6.10 Settings

Owner, 2026-11-25 (card «Impostazioni»; working in «Main»: the toolbar's `settings` button, Ctrl+,).

A dialog 780 × 560 (at most the window minus 16 px), `raised`, level-3 shadow, over the `scrim`. On the left a 168 px
column on `bg` with the overline «SETTINGS» and six sections (the current one `control` with a 2 px `accent` mark on
its left); on the right the section, scrolling; at the bottom a 48 px bar. **Changes apply at once**, the window
behind changes with them: no OK, no Apply, no Save. Esc or «Close» closes it. It opens at General; «Change» next to
the scan mode (Start, § 6.7), the scan mode and the units in the status bar, and «Change in Settings» in the map's
menu open it at their section.

Each setting is a row: the label (150 px) on the left, the control, then one `caption` line on what it does.

| Section | Settings |
| --- | --- |
| **General** | Language: «Like Windows (Italian)» · Italian · English (the default follows Windows: Italian if Windows is Italian, otherwise English; applies at the next launch). Units (§ 4.1): «Like File Explorer» · «Binary (GiB)» · «Decimal», with an example line («A 1 TB drive shows 931 GB, the same numbers as File Explorer»). CSV files (§ 6.18): «Like Windows ( ; )» · Comma · Semicolon · Tab, the separator of exported CSV files; «Like Windows» reads the regional list separator, which is what Excel expects on a double-click. Notifications: a switch «Tell me when a scan ends and DiskDome is not in front» (on). |
| **Appearance** | Theme: «Like Windows» · Dark · Light (follows Windows even when it changes). Density: Normal · Compact, with a live preview of three table rows. Map style: the eight styles as 96 × 64 tiles, each a small live map of four rectangles in that style, the chosen one with a 1 px `accent` ring (Vignette the default). Map colours: Standard · Soft · Vivid. Map names: None · Few · Many · All (also in the map's menu). |
| **Scan** | Mode: the three `OptionCard`s of § 6.7. With «never asks» chosen and no service yet: an InfoBar «The service is not installed yet. Windows asks for permission once.» with «Install the service» (`primary`, `shield-alert`: the UAC prompt); installed: «Service installed and running · it only reads the index: it does not write, delete or move.» with «Remove the service» (UAC again); it stays installed if another mode is chosen, and the line says so. One caption: «The interface never runs as administrator: the permission is only for the small index reader» (rule 4). Live updates: a switch (on; «from NTFS's journal; not on folder walks and network paths»). At launch: a switch «Rescan NTFS drives when you open DiskDome» (on; folders and network paths rescan only when asked). Drives: one checkbox per drive, «shown in Start and in the "+" menu»; unticked ones are only here. |
| **Saved summaries** | «DiskDome keeps a summary of every scan, only on this PC: the last 30 and one per month. Timeline, History and "Since yesterday" use them.» One row per drive or folder: name, «31 summaries · 74.2 MB · since 24/10/2025», «Delete…». Delete asks in place: «Delete the 31 summaries of C:? Timeline, History and "Since yesterday" start again from the next scan.» with «Delete» (`danger`) and «Cancel» (default). Under the list the total and the folder («%LOCALAPPDATA%\DiskDome») with «Open the folder». |
| **Columns** | The Tree's and the File list's columns as checkable lists (the same as «Columns…», § 5.10), each with «Restore»; order and widths are changed by dragging in the table. |
| **File Explorer** | A switch «Show "Analyze with DiskDome" on folders and drives» (on; also on the background of an open folder; on Windows 11 it is under «Show more options»; a folder on a drive already scanned opens there without reading it again), and the command line `diskdome.exe <path>`, which does the same (§ 6.20). |

- **Bottom bar**: «Restore the defaults…» (ghost) and «Changes apply at once: nothing to save.» on the left,
  «Close» (secondary) on the right. «Restore the defaults…» asks in the bar: «Put every setting back as at the
  start? Saved summaries and saved filters stay.» with «Restore» (`danger`) and «Cancel» (default).
- Settings are stored on this PC with DiskDome's data (`docs/architecture.md`); nothing is sent anywhere.
- Keyboard: Ctrl+, opens; ↑ ↓ in the section list, Tab into the section; every control is reachable; Esc closes.
  Narrator reads the section's name, then each row's label, control and caption.

### 6.11 Flows and navigation

Owner, 2026-11-24: «fix the flows too: what you see before and after the analysis, the sections and how to reach
them». Artboard «Flussi»; in the «Main» artboard the sections and Back work.

**Three levels, always the same.** 1 · **drive** (the drive tabs: what to look at) → 2 · **section** (the section
bar: how to look at it) → 3 · in Explore only, the **layout** (Tree · Both · Map) and what the table lists
(Tree · File list). Search, filters and Back work at every level.

| Section | Keys | Content | Also reached from |
| --- | --- | --- | --- |
| Overview | Ctrl+1 | § 4.5: one sentence on the drive, the map, what you can free | automatically after a drive's scan |
| Explore | Ctrl+2 | the table (Tree or File list), the map, the side panel | the map and every number, row, bar or dot elsewhere; Enter in the search |
| Clean-up | Ctrl+3 | § 6.8 | «Free up space…» (Overview's primary button) and «Open Clean-up» |
| Timeline | Ctrl+4 | what changed and history (from the second scan), the last 7 days, months, calendar (§ 4.5, § 4.7) | the day bars; «See what changed» in Overview |
| Statistics | Ctrl+5 | § 4.6 | the summary in the section bar and the status bar |
| Activity | Ctrl+6 | § 4.5; empty at first: «Here you see what changes while DiskDome is open. Nothing yet.» | «Live» in the status bar |

- **Section bar**: a row at `tabbar` height under the toolbar, full width, present once the drive has been scanned:
  the six section tabs left, ordered by use (Overview, Explore, Clean-up, Timeline, Statistics, Activity), the summary «670 GB used · 284 GB free» right (a ghost button to Statistics). It
  replaces the old view tabs (Tree, Largest files, Clean-up): Largest files becomes Explore › File list.
- **Explore's bar** (full width under the section bar, Explore only): «Tree · File list» (Ctrl+Alt+T / L), the layout
  «Table · Both · Map» and the panel toggle, which therefore never appear disabled in other sections; on the right
  the selection with its main actions (§ 2.1), so nothing needs a right-click.
- **The toolbar** holds only what works everywhere (§ 2.1).
- **Drill-down**: a click on any number, row, bar or dot outside Explore opens Explore with the matching filter
  chip (or selection) already set; **Back** (toolbar's first button, Alt+←; Alt+→ forward) returns to the section,
  filter, selection, zoom and scroll position of before. History per drive tab, 30 steps.
- **Before the analysis** (a drive never scanned): no section bar, no search, filters or layout control; the
  content is the one call to action of § 2.2 («Scan C:», `hero`), the mode line with «Change», the other drives and
  «Scan a folder…».
- **While scanning**: the progress view of § 2.2 (percentage, bar, what it is doing, counters, phases, time left,
  «Stop»). With the drive's index the sections wait for the end; with a folder walk Explore fills while it reads and
  Overview arrives at the end.
- **After the scan**: Overview, the section bar, search and filters, a toast «Analysis of C: complete · 3,789,066 files in 270,761 folders. From now on changes update by themselves.»,
  the drive tab's usage bar, live updates on; «Scan again» appears in the toolbar, secondary.
- **Next launch**: the tabs come back as they were. A drive with a saved summary shows it at once with an info
  bar «Yesterday's scan at 21:14 · updating», and NTFS drives rescan by themselves (a setting, on by default for
  MFT scans, off for folder walks and network paths; owner, 2026-11-24).
- **Without administrator rights**: everything works, slower; an info bar (`warning`) says what is missing
  («214 folders could not be read: totals are lower than real») with «Change mode»; Activity is disabled.

| Part | Before the analysis | After |
| --- | --- | --- |
| Section bar | absent | the six sections |
| Content | the Home: one card per drive | the chosen section; Overview after a scan |
| Drive tab | letter and label, faint | plus the usage bar |
| Primary button | «Scan C:», large, in the middle | «Scan again», secondary, in the toolbar |
| Search and Filters | absent | in the toolbar, on the whole drive |
| Table · Both · Map | absent | in Explore's bar |
| Status bar | mode and units | plus files, scan time, «Live» |

### 6.12 Every control

The owner asked for the analysis to include every single button. This is the complete inventory, region by region
(artboard «Comandi», in Italian, from the same source: `controls.py` of the design work). «→» means it leads to
another section. A new control is added here before it is drawn; the Kit (§ 9) lists the same ids.

**Title bar** (always)

| Control | Keys | Does |
| --- | --- | --- |
| DiskDome mark | Alt+Space | opens Windows' window menu (move, size, close) |
| Drive tab | Ctrl+Tab, Ctrl+Shift+Tab | shows that drive with its selection, zoom and section as they were; drag to reorder |
| Drive tab, right-click |  | menu: Scan again · Open in File Explorer · Drive properties · Close (folders only) · Close the others |
| Tab × (folders and network only) | Ctrl+W, middle-click | closes the tab and frees its memory; Ctrl+Shift+T reopens it |
| «+» | Ctrl+T | menu: drives not open · Scan a folder… · Network path… · recent folders |
| Minimize, Maximize, Close | Win+↓, Win+↑, Alt+F4 | Windows' own; Close during a copy or move asks whether to stop it (a scan just stops) |

**Toolbar** (always; search, filters and Scan again after the first scan)

| Control | Keys | Does |
| --- | --- | --- |
| Back, Forward | Alt+←, Alt+→ | go back to the previous section, filter and selection (history per tab) |
| Scan again | Ctrl+F5 | secondary, only after the first scan (before, the call to action is in the middle); while scanning it becomes «Stop» |
| Search for a file or folder | Ctrl+F, Esc | filters as you type; Enter from another section goes to Explore › File list; Esc clears |
| Aa, .* | Alt+C, Alt+R | match case, regular expression |
| Filters | Ctrl+Shift+F | popover: size, date, category or extension, attributes, owner; each filter becomes a chip |
| Settings | Ctrl+, | opens Settings |
| More «⋯» |  | menu: Export… (Ctrl+Shift+E) · Command palette (Ctrl+K) · Keyboard shortcuts (Ctrl+\) · Settings · Help (F1) · Report a problem · About DiskDome |

**Filters and messages** (only with a filter or a message)

| Control | Keys | Does |
| --- | --- | --- |
| Filter chip |  | click: edit the filter; «×»: remove it |
| Clear filters |  | removes all the tab's filters |
| Info bar action |  | does what it says («Change mode», «Try again», «Scan folder by folder») |
| Info bar × |  | hides the message for this tab |

**Sections** (after the drive's first scan; ordered by use)

| Control | Keys | Does |
| --- | --- | --- |
| Overview | Ctrl+1 | the drive at a glance; the section you land on after a scan |
| Explore | Ctrl+2 | tree, file list and map |
| Clean-up | Ctrl+3 | what you can free, to send to the Recycle Bin |
| Timeline | Ctrl+4 | when the space arrived |
| Statistics | Ctrl+5 | the drive's numbers, each one clickable |
| Activity | Ctrl+6 | what changes now (disabled without live updates) |
| Summary on the right |  | click: Statistics |

**Before scanning** (drive never scanned: the call to action in the middle)

| Control | Keys | Does |
| --- | --- | --- |
| Scan C: (large, 40 px) | Enter | the only primary button on screen; in fast mode Windows asks for permission |
| Mode · Change |  | one line explaining the mode; «Change» opens the three options right there (no first-launch dialog) |
| Other drives: Scan |  | secondary; used and free come at once from Windows |
| Scan a folder… | Ctrl+O | Windows' folder picker → a new tab |
| Drop a folder here |  | scans it in a new tab |

**While scanning** (scan running)

| Control | Keys | Does |
| --- | --- | --- |
| Stop |  | stops the scan; keeps what was read (folder walk) or nothing (MFT) |
| Sections |  | disabled until the MFT is read; with a folder walk Explore fills as it reads |

**Overview** (Overview section)

| Control | Keys | Does |
| --- | --- | --- |
| Map and «Open the map» |  | → Explore › Map |
| Free up space… (primary) |  | → Clean-up |
| Category row (File types) |  | → Explore › Map with the category highlighted |
| Folder row (Where the space goes) |  | → Explore › Tree with the folder selected |
| Open Clean-up |  | → Clean-up |
| Show (Temp, old Downloads) |  | → Explore › File list with the filter set |
| Open the Recycle Bin |  | opens Windows' Recycle Bin |
| Power, Performance, System protection, Disk Clean-up |  | open Windows' tool; DiskDome never touches these files |
| File row (Largest files) |  | → Explore with the file selected |
| All |  | → Explore › File list by size |

**Explore: its bar** (under the sections, Explore only)

| Control | Keys | Does |
| --- | --- | --- |
| Tree · File list | Ctrl+Alt+T / L | the table shows nested folders or every file, largest first |
| Table · Both · Map | Ctrl+Alt+1 / 2 / 3 | layout |
| File types (panel) | Ctrl+Alt+P | shows or hides extensions and categories |
| Selection: Show · Move to the Recycle Bin · ⋯ | Ctrl+E, Del | with a selection: name, size and the main actions, no right-click needed; «⋯» opens the same menu as right-click |
| Rescan the folder | F5 | in the «⋯» menu and the right-click menu |

**Explore: table** (Explore section)

| Control | Keys | Does |
| --- | --- | --- |
| Column header |  | sorts; a second click reverses; drag the edge to resize; right-click: visible columns |
| Row chevron | → / ←, Num * | expands or collapses; Num * expands the whole branch |
| Row | Ctrl, Shift | selects (in the map too); double-click: folder → expands it, file → opens it |
| Row, right-click | Menu key, Shift+F10 | context menu |
| Table / map splitter |  | drag; double-click: back to 42 % |

**Explore: map** (Both and Map layouts)

| Control | Keys | Does |
| --- | --- | --- |
| Hint in the header |  | with nothing hovered: «double-click a rectangle to go into it» |
| Home | Home (map focused) | zoom to the drive root |
| Up | Backspace, Alt+↑ | up one level |
| Breadcrumb |  | zoom to that level |
| Free space |  | shows or hides the free-space block (drives only) |
| Options «⋯» |  | Shape: Rectangles, Rings, Layers · Colours: Type, Age · Names · Style (in Settings) · Save as image… · Copy as image |
| Rectangle |  | hover: outline and details in the header; click: select; double-click or Enter: zoom into the folder |
| Rectangle, right-click |  | context menu |
| Free-space block |  | hover only (not selectable) |

**Explore: side panel** (Explore, when open)

| Control | Keys | Does |
| --- | --- | --- |
| Extensions · Categories |  | switches the list |
| Row |  | highlights in the map; a second click removes it; Ctrl+click: becomes a filter |
| Column header |  | sorts by size, % or file count |

**Context menu** (table, map, lists)

| Control | Keys | Does |
| --- | --- | --- |
| Open · Show in File Explorer · Open terminal here · Properties | Enter, Ctrl+E, —, Alt+Enter | Windows' own actions, with your rights |
| Copy path · Copy name · Copy folder · Copy list | Ctrl+Shift+C | to the clipboard |
| Scan again · Zoom in the map · Show in the tree | F5 | stay in DiskDome |
| Filter: this extension · this folder |  | adds a chip |
| Rename · Move to… · Cut · Paste | F2, —, Ctrl+X, Ctrl+V | inline edit; Move dialog |
| Move to the Recycle Bin | Del | confirmation dialog (§ 6.4) |
| Delete permanently | Shift+Del | confirmation dialog, «Cancel» is the default |

**Timeline** (Timeline section)

| Control | Keys | Does |
| --- | --- | --- |
| 12 months · 24 months · All |  | the chart's range |
| Month bar, «earlier» |  | → Explore › File list with the chip «Modified: March 2026» |
| Calendar day |  | → Explore › File list, filtered to that day |

**Statistics** (Statistics section)

| Control | Keys | Does |
| --- | --- | --- |
| Number row |  | → Explore › File list with that number as a filter (hidden, empty, owner, long path…) |
| Size band |  | → Explore › File list with a size filter |
| Copy the numbers |  | every statistic as text, to the clipboard |

**Activity** (Activity section)

| Control | Keys | Does |
| --- | --- | --- |
| 10 minutes · 1 hour · Since opened |  | range |
| Pause |  | freezes the list (updates keep running underneath) |
| Folder row |  | → Explore with the folder selected |

**Clean-up** (Clean-up section)

| Control | Keys | Does |
| --- | --- | --- |
| Header checkbox | Ctrl+A | selects or clears all |
| Row checkbox, row | Space | selects that folder |
| Move to the Recycle Bin (primary) | Del | confirmation dialog; disabled with nothing selected |

**Dialogs** (over the window)

| Control | Keys | Does |
| --- | --- | --- |
| First launch: three options · Continue | Enter | picks the scan mode; cannot be dismissed without a choice |
| Deletion: Move to Recycle Bin / Delete permanently · Cancel | Enter, Esc | Enter never deletes permanently |
| Deletion: Include system items |  | ticked, a second dialog follows |
| Move: Browse… · Move · Cancel |  | destination folder |
| Export: what · format · options · Save in | Ctrl+Shift+E | CSV, JSON, HTML report, map image; «Export» or «Replace» (the old file to the Recycle Bin) |
| Settings: sections · controls · × | Esc | changes apply at once; no OK |
| Commands (palette) | Ctrl+K | find and run any command or folder by name |

**Status bar** (always)

| Control | Keys | Does |
| --- | --- | --- |
| Scan summary |  | → Statistics |
| Selection |  | → Explore with the selection in view |
| Live |  | → Activity |
| Scan mode |  | → Settings › Scan |
| Units |  | switches the units: like File Explorer, GiB, decimal GB |

**Toasts** (after an action)

| Control | Keys | Does |
| --- | --- | --- |
| Show in the Recycle Bin |  | opens Windows' Recycle Bin |
| Show details |  | list of the failed items with the reason |

### 6.13 States of every control

When each control of § 6.12 is enabled, disabled, hidden or asks first, in ten situations (artboard «Stati»). A
disabled control **stays in place**, so nothing moves, and its tooltip says why («Select an item», «Needs the USN
journal», «E: is not connected»); a hidden one has no meaning in that situation. **File actions are never disabled
because another one is running** (owner, 2026-11-24: «disabling them looks slow; only a loading modal would
justify it, and we have none»): they run in parallel, each listed in the status bar (§ 5.29);
only closing the window asks while one runs.

| Control | Never scanned | MFT scan | Folder walk | Ready, nothing selected | One item | Several items | System item | No-admin mode | File action running (in parallel) | Drive unavailable | Note |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Back | disabled | on | on | on | on | on | on | on | on | on | disabled with no history |
| Scan (middle) · Scan again (toolbar) | on | on | on | on | on | on | on | limited | on | disabled | never scanned: large in the middle; after: secondary in the toolbar; while scanning: Stop |
| Rescan the folder (F5) | — | disabled | disabled | disabled | on | disabled | on | limited | on | disabled | one folder only |
| Search, Filters | — | — | limited | on | on | on | on | on | on | on | during a walk: on what was read; drive gone: on the last scan |
| Table · Both · Map | — | — | on | on | on | on | on | on | on | on | in Explore’s bar |
| Selection: Show · Bin · ⋯ | — | — | limited | — | on | on | confirms twice | on | on | disabled | in Explore’s bar; the Bin asks first |
| Sections | — | disabled | limited | on | on | on | on | on | on | on | during a walk Explore only |
| Activity | — | disabled | disabled | on | on | on | on | disabled | on | disabled | needs the USN journal |
| Open (Enter) | — | — | limited | disabled | on | confirms | on | on | on | disabled | more than 10 files: asks |
| Show in File Explorer | — | — | limited | disabled | on | limited | on | on | on | disabled | several: if in the same folder |
| Open terminal here | — | — | limited | disabled | on | disabled | on | on | on | disabled | one item only |
| Properties | — | — | limited | disabled | on | on | on | on | on | disabled |  |
| Copy path, name, list | — | — | on | disabled | on | on | on | on | on | on | also with the drive gone |
| Rename (F2) | — | — | disabled | disabled | on | disabled | confirms twice | on | on | disabled | one at a time |
| Move to…, Cut, Paste | — | — | disabled | disabled | on | on | confirms twice | on | on | disabled | Paste onto a folder only |
| Move to the Recycle Bin (Del) | — | — | disabled | disabled | confirms | confirms | confirms twice | confirms | confirms | disabled | always confirmed (§ 6.4) |
| Delete permanently | — | — | disabled | disabled | confirms | confirms | confirms twice | confirms | confirms | disabled | «Cancel» is the default |
| Zoom in the map | — | — | limited | disabled | on | disabled | on | on | on | on | a folder |
| Free space | — | — | disabled | on | on | on | on | on | on | disabled | drives only, not folders |
| Clean-up: Move to the Recycle Bin | — | — | disabled | disabled | confirms | confirms | — | confirms | confirms | disabled | needs at least one ticked folder |
| Export | disabled | disabled | disabled | on | on | on | on | on | on | on |  |
| Close tab (folders) | on | limited | limited | on | on | on | on | on | confirms | on | stops a scan; asks during an action |
| Close the window | on | on | on | on | on | on | on | on | confirms | on | asks only during a file action |
| Settings | on | limited | limited | on | on | on | on | on | on | on | a changed mode applies to the next scan |

### 6.14 When something goes wrong

Eighteen cases (artboard «Errori»), each shown by one component: an **info bar** for what blocks or limits the
scan (it stays until solved or closed), a **toast** for an action that failed in part (with «Show details»), an
**inline error** in the field or dialog that caused it, an **empty state** for a tab that cannot show anything. The
engine sends the code; the interface words it (what happened, what to do), never a path or an OS message in the
sentence; the details go to the log (§ 6.9).

| Case | Shown in | Message | Actions | Engine code |
| --- | --- | --- | --- | --- |
| Permission refused (UAC) | info bar, warning | You did not give permission: the drive’s index can’t be read. | Try again · Scan folder by folder | `ElevationDenied` |
| Drive not NTFS | info bar, neutral | E: is exFAT: scanning it folder by folder, it takes longer. | — | `NotNtfs { fs }` |
| Drive locked (BitLocker) | info bar, danger | D: is locked by BitLocker. Unlock it in File Explorer and try again. | Try again | `VolumeLocked` |
| The service does not answer | info bar, danger | DiskDome’s service does not answer. | Restart the service · Ask for permission this time | `ServiceUnavailable` |
| Folders not readable | info bar, warning | 214 folders can’t be read without permissions: totals are lower than real. | Show the list · Change mode | `Unreadable { count }` |
| Drive removed while scanning | info bar, warning | E: was removed: the scan stopped. | Scan again (when it’s back) | `VolumeGone` |
| Drive removed afterwards | info bar, neutral | E: is not connected: showing the 14:02 scan. File actions are off. | — | `VolumeGone` |
| Network path unreachable | empty state | \\nas\foto does not answer. Check the connection and try again. | Try again | `NetworkUnreachable` |
| File in use | toast, danger | 2 items were not moved to the Recycle Bin. | Show details → «In use by Steam» · Try again | `InUse { by }` |
| No permission for the action | toast, danger | You don’t have permission to move 3 items. DiskDome doesn’t ask for administrator rights for file actions. | Show in File Explorer | `AccessDenied` |
| Not enough space to move | the dialog | D: has 12 GB free, 48.5 are needed. | «Move» disabled · Choose another folder | `NoSpace { free, need }` |
| Too large for the Recycle Bin | the dialog | 2 items are too large for the Recycle Bin: Windows would delete them for good. They are left out. | «Move the other 3 to the Recycle Bin» · «Delete these permanently…» (its own confirmation) | `TooLargeForBin` |
| Invalid or taken name | the field | A file with this name already exists. | the field stays open | `NameTaken / NameInvalid` |
| Path too long | toast, warning | File Explorer can’t open paths this long (318 characters). | Open terminal here · Copy path | `PathTooLong` |
| Journal overrun | status bar, warning | Live updates paused: too many changes at once. | Scan again | `JournalWrapped` |
| Memory at the limit | the tab | Scanned 10 minutes ago: re-read when needed to save memory. | Scan again (2 s) | `ModelReleased` |
| Internal scan error | info bar, danger | The scan stopped because of an error. The details are in the log. | Try again · Open the log | `Internal { id }` |
| Saved summary unreadable | no message | Home says «last scan: not available»; the rest goes to the log. | — | `SummaryCorrupt` |

### 6.15 The new-user walk-through

Owner, 2026-11-24: «go round again as a new user, it must be intuitive: if scanning is the first thing to do, it
can't be a small hidden button; reason like this». The path redone as someone who opens DiskDome for the first
time to learn what fills the drive and how to free it, with four questions at every screen: is the thing to do now
the most visible one? Is every control where it applies? Can it be reached without right-click and without
shortcuts? Are the words those of a Windows user? Artboards «Giro» and «PrimaVolta».

| # | Moment | What a new user meets | Change | Status |
| --- | --- | --- | --- | --- |
| 1 | First launch | A dialog asks how to scan (UAC, service, no admin) before the app is seen. | No dialog: the recommended mode is chosen and said in one line under the button; «Change» opens the options in place; Windows asks for permission when Scan is pressed. | done in the mockup |
| 2 | First launch | Scanning is the first thing to do, but the button was small in the toolbar and Home had three equal small buttons. | One large call to action: «Find out what takes up space on C:», a usage bar, «Scan C:» 40 px tall (new `hero` size), «A few seconds. It only reads: nothing is changed.» | done in the mockup |
| 3 | First launch | Search, Filters and the layout control shown disabled. | Before the first scan the toolbar has only Settings and «⋯»; search and filters arrive after. | done in the mockup |
| 4 | First launch | Two primary buttons: «Scan» in the toolbar and «Scan C:» in the middle. | The toolbar button appears only after a scan, as «Scan again», secondary. | done in the mockup |
| 5 | After the scan | Overview had no map, the most intuitive view for newcomers. | The map is Overview's first card (click → Explore › Map), «What you can free» beside it. | done in the mockup |
| 6 | After the scan | No sentence saying what is going on. | One sentence at the top: how full, the biggest things, how much can be freed. | done in the mockup |
| 7 | After the scan | «What you can free» had five different buttons and no main step. | A primary «Free up space…» to Clean-up; Clean-up gathers rebuildable folders, Temp and old Downloads (guided clean-up, to decide). | done in the mockup |
| 8 | After the scan | Clean-up was the last section. | Order by use: Overview, Explore, Clean-up, Time, Statistics, Activity. | done in the mockup |
| 9 | Explore | The layout control and the panel toggle sat in the toolbar, disabled in the other sections. | They live in Explore's bar with «Tree · File list»; the toolbar keeps global things only. | done in the mockup |
| 10 | Explore | Delete, show or open only through right-click. | With a selection, Explore's bar shows name, size, «Show», «Move to the Recycle Bin», «⋯». | done in the mockup |
| 11 | Explore | Double-click to zoom into the map is not discoverable. | The map header says «double-click a rectangle to go into it» when nothing is hovered. | done in the mockup |
| 12 | Words | «regex», «MFT», «UAC», «USN journal» in visible text. | «Search for a file or folder» (syntax in the tooltip); «Fast scan» in the status bar; «the drive's index» for MFT in user text. | done in the mockup |
| 13 | Words | «Esplora risorse» in the menu, «Esplora file» elsewhere. | «Esplora file» everywhere, as Windows 10 and 11. | done in the mockup |
| 14 | Words | «GiB» instead of «GB». | Proposal: like File Explorer (binary values labelled GB) by default; GiB and decimal GB in Settings. | done in the mockup |
| 15 | Words | «Tempo» as a section name says little. | Proposal: «Cronologia». | done in the mockup |
| 16 | Activity | Empty right after opening, looks broken. | An empty state that explains it. | to decide |

### 6.16 The flow from zero

Owner, 2026-11-24: «how do I choose the drive or folder to analyze? You take too much for granted: everything must
have a source, the user's or another; start from a flow as if you knew nothing about the subject». The path is
redone for someone who does not know what DiskDome does, what «C:» is or which drive is full (artboard «DaZero»),
and drawn at real size, clickable in Play, as the canvas's first row: Inizio → Permesso → Analisi → Panoramica →
Main, plus Cartella and Ritorno.

**Every piece of information on screen has a source**, one of five:

| Source | What it gives |
| --- | --- |
| Windows | the list of drives, their names (as in «This PC»: «Windows (C:)») and type, used and free space, the system drive, the known folders (Downloads, Documents, Temp), Windows' own dialogs (folder picker, UAC, Recycle Bin) |
| Scan | size, dates and attributes of every file and folder, from the drive's index or folder by folder |
| Saved | the summaries of past scans, recent folders, settings: all on the PC |
| User | what they choose, drop, type or confirm |
| DiskDome | rules and texts we write: which drive to recommend (the system drive, or the fullest above 90 %), which folders are rebuildable, time estimates, «What is it?» (not for now, owner 2026-11-24⟩ |

**Where the user comes from**: the Start menu (first time: the Start tab); Windows' «low disk space» warning
(Windows does not open DiskDome; the drive is marked «almost full» on Start); File Explorer › right-click a drive or
folder › «Analyze with DiskDome» (v1, owner 2026-11-24: a menu entry registered by the installer, no shell extension;
what DiskDome then does is § 6.20); a folder
dropped on the icon or the window; the next launches (the tab and section of before, § 6.11).

**All the ways to choose what to analyze**

| Way | Where | Source | What happens |
| --- | --- | --- | --- |
| A drive's card | Start | Windows | scans the drive: fast if it is an internal NTFS drive (asks permission), folder by folder for USB, exFAT, network |
| The drive's tab | title bar | Windows | never scanned: its call to action; else its last scan |
| «Choose a folder…» | Start, «+» | User, Windows' folder picker | a new closable tab, scanned folder by folder, no permission |
| Drop a folder | anywhere in the window | User | as «Choose a folder…» |
| «Network path…» | Start, «+» | User | type `\\server\share` or pick it under Network; mapped network drives are listed with the drives |
| Recent | Start, «+» | Saved | reopens the tab with its last scan and updates it |
| Right-click a folder in Explore › «Open in a new tab» | DiskDome | Scan | a tab of that folder, no new read |
| «Analyze with DiskDome» | File Explorer | User | § 6.20: the drive's tab, the folder inside its scanned drive, its own tab, or a new folder tab |
| Command line `diskdome D:\Projects` (v1) | terminal | User | as «Analyze with DiskDome» (§ 6.20) |

**Step by step**: (1) opening: «What do you want to analyze?» and one sentence on what DiskDome does (it only
reads); (2) the drives as in «This PC», each with a usage bar, used and free, type, «never scanned», and one
«recommended» with its reason; (3) or a folder through Windows' picker; (4) the permission, explained before the
click, with a way out («Scan folder by folder») after «No»; (5) the scan: a bar with the percentage, what it is doing, files, folders and space counted growing, the phases, the
time left and «you can keep using the PC»; at the end a notice says what it found; (6) the result: one sentence, the map with its caption, «Free up
space…»; (7) a file: name, folder, type, size (not for now) «What is it?», two lines we write for the most
common large files and folders (`hiberfil.sys`, `pagefile.sys`, `.vhdx`, `node_modules`, Steam libraries,
OneDrive…), offline; (8) freeing space: the Recycle Bin by default, why each Clean-up item is safe, system files
left out; (9) the next time: where they were, the last scan at once and «since yesterday +3.2 GB» with where it
came from.

### 6.17 Scanning more: network paths, a folder again, several drives, live updates

Owner, 2026-11-25 (card «Analisi»; artboard «AnalisiPiu», and working in «Main»).

- **Network path…** (Start and the tabs' «+»): a dialog `medium` «Analyze a network path», a monospace field
  `\\server\folder` (focused), «RECENT» with the last paths (path, «scanned on 12/11/2026 · 1.40 TB» or «never
  scanned»; a click fills the field), «Browse the network…» (Windows' folder picker on Network) and one caption: it is
  read folder by folder with the user's rights, slower on a slow network, no live updates, and mapped drives (Z:) are
  already among the drives. «Analyze» (`primary`, disabled while empty) and «Cancel». Errors stay in the dialog, under
  the field in `danger`: a drive letter («Z: is a drive: open it from the tabs or the "+" menu»), a path without
  `\\`, a server that does not answer («\\nas\foto does not answer. Check the connection and that the path is
  right, then try again»). Credentials are asked by Windows in its own dialog; meanwhile a neutral InfoBar says
  «Windows is asking for a user name and password for this server. DiskDome does not see them or save them.» and
  «Analyze» waits. The path opens as a closable tab with the `network` icon, scanned at once; no rescan at launch.
- **Rescan this folder** (F5, or the menu; on a file, its folder; Ctrl+F5 the drive): only that folder is read
  again while everything stays usable. Its row shows the 16 px progress ring instead of the icon and «rescanning
  35%» after the name in `ink-faint`; its numbers stay, in `ink-faint`, until the new ones arrive. In the map its
  box is dimmed (`bg` at 40 %) with a 2 px `accent` line along its top edge that grows. At the end every total
  changes at once and the notice says the difference: «"Downloads" rescanned at 15:27: +1.2 GB since last time»
  or «no difference since last time».
- **Another drive while you look at this one**: a scan runs in its tab while another tab is shown. The scanning
  tab shows a 14 px progress ring instead of its icon (tooltip with the percentage); the status bar lists
  «Analysis of E:» among the running operations (§ 5.29), with «Folder by folder · 42% · you can keep looking at C:»
  and «Stop» in its popover. At the end a notice
  «Analysis of E: complete · 4,552 files, 32.2 GB · you'll find it in its tab» and, when DiskDome is not in front, a
  Windows notification (Settings › General). Two drive's-index scans of the same drive never run together.
- **Stopped**: a folder walk keeps what it read: a `warning` info bar under the toolbar «Analysis of E: stopped at
  42%: you see what was read, totals are lower than real.» with «Resume» (reads again from the start, keeping the
  partial view until the new one is ready); the tab's tooltip adds «scan stopped». A drive's-index scan is all or
  nothing: the previous scan stays («stopped: the scan of today at 21:14 stays»), or the call to action comes back.
- **Live updates** (§ 2.2): the status bar's dot and label say the state; a click opens a 320 px popover with the
  state, why, and one action:

| Label | When | Action |
| --- | --- | --- |
| Live (`success` dot) | NTFS, drive's index, on | Pause |
| Live updates paused | paused by the user; resuming brings the changes of those minutes from the journal | Resume |
| Live updates paused | the journal overflowed (`JournalWrapped`, § 6.14) | Scan again |
| Live updates off | off in Settings › Scan | Turn on |
| No live updates | not NTFS, network path, folder walk | Scan again · Change mode |
| No live updates | a folder scanned apart (its drive may be live) | Rescan (F5) |

  Activity without live updates is an `EmptyState` with the same reason and action.

### 6.18 Export

What a scan found, taken out of DiskDome as a file (owner, 2026-11-25; artboard «Esportazione»). Exporting only
reads the model and writes the one file chosen, with the user's rights (rule 4); it never touches the files analysed.

**From where.** The app's ⋯ menu › «Export…» (Ctrl+Shift+E; Ctrl+E stays «Show in File Explorer») starts on the
whole drive; the context menu › «Export the selection…» (in the Copy group) starts on the selection; the map's options
menu › «Save as image…» starts on «Map image» and the folder open in the map (§ 4.4). The command palette lists
«Export…» and «Save the map as an image…». The dialog remembers the last format and its options; the starting point
decides «What» every time.

**The dialog** (720 px, fits 960 × 600 without scrolling; artboard «Esportazione»):

- **What**, a radio list with the counts on the right: «All of C:» (or «All of the folder …» for a folder tab),
  «The folder open in the map» (only when the map is zoomed), «The selection» (disabled with its reason when nothing
  is selected, or, for an image, when it is not one folder). Selected items hidden by filters or the search are left
  out until «Also include …» is ticked, as when deleting (§ 6.1, § 6.4). With filters or a search active, a ticked
  «Only what passes the filters: Video» makes rows and totals follow what is on screen; for an image the filters stay
  as on screen (dimmed).
- **Format**, a list of four with what each is for: **CSV** «for Excel and spreadsheets», **JSON** «for scripts and
  other programs», **HTML report** «to read, print or send», **Map image** «PNG».
- **Options** of the chosen format, on the right:
  - CSV and JSON: rows (folders and files · only folders · only files) and depth (all, or up to N levels from the
    starting point, 1–20, with a stepper).
  - CSV: columns (Path plus the table's visible columns in their order, or all: Path, Name, Type, Extension,
    Category, Size, Allocated, % of folder, Files, Folders, Modified, Created, Accessed, Attributes, Owner) and sizes
    (in bytes, to calculate; or readable, as in the table). The separator is not chosen here: it is a setting
    (Settings › General › «CSV files», § 6.10), by default Windows' list separator («;» in Italian), and the dialog
    says which one it will use («Separator: «;», like Windows · changed in Settings › General»).
  - HTML report: what it contains, ticked by default: the map, the 50 largest folders, the 50 largest files, file
    types, what can be freed, what changed (only with a saved scan, naming its date); and «Hide your user name in
    paths» (`C:\Users\…\`), unticked.
  - Map image: size (as on screen · 1920 × 1080 · 3840 × 2160, with the screen's size written under), «With folder
    names», «With the colour legend and the total».
  - Under the options, one line saying what the file is like.
- **Save in**: the whole path in one field and «Browse…» (Windows' Save As). The first time Documents, then the last
  folder used; the name «DiskDome C 2026-11-25.csv» (with the folder's name, or «selection», when it is not the whole
  drive), «(2)» added when it exists.
- Left of the buttons, the estimate: «4,075,050 rows · about 544 MB» (pixels for an image, «with the filters» when
  they apply), updated at every choice. «Export» is the default button.

**Cases.** Over 1,048,576 rows in a CSV: a `warning` InfoBar with the real number and what to do («Only folders»,
fewer levels, another program); it does not block. A name that exists: an InfoBar and the button becomes «Replace»;
the old file goes to the Recycle Bin (rule 3) and the focus moves to «Cancel». A destination without permission, a
full disk or a drive removed is found while writing (below).

**While and after.** The dialog closes at once and the export is an operation in the status bar (§ 5.29), «Exporting
C: to CSV», with «Stop»; work goes on and several can run. It reads a snapshot of the model taken when it starts, so
changes from the journal do not mix into the file. It writes to `name.ext.part` beside the target and renames it at
the end: stopping or failing leaves no half file and never touches a file with that name. At the end a notice «Exported
"DiskDome C 2026-11-25.csv" · 544 MB · 4,075,050 rows» with «Show in File Explorer» and «Open» (Windows' default
program). On failure a `danger` notice that stays until closed says why (space, permission, drive gone) and offers
«Choose where to save…» and «Retry»; the full path and the OS error go to the log (§ 6.14).

**The files.**

- **CSV**: UTF-8 with BOM (Excel reads the accents); a header row with the column names in the app's language; text
  in double quotes (quotes doubled); dates «2026-11-25 14:00» in local time, which Excel recognises; percentages with
  Windows' decimal separator; folders end with `\`. One row per item, parents before children.
- **JSON**: UTF-8; sizes always in bytes, dates ISO 8601 in UTC, keys in English (it is for programs): `diskdome`
  (version), `drive` (letter, label, file system, capacity, free), `scanned`, `scope`, `filters`, `depth`, then
  `root`, nested like the tree, each item with `name`, `size`, `allocated`, `files`, `folders`, `modified`,
  `created`, `category`, `attributes`, `owner` and `children`. The command line writes the same format.
- **PNG**: the map redrawn at that size by `diskdome-treemap` (not scaled up), in the current style, colours and
  theme, with names and legend when ticked; no selection, no hover.
- **HTML report**: one `.html` file of about 2–3 MB with its style, the map (PNG inside) and tables; no script, no
  font, image or link fetched from the internet, no form: it opens offline in any browser and can be attached to a
  mail. A header (drive, scan date, usage bar and counts), then the parts ticked; each table shows 10 rows and «Show
  all 50» opens the rest (a `details` element). It follows the reader's light or dark theme
  (`prefers-color-scheme`) with DiskDome's colours, prints light on white, A4 portrait, a part never split across two
  pages. The footer says it was made by DiskDome on this PC, that nothing was sent, and what a GB is. No PC name.
- Sizes follow the units setting (§ 4.1) where they are readable; in bytes they depend on nothing.

**Copy as image** (map options menu) puts the map at the size on screen on the clipboard and says so in a notice
(«Map copied to the clipboard as an image · 1240 × 410 px»).

### 6.19 The app menu, the command palette, shortcuts, About

The app's own commands (owner, 2026-11-25; artboard «PaletteComandi»).

**The app menu.** The `ellipsis` button at the end of the toolbar, after Settings, opens a `Menu` that never changes:
«Export…» (Ctrl+Shift+E), «Command palette» (Ctrl+K), «Keyboard shortcuts» (Ctrl+\) · «Settings» (Ctrl+,) · «Help»
(F1) and «Report a problem», each with a small `external-link` because they open the browser · «About DiskDome».
Help opens the user guide in the repository (the README until a guide exists). Report a problem opens a new GitHub
issue with the version and Windows' version already written; DiskDome itself sends nothing. The menu holds only the
app's commands, never a file's: those are in the context menu and in Explore's bar.

**The command palette** (Ctrl+K, or the app menu): a popup (§ 3.11: one at a time) at the top centre, 40 px from the
window's top, over the toolbar, 640 px wide (8 px from the edges in a narrow window), at most the window's height
less 80 px. A field «Type a command, a folder or a filter…» with `search` and an `Esc` key cap; the results; a
footer with the keys (↑ ↓ to move, Enter to run, Esc to close, Ctrl+\ all the shortcuts).

- **Empty**: «Recent» (the last 5 commands run, kept between sessions), «Sections» with Ctrl+1…6, «DiskDome» (the app
  menu's commands).
- **Typing**: «Commands» (up to 8), «Settings» (up to 3: each section of Settings and the main choices, «Theme: Dark»),
  and from two letters «Folders and files» (the 5 largest whose name contains the words, with path and size), «Saved
  filters», «Drives and open folders». Words match at the start of a word of the name or of its keywords, which
  include the English names in the Italian interface («rename», «export»); case and accents do not matter.
- **Rows**: 30 px, `Menu` items with an icon, the name, a hint in `ink-faint` (a path and size, a filter) and the
  shortcut as a `Kbd`. A command that cannot run now stays, dimmed, with its reason where the shortcut goes («select
  something first», «one item at a time», «no filter on»), as disabled controls do (§ 6.13); it sinks below the
  ones that can run.
- **Keys**: ↑ ↓ move the highlight (the pointer too), Enter runs the highlighted row and closes, Esc closes. A folder
  or a file opens Explore with it selected; a saved filter applies; a drive opens its tab; a setting opens Settings on
  its section.
- **Nothing found**: «Nothing for "xyz". Try another word, or search the files with Ctrl+F».
- The commands, their names, shortcuts and reasons are the list of § 6.12 (artboard «Comandi»): one source for the
  menus, the palette and the shortcuts dialog.

**Keyboard shortcuts** (Ctrl+\, or the app menu; owner, 2026-11-25: not Ctrl+/, which needs Shift on the Italian keyboard): a 780 px dialog, «Keyboard shortcuts», a search field at the top
right («Search an action or a key»), the shortcuts of § 6.2 by area in two columns — General, Drives and sections,
Search and filter, Explore, Selection, Files — each row the action and its keys as `Kbd`, written as Windows writes
them in the interface's language (Maiusc, Canc, Invio, PagSu in Italian). Searching filters by action or key
(«ctrl+e»); with nothing found it says so. The footer: «Windows' shortcuts work here too. In menus and in the palette
every command shows its own.» and «Close». A shortcut added to § 6.2 appears here, in the menus and in the palette.

**About DiskDome** (the app menu): a 560 px dialog with three tabs of the same height:

- **DiskDome**: the mark at 48 px, the name, «Version 0.1.0 · 25 November 2026 · Windows 11 24H2, x64» with «Copy»
  (for a report), one sentence on what it does, the copyright («© 2026 Stefano Domenghini», the owner confirms the
  holder), the GPL notice («DiskDome is free software: you can redistribute it and/or modify it under the terms of
  the GNU General Public License, version 3 or, at your option, any later version. It comes with no warranty.»),
  «Source code» and «Report a problem» (browser), and «DiskDome does not connect to the internet and sends no data to
  anyone. The buttons with the arrow open your browser.»
- **Licence**: the full text of the GNU GPL v3 shipped with the program (`LICENSE`), scrollable; the English text is
  the one that applies. The GPL asks that an interactive program show it to the user.
- **Third-party components**: component, version, licence and what it is for — Slint (used under GPL-3.0-only),
  windows-sys and rayon (MIT or Apache-2.0), Lucide (ISC) — then every crate they use, each expandable to its licence
  text, generated at build time from `Cargo.lock` (cargo-about), plus the assets of `THIRD_PARTY.md`. The font is
  Windows' Segoe UI, not shipped.

### 6.20 «Analyze with DiskDome» from File Explorer and the command line

Owner, 2026-11-25 (artboard «AnalizzaCon»; in «Main», Tweaks › «Prova: Analizza con DiskDome»).

**In File Explorer.** One entry, «Analyze with DiskDome», with the mark: on a folder, on a drive (in «This PC») and on
the background of an open folder (it analyses that folder); never on a file. The installer registers it for the user
(HKCU) as a static verb: no shell extension, no administrator rights (rule 4); Settings › File Explorer removes and
restores it (§ 6.10). On Windows 11 it is under «Show more options» (Shift+F10): the first menu takes only apps with a
signed package identity, after v1 with code signing. Several folders selected: one tab each, up to 8; a notice says how
many were left out.

**From the command line.** `diskdome [path …]` does what the entry does, path by path, and returns to the prompt at
once. Relative paths against the terminal's folder (`.`), quotes around spaces; a letter alone (`C:`) is the drive;
`\\server\share` is a network path; a file opens its folder with the file selected. The installer registers
`diskdome.exe` under App Paths (Win+R too) and adds its folder to the user's PATH. Scripts that want data without a
window use `diskdome-cli`, whose JSON is the export's (§ 6.18).

**What DiskDome does**

| What you open | What DiskDome does |
| --- | --- |
| A drive | its tab; never scanned: the scan starts at once (USB, exFAT folder by folder; an internal drive with its index: Windows' permission prompt first); scanned: as it was, kept current by live updates |
| A folder on a drive already scanned | the drive's tab, Explore, the folder selected and the map zoomed on it; nothing is read again, and a notice says so: «"Downloads" is in C:, scanned today at 21:14: shown here, not read again» with «Read it again (F5)» |
| A folder that already has its tab | that tab, no second one; the notice says when it was read, with «Read it again (F5)» |
| A folder on a drive not scanned | a new tab for the folder, read folder by folder with the user's rights: faster than the whole drive |
| A network path | the «Network path» dialog with the path written in; Enter starts (§ 6.17) |

- **DiskDome closed**: it opens straight on that tab, not on Start; the tabs of the last session reopen behind it.
- **DiskDome open**: one window. A second launch hands its paths to the first and exits; the window comes to the front
  (restored if minimised); the tab shown before keeps its state (Alt+← goes back).
- **A scan running**: the new one runs alongside in its tab, both listed in the status bar (§ 5.29).
- **A dialog open** (Export, Settings): it stays; the new tab opens behind it and its notice waits (§ 3.11); a network
  path waits and its dialog opens when the other closes.

**Errors** are `danger` notices that stay until closed (§ 5.20), with what to do; with DiskDome closed it opens anyway,
on Start or the last tab, so the notice is seen:

- a drive that is not there: «F: is not connected. Reconnect it and try again.» with «Retry»;
- a path that does not exist: «Can't find "C:\Progetti": it may have been moved or renamed.» with «Choose a folder…»;
- a path that is not valid (command line): «"…" is not a valid path: it needs a drive letter or \\server\share.» with
  «Choose a folder…»;
- a network path that does not answer: the error inside its dialog, with «Retry» (§ 6.17);
- a folder the user's rights cannot read: the tab opens and says why (`folder-lock`, «You can't read "System Volume
  Information" with your rights»), with «Analyze all of C: (asks permission)» — the drive's index reads it, then the
  folder is shown there — and «Close the tab».

The full path and Windows' message go to the log, never into the sentence (§ 6.14).

## 7. Accessibility and languages

- Contrast (WCAG 2.1, checked by a test on the token values, § 10): text ≥ 4.5:1 on its ground (`ink`,
  `ink-muted`, `accent-text`, the state colours on `bg`, `surface`, `raised`; on a selected row, `accent-soft` over
  `surface`, `ink` is 9.7:1 / 11.6:1 and `ink-muted` 5.3:1 / 5.8:1, dark / light); `ink-faint` only for disabled
  text, placeholders and the dash of missing values (≥ 4.0:1 nonetheless); focus ring and checkbox / radio
  boxes ≥ 3:1; `on-accent` on `accent` ≥ 4.5:1; `on-data` on every category ≥ 4.5:1.
- Never colour alone: categories also have a name (side panel, treemap header), states an icon and a word, the sorted
  column an arrow.
- Every custom component sets Slint's `accessible-role`, `accessible-label` (icon-only buttons: their `label`),
  `accessible-checked`, `accessible-value` (progress, usage), so Narrator reads them.
- Everything by keyboard (§ 6.2), focus always visible while navigating by keyboard. **Tab order** (Tab moves
  between zones, arrows inside a zone, F6 jumps between the big zones):

  1. **Drive tabs**: one stop; ← → between tabs, Enter opens, Ctrl+W closes a folder tab (not Del: too close to deleting)
  2. **Toolbar**: Back, Scan, F5, Search (Aa and .* inside with Tab), Filters, layout (one stop, ← →), Panel, Settings, More
  3. **Filters and info bar**: each chip, «Clear filters», the message’s action, ×
  4. **Section bar**: one stop; ← → between sections; then the summary
  5. **Explore: header**: Tree · File list (one stop)
  6. **Table**: one stop; arrows, PgUp/PgDn, Home, End inside; Space selects; Menu key opens the menu
  7. **Side panel**: Extensions · Categories, then the list (one stop, arrows)
  8. **Map header**: Home, Up, each crumb, Free space, ⋯
  9. **Map**: one stop; arrows go to the neighbouring rectangle that way, Enter zooms, Backspace goes up. The map is an image: one hidden accessible element carries the active rectangle’s text
  10. **Status bar**: each item is a button: summary, selection, Live, mode, units

- **What Narrator reads** (name, type, value, state, in the interface's language). Announcements do not move the
  focus: polite for results, assertive only for errors ⟨to verify: Slint 1.19's support for live announcements, § 12
  test 15⟩.

  - Drive tab: «C: Windows, NTFS, 670 GB used of 954. Tab 1 of 4, selected.»
  - Tree row: «Users, folder, 310 GB, 46.4 percent, expanded, level 2.»
  - Map rectangle: «docker_data.vhdx, Archives, 48.5 GB, 7.2 percent of C:.»
  - Progress bar: «Scanning C:, 62 percent, reading the index.»
  - Chart: «Calendar, 365 days; Enter opens the table of values.»
  - Announcement, polite: «C: scanned: 3,789,066 files.» · «3 items moved to the Recycle Bin, 4.2 GB freed.»
  - Announcement, assertive: errors only: «The scan stopped because of an error.»

- Text scale: Windows' «Text size» setting is not applied by Slint; DiskDome reads it and multiplies the type
  steps ⟨to verify⟩, layouts grow with it (no fixed heights for wrapping text).
- Languages: English source, Italian translation (`@tr()`); the language follows the OS (Italian or English) unless Settings › General forces one (§ 6.10).
  Plurals through `@tr("{n} item" | "{n} items" % n)`.

## 8. Implementation in Slint

### 8.0 Library-grade components

Every component of the design system is as solid as a shadcn / Radix one, not the minimum one screen needs. It has
its full behaviour (pointer, keyboard, focus return, every state); it handles every placement, alignment, collision
and overflow case (window edges, long and translated text, empty, many items, scrolling); it is accessible; it is
customised through properties whose defaults are tokens. It holds nothing specific to DiskDome, so the same
component serves future Slint projects. Before a component is built, its behaviour is listed against its Radix /
shadcn counterpart on its card; the Kit shows every case and the headless tests prove it. A Slint built-in element
that cannot follow the design is replaced by ours, without waiting for the owner.

### 8.1 Folders

The design system is its own crate, **Dome UI** (a library of components for every Dome app, as shadcn is for React;
card https://trello.com/c/9yiz5cgx). Its Slint half is a Slint library: `slint-project.json` names `index.slint` as
its main file, so an app imports `@dome-ui` and the compiler refuses any other file of it. The app maps the name in
its `build.rs` (`with_library_paths`). It moves to its own repository with the second Dome app that uses it; the
name is provisional.

```
crates/dome-ui/                  the design system (package `dome-ui`)
  README.md                      the map (this document's short version)
  src/                           the Rust half: pure functions (menu.rs, segmented.rs) and the macros that wire them (menu_nav!, segmented_nav!)
  ui/
    slint-project.json           the library: main file index.slint
    index.slint                  the only file apps import, as `@dome-ui`
    tokens/
      theme.slint  data.slint  size.slint  type.slint  motion.slint
    enums.slint                  ButtonVariant, ButtonSize, InkTone, MenuEntryKind…
    icons/                       Lucide SVGs + icons.slint (global Icons)
    primitives/                  behaviour shared by components, never used by apps
    components/                  one file per component (§ 5)
crates/ui/ui/                    the app
  app.slint                      MainWindow: composes the screens
  kit/kit.slint                  KitWindow (§ 9), with the components' test windows
  screens/
    main/  empty/  dialogs/  settings/
```

### 8.2 Shape of the tokens

```slint
// tokens/theme.slint (excerpt). Values: the tables of § 3.2.
export enum ThemeMode { dark, light }

export global Theme {
    in-out property <ThemeMode> mode: ThemeMode.dark;
    out property <bool> dark: mode == ThemeMode.dark;

    out property <color> bg:                dark ? #0f1113 : #dee0e2;
    out property <color> surface:           dark ? #161719 : #eaebed;
    out property <color> raised:            dark ? #1f2123 : #f2f3f5;
    out property <color> field:             dark ? #1c1e20 : #f1f2f4;
    out property <color> control:           dark ? #232528 : #d7d9dc;
    out property <color> control-hover:     dark ? #2c2e31 : #cdd0d3;
    out property <color> line:              dark ? #282a2d : #cbced1;
    out property <color> line-strong:       dark ? #6b6e73 : #717579;
    out property <color> edge:              dark ? #35383c : #bbbec2;
    out property <color> ink:               dark ? #e8eaec : #191c20;
    out property <color> ink-muted:         dark ? #abaeb3 : #494d54;
    out property <color> ink-faint:         dark ? #7e8288 : #65696f;
    out property <color> hover:             dark ? #222325 : #e0e1e3;
    out property <color> selected-inactive: dark ? #2b2c2e : #d7d8db;
    out property <color> danger:            dark ? #f66d67 : #b32322;
    out property <color> warning:           dark ? #f6b84d : #8a5100;
    out property <color> success:           dark ? #5fcf83 : #10703a;

    // Accent: Laguna
    out property <color> accent:            dark ? #37c2bb : #007c79;
    out property <color> accent-hover:      dark ? #4dd2cb : #006f6d;
    out property <color> accent-pressed:    dark ? #12aea8 : #006360;
    out property <color> on-accent:         dark ? #0f1113 : #ffffff;
    out property <color> accent-text:       dark ? #37c2bb : #006663;
    out property <color> accent-soft:       dark ? #37c2bb38 : #007c7929;
    out property <color> focus:             dark ? #37c2bb : #006663;
}

// tokens/size.slint (excerpt)
export global Size {
    in-out property <bool> compact: false;
    out property <length> control-sm:  compact ? 20px : 24px;
    out property <length> control:     compact ? 24px : 28px;
    out property <length> control-lg:  compact ? 28px : 32px;
    out property <length> row:         compact ? 18px : 22px;
    out property <length> header-row:  compact ? 20px : 24px;
    out property <length> toolbar:     compact ? 32px : 36px;
    out property <length> space-8:     8px;
    out property <length> radius-button: 4px;
}
```

A component reads them by name: `height: Size.row; background: Theme.surface;`. `Label` maps a `TypeStep` to
size, weight and line through pure functions in `global Type`.

### 8.3 What Slint 1.19 does and does not do (checked 2026-11-24)

| Need | Slint | Consequence |
| --- | --- | --- |
| shadows, per-corner radii, gradients | `drop-shadow-*`, `border-*-radius`, `@linear-gradient` on `Rectangle` | elevation as specified |
| backdrop blur (glass) | no | no glass (§ 3.8) |
| OpenType features (tabular figures) | no (`Text` has family, size, weight, letter-spacing, overflow, wrap) | a font with tabular default figures (Segoe UI) |
| middle elision | no (end only) | paths elided in Rust |
| keyboard shortcuts | `KeyBinding` + `@keys()` (1.16) | § 6.2 |
| tooltips | `Tooltip` element | ours (decision 11, § 5.17) |
| context menu, menu bar | `ContextMenuArea`, `Menu`, `MenuItem` | ours (decision 11, § 5.16) |
| huge lists | `Model` trait, only visible rows | the virtual table |
| pixel buffers | `SharedPixelBuffer` → `Image` | the treemap |
| drag and drop with Explorer | experimental | after the basics |
| live preview | VS Code extension; `SLINT_LIVE_PREVIEW=1` in the app | design work in the Kit window |

Standard widgets (`std-widgets.slint`) are used only inside design-system components and only where their look
can be fully set by our tokens (or not at all: for behaviour such as `TextInput` editing).

### 8.4 Names

A name says what a thing holds or does, and the same idea has the same name in every component.

- **Components** are nouns in PascalCase, with every word capitalised: `ScrollBar`, `StatusBar`, `ToolBar`,
  `ProgressBar` (`Checkbox` stays one word, as in shadcn and WinUI). The base window is `DomeWindow`.
- **A component's own types carry its name**: its enums (`ButtonVariant`, `ButtonWidth`, `SegmentedSize`,
  `SegmentedWidth`), its row struct (`MenuEntry`, `SegmentedItem`), the global Rust answers for it (`MenuNav`,
  `SegmentedNav`), and, in Rust, its module and wiring macro (`menu`, `menu_nav!`; `segmented`, `segmented_nav!`).
- **Item or entry.** An *item* is something the user chooses; an *entry* is any row of a list that also holds
  separators and section titles (`MenuEntry` with its `kind`). The property is named after its rows: `items`,
  `entries`.
- **Indices end in `-index`** and are -1 for none: `current-index`, `highlighted-index`, `submenu-index`.
- **Callbacks are events, in the past tense or as a request**: `clicked`, `toggled`, `selected`, `activated`,
  `closed`, `requested`, `search-requested`, `unhandled-key`. Never `changed`, which reads like Slint's
  `changed <property> =>` handlers.
- **One idea, one name**, in every component that has it: `enabled`, `size`, `variant`, `width-mode`, `tooltip`,
  `tooltip-shortcut`, `wrap-around`, `kit-state`. Anything that floats over the window is a *popup* (`PopupSide`,
  `PopupAlign`, `PopupAnchor`, `PopupSurface`), never a popover or flyout.
- **Globals say their state**: `InputMode.keyboard` (the user moves with the keyboard, so focus rings show).
- **Dome UI is generic.** A component that knows a drive, a scan or a DiskDome view lives in the app, built from
  Dome UI's components, and is named in the app's words (`DrivePicker`, `DriveTabs`, `ViewSwitch`).

## 9. The Kit window

`cargo run -p diskdome-ui --features kit -- --kit` (debug builds only; the feature is never in a release) opens
`KitWindow`: every component in every variant, size and state, side by side.

- A top bar: theme (Dark / Light), density (Normal / Compact) and the review board's count. Language (English /
  Italian), map style and palette join it when something in the Kit changes with them: translated component texts,
  the treemap, the Soft and Vivid palettes.
- The tokens first, in the order of § 3, each drawn with itself (swatches, bars as long as the length, tiles with
  the radius, border, shadow or opacity, durations played on a track), so a theme or density switch shows at once.
- One section per component; each shows its states with the real component, never a picture. Main screens'
  parts (a table with 30 fake rows, a treemap from a fixed fake tree, the side panel, the status bar) are shown too,
  as their components are built.
- **Review board**, as in EmuDome: under a section, a `KitIssue` holds a problem with «Now» and «Proposal»
  rendered with the real components and a status (open / proposed / decided); the top bar counts the open ones.
  A decided issue is deleted once the component carries it.
- The Kit is where every new component or variant appears first, and where the open points of this document are
  decided (font, renderer, menus).

## 10. Automatic checks

A Rust integration test in `crates/ui` (`tests/design_conventions.rs`), run by `cargo test` in the CI on Windows
and macOS and by `scripts/check.ps1`. It reads the `.slint` sources as text, every rule self-tested on its
fixture in `tests/fixtures/conventions/` (a `bad_*.slint` must fail, `good.slint` must pass), each rule's blind
spots written beside it:

| Rule | Forbids | Where it applies |
| --- | --- | --- |
| `literal-colour` | `#rgb`, `#rrggbb(aa)`, `rgb(`, `rgba(`, `hsl(`, `Colors.` | everywhere outside Dome UI's `tokens/` |
| `literal-length` | a number with `px`, `phx`, `rem`, `pt`, `cm`, `mm`, `in` (`0` and `%` allowed) | everywhere outside Dome UI's `tokens/` |
| `literal-duration` | a number with `ms` or `s` | outside Dome UI's `tokens/` |
| `font-literal` | `font-size`, `font-weight`, `font-family`, `letter-spacing` | outside `components/label.slint`, `components/selectable-label.slint`, `primitives/text-entry.slint` (the editable text) and the tokens |
| `std-widgets` | `import … from "std-widgets.slint"` | outside Dome UI's `components/` |
| `boundary` | an import of a Dome UI file other than the library itself (`@dome-ui/…`, or a path into `dome-ui`); the compiler refuses it too | in `screens/` and `app.slint` |
| `raw-element` | `Text`, `TextInput`, `TouchArea`, `FocusScope`, `Image`, `PopupWindow`, `Flickable` | in `screens/` and `app.slint` |
| `untranslated` | `text: "…"` with a non-empty literal not in `@tr()` | outside `kit/` |
| `kit-coverage` | an exported component of Dome UI's `components/` missing from `kit.slint` | — |
| `icon-label` | an icon-only `IconButton` or `Button`, or a `ToggleButton`, without `accessible-label` (Narrator would read nothing) | everywhere |
| `binary-units` | a literal «GiB», «MiB», «TiB» in a string: sizes go through the unit formatter (§ 4.1) | outside `design-system/tokens/` and the units setting |

And tests on the values:

- contrast of every pair listed in § 7, in dark and light, plus the age labels and the chart marks
  (`tests/design_contrast.rs`, reading the compiled `Theme` and `Data` globals; the Soft and Vivid palettes join it
  with their transform);
- the minimum ΔE OK between two categories (≥ 0.09) and `on-data` ≥ 4.5:1 on each (the same test);
- density: every compact size ≤ its normal one, `row` ≥ 18, `control` ≥ 20.

**Overlap check** (§ 3.11): a screen test with Slint's testing backend (`ElementHandle`: position and size by
accessible label) opens each state where two layers can meet, at 960 × 600 and 1280 × 800, in both densities: every
popup open with operations running and a notice; Clean-up with a notice; a dialog and Settings with operations and a
notice; the context menu on the last visible row; the status bar's popovers; Statistics, Timeline and Clean-up at
960; the export dialog in its four formats at 960 × 600; the command palette empty and with results, the shortcuts
dialog and About in its three tabs at 960 × 600; a path arriving from File Explorer in each case of § 6.20, also with a
dialog open. It fails when two floating elements intersect, a popup or the notice is closer than 8 px to the window's edge
or touches a bar, a floating element is clipped by its container, or two texts of the content overlap. Blind
spot: only the listed states; a new popup or bar is added to the list in the same commit. (The prototype runs the
same check on the canvas.)

What only review holds: one primary button per view; the accent never in the data area; the tone of the text;
that a component does what its name says.

## 11. Decisions

**Decided by the owner on 2026-11-24** (`docs/decisions.md`): the accent Laguna at L 0.74 (§ 3.2); two themes, Dark
and Light; the data colours by category with three palettes, Standard, Soft and Vivid; the three layouts of
Explore, starting on Both; the map names on «Many»; gaps between folders in every map style; the free space as a
checkbox; drive tabs in the title bar (if test 1 passes); the «Start» tab and no first-launch dialog; the six
sections ordered by use, with «Timeline» for the time section; Overview after scanning a drive, Explore after a
folder; every new view and the chosen feature groups in v1, with the File Explorer menu entry and the command line;
units like File Explorer by default; the tree's default columns with Allocated; compact density 18 / 12 px;
nothing preselected in Clean-up; saved summaries (the last 30 and one per month) and a rescan at launch for NTFS
drives; file actions in parallel with their own progress, never disabled; Activity's aggregation rule; drive names
as in «This PC»; no «What is it?» for now; the table on `ListView` with our own virtual scrolling; global terms
(«Start», «Home», «Timeline») kept in English in the Italian interface; Vignette as the default map style, with
Folders offered as a setting at every window size; «Open» asks for confirmation above 10 files.

**Final pass (2026-11-25, card «Parti da qui»)**: every v1 feature of `docs/features.md` is designed, on the canvas and here; the
prototype's checks (§ 10: overlaps, clipping, popups near the edges, text collisions, dialogs inside the window, icon
buttons without a name, binary units where the setting says otherwise) pass on 200 screens at 960 × 600, 1280 × 800
and 1920 × 1032, dark and light, normal and compact. What is left is decided in the Kit (8, 9, 11) and the logos'
second phase.

**Decided by the owner on 2026-11-26**: Segoe UI at 400 and 700, every strong step in Bold so bold figures stay
tabular (decision 8, § 3.3); menus and tooltips are ours on `PopupWindow`, since Slint's cannot take the tokens
(decision 11, test 8, § 5.16, § 5.17); a built-in element that cannot follow the design is replaced by ours without
waiting; the design system's components are library-grade (§ 8.0).

**Decided by the owner on 2026-11-25**: the software renderer (decision 9, § 3.3); the «Pie» app mark; «Few» map names at 72 × 30; where every chart lives and
when it appears (decision 32: § 4.4 options menu, § 4.7, § 5.27 Start from the second time, § 6.8 «To review»);
multiple selection with Ctrl+click only in the map and nested items counted once (§ 6.1); file actions
(§ 6.5: rename, Move to…, conflicts with «Keep both» first and «Replace» to the Recycle Bin, copy and paste, drag and
drop by Windows' rules, Open above 10, partial results, Undo); filters (§ 6.6: the panel applying at once, the
filter bar, written filters with suggestions, saved filters, filtered totals with the map dimmed; selected items
that a filter hides stay selected and shown apart, § 6.1); the guided clean-up (§ 6.8: rebuildable folders,
temporary files, downloads untouched for a year, identical copies compared on request with the last copy never
removable, Windows' tools, nothing preselected, files in use skipped and said); Settings (§ 6.10: a dialog with six
sections applying at once, the language overridable, the scan service installed and removed from there, saved
summaries deletable per drive); scanning more (§ 6.17: network paths with errors in the dialog and Windows asking for
credentials, F5 on a folder with progress in its row and box, a drive scanning in the background with its card,
stopped walks kept with «Resume», live updates' states and popover); comparing with a saved scan (the «compared with…»
menu, History's points, moved folders shown once, summaries deletable one by one); layers and collisions (§ 3.11:
five layers, one popup at a time, the notice one at a time and waiting under popups and dialogs, running operations
in the status bar, the placement rules near the edges and the bottom-right corner, bars that shorten instead of
wrapping); export (§ 6.18: from the app menu, the context menu and the map menu; CSV, JSON, HTML report and map
image; the selection's hidden items and the filters said in the dialog; a snapshot written to a `.part` file;
progress in the status bar and a notice to show or open it); the CSV separator as a setting (§ 6.10); the app menu,
the command palette, the shortcuts dialog and About (§ 6.19: commands that cannot run stay with their reason, English
words found in the Italian interface, the GPL text and the third-party licences in the app); the keyboard shortcuts list
on Ctrl+\ (§ 6.2); «Analyze with DiskDome» and the command line (§ 6.20: one window, the right tab, nothing read twice,
errors as notices that stay); the Tree and the File list (§ 5.10: columns and the
«Columns…» menu, sort with arrow and Ctrl+Shift+1…9, resize and reorder by dragging, special entries, pending rows;
§ 4.1: what is shown but not counted is `ink-faint`, and online-only files count 0).

**Still open**: none.

## 12. Tests before the code relies on them

Spikes and measurements, each a Trello card, each with a clear «passes if». The design waits on 1, 2 and 6;
the rest is checked in the Kit as the components are built.

| # | Test | How | Passes if |
| --- | --- | --- | --- |
| 1 | **Own title bar with tabs** in Slint on Windows 10 and 11 | a spike window with `no-frame`, our caption buttons, tabs, drag area; resize edges; `crates/platform` handles `WM_NCHITTEST` (Snap Layouts on the maximize button) | drag and double-click to maximize, Snap Layouts flyout on Windows 11, Aero Snap, resize from every edge, Alt+Space menu, no flicker at 100–200 % DPI and across two monitors, dark / light |
| 2 | **Memory with several drives** | step 3's engine: C: + D: + a USB drive scanned together, peak and steady memory with `diskdome-cli` | the total of the open models stays within the limit we set (proposal: 2.5 GB), or the release policy of § 5.27 is needed |
| 3 | **Live updates on several drives** | USN journal readers on 2–3 volumes, idle for 10 min, then a burst of 100 000 file changes (`npm install`) | idle CPU ≈ 0 %, the burst applied within 1 s, no missed change |
| 4 | **Map styles in the CPU renderer** (done 2026-11-24: redraw 0.4–1.9 ms, Cushion the slowest) | each style at 1920 × 1080 with 50 000 rectangles, release build | redraw ≤ 16 ms per style (gradients per rectangle are the costliest) |
| 5 | **Free space values** | `GetDiskFreeSpaceExW` on NTFS, exFAT, a network share with quotas | used + free = capacity within 1 %, the quota case explained in the tooltip |
| 6 | **People test** | the mockup's «Map» and «Both» views, then Overview, Rings and «Colours: Age», shown to 3 people who never used a disk analyzer: «what takes the most space? where is it? what haven't you touched in years?» | they answer in < 30 s without help; the view and style they find clearest becomes the default |
| 7 | **Text and digits** (done: software renderer 2026-11-25, Segoe UI 400 / 700 2026-11-26, § 3.3) | the Kit's «Text and digits» section at 12–13 px with Segoe UI and Segoe UI Variable, femtovg, Skia and software | digits tabular, text crisp (owner's eye) |
| 8 | **Slint menus and tooltips** (done 2026-11-26: the menu cannot take the tokens; the tooltip can, with our content, but not its keyboard behaviour; both ours, decision 11) | `ContextMenuArea`, `Menu`, `Tooltip` with our tokens, off screen (a throwaway spike, removed) | they take our colours, radii and sizes; otherwise ours (§ 5.16, § 5.17) |
| 9 | **Contrast of every theme and palette** | the automatic test of § 10, extended to the Soft / Vivid palettes and the age ramp | every pair of § 7 passes |
| 10 | **Map names** (placement done 2026-11-24: ≤ 0.05 ms; Slint's drawing of 700 names still to check) | placement in Rust on 50 000 rectangles at 1920 × 1080 with «All»; Slint draws 700 names over the image while the window is resized | placement ≤ 2 ms (release); resizing stays smooth (≥ 60 fps); otherwise «All» is dropped and «Many» capped lower |
| 11 | **Windows' own tools** | each button of «Windows' space» (§ 4.5) on Windows 10 and 11, as a standard user | the right tool opens; DiskDome never elevates (a tool that needs it shows its own UAC prompt) |
| 12 | **Dates and allocated sizes from the MFT** | step 3's reader: last-modified date, allocated size, compressed and sparse flags, link count for 1 000 sample files | equal to what Explorer's Properties shows; the age buckets and the «lost space» tiles add up to the drive's numbers |
| 13 | **The new per-entry fields** (done 2026-11-24: 61.5 bytes per entry, peak 0.89 GB) | allocated size, modified time, attributes, owner and flags added to the model, `diskdome-cli` on the reference drive | the targets of `docs/architecture.md` § Performance targets still met; the numbers in the PR |
| 14 | **Saved summaries** | write and read a summary of the reference drive; compare two | ≤ 3 MB, written in < 200 ms after the scan, read in < 100 ms at launch |
| 15 | **Narrator** | the Kit's components and the table, map and charts with Narrator on Windows 11; announcements of a finished scan and of an error | every element of § 7 read as written; announcements without moving the focus, or a fallback (a hidden live text) |
| 16 | **Drive removed** | unplug a USB drive during an MFT scan, a folder walk and a move | the scan stops cleanly with the info bar of § 6.14, the model stays readable, no file action half-done without a report |
| 17 | **Long paths** | open, rename, move and delete files with paths over 260 characters | every action works through long-path APIs; File Explorer's limit is reported as in § 6.14 |
