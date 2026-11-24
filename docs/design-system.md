# Design system: starting point

Step 4 of the plan produces the detailed design system. This is where it starts from. Interface and design
decisions are taken with the owner, not autonomously.

## Path

1. Principles.
2. A mockup of the main window at real size, already compact (toolbar, tree, treemap, extension panel).
3. Tokens taken from the mockup.
4. Components with their variants and states.
5. The Kit window.
6. Automatic checks.

## References

The owner's other projects (outside this repo):

| Project | What we take | Limit |
| --- | --- | --- |
| EmuDome | structure and checks: tokens per family with the default scales reset, 27 primitives with `as const` variants, the Kit page (also a review board, «Now» / «Proposal»), a self-tested convention scanner, the design system's import boundary | built for a console-style app, in Vue / Tailwind / Reka |
| NoteDome | the best-written principles | no automatic checks |
| rapportino | normal / compact density from one size scale | a mobile app |
| GameDome | the `tokens.json` EmuDome derives from | less evolved |

## From EmuDome to Slint

| EmuDome (Vue / Tailwind) | DiskDome (Slint) |
| --- | --- |
| tokens in 9 CSS files, Tailwind's scales reset | typed `global Tokens` and `global Theme` (`length`, `color`, `duration`): a wrong token does not compile |
| `as const` variants (`ButtonVariant.Primary`) | `enum ButtonVariant` |
| primitives in `design-system/components/` | `.slint` components in `crates/ui/ui/design-system/`, one file each |
| Kit page `#kit` | a dedicated Kit window + Slint's live preview |
| `scan-conventions.mjs` | a CI script: no `px` and no colour written by hand outside the design system |
| ESLint boundary | screens import only from the design system |

## Starting values (to confirm on the mockup)

From the owner's decision of 2026-11-22 in EmuDome ("giant buttons, make everything more compact").

| Element | Value |
| --- | --- |
| Table row | 22 px |
| Controls | 24 / 28 / 32 px (small / normal / large) |
| Text | body 13/18, caption 12/16, overline 11/14 |
| Radii | 2 / 2 / 3 / 4 px (badge / control / card / panel) |
| Card padding | 10 px |
| Numbers | tabular figures, right-aligned |
| Colour | neutral graphite; colour comes from the data (file types in the treemap); the accent only on the main action |
| Density | normal / compact from one scale |

## Principles (from NoteDome)

- Hierarchy by lightness, not by colour: a darker background sits deeper.
- Subtle layers: a raised element = a slightly lighter background + a 1 px low-opacity border + a soft shadow.
- Typography as structure, instead of dividers and boxes.
- The accent in small doses: the main action, the selection, the active state.

## Slint shape (style reference)

```slint
export enum ButtonVariant { primary, soft, ghost }

export global Tokens {
    out property <length> control-sm: 24px;
    out property <length> radius-control: 2px;
    out property <length> space-8: 8px;
    out property <duration> fast: 150ms;
}

export global Theme {
    in-out property <bool> dark: true;
    out property <color> bg:      dark ? #17181a : #ffffff;
    out property <color> surface: dark ? #222326 : #f4f4f5;
    out property <color> ink:     dark ? #e6e6e6 : #1a1a1a;
    out property <color> accent:  #faa40f;
}
```

The colours above are placeholders: the real palette comes from the mockup.
