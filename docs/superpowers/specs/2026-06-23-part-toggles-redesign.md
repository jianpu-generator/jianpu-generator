# Part Toggles Redesign

**Date:** 2026-06-23

## Goal

Replace the current multi-pill layout (separate pills for visibility, solo, and lyrics) with a single segmented pill per part.

## Current Layout

Each part renders as separate pills:

```
[Eye/EyeOff  ABBR]  [Headphones]  ──  [☐ lyrics]
```

- The eye+abbreviation pill toggles part visibility
- The headphones pill toggles solo
- The lyrics pill (conditional) toggles lyrics visibility, connected by a line

## New Layout

Each part renders as one segmented pill:

```
┌──────┬──┬──┐       ┌──────┬──┬──┬──┐
│ ABBR │👁│🎧│       │ ABBR │👁│🎧│🎤│
└──────┴──┴──┘       └──────┴──┴──┴──┘
 (no lyrics)          (has_lyrics && enabled)
```

### Segments

| Segment | Content | Behaviour | Tooltip |
|---------|---------|-----------|---------|
| ABBR | Part abbreviation (monospace) | Non-interactive label | — |
| Eye | `Eye` / `EyeOff` icon | Toggles part visibility | "Show/Hide" |
| Headphones | `Headphones` icon | Toggles solo | "Solo" |
| Mic | `Mic` icon | Toggles lyrics | "Lyrics" |

### Conditional rendering

The **Mic segment** appears only when `part.has_lyrics && !disabledParts.has(part.abbreviation)` — the same condition as the current lyrics pill.

### Active states

- **Eye (enabled):** blue accent (`--accent-selected-bg`, `--accent-selected-border`, `--accent-selected-text`)
- **Headphones (soloed):** amber (`#f59e0b` family, same as current)
- **Mic (lyrics enabled):** blue accent (same as Eye active)
- **Disabled part:** entire pill dims to `opacity: 0.4`

### Segmented pill styling

- Single outer border (`border-radius: 4px`) wrapping all segments
- Internal vertical dividers between segments (using `border-left` on each icon segment)
- No gaps between segments — they share the border
- ABBR segment has slightly more horizontal padding to read as a label
- Icon segments are square-ish with equal padding on all sides

## Tooltips

Use `@radix-ui/react-tooltip`. Wrap the app (or the `PartToggles` component subtree) in `<TooltipProvider>`. Each icon segment is wrapped in a `<Tooltip>` with the label above.

Tooltip text:
- Eye segment: `"Show/Hide"`
- Headphones segment: `"Solo"`
- Mic segment: `"Lyrics"`

## Files Changed

| File | Change |
|------|--------|
| `web/package.json` | Add `@radix-ui/react-tooltip` dependency |
| `web/src/components/PartToggles.tsx` | Rewrite layout to segmented pill |
| `web/src/components/PartToggles.css` | Rewrite styles for segmented pill |
| `web/src/App.tsx` | Wrap with `<TooltipProvider>` if needed |

## Out of Scope

- No change to toggle logic or state management
- No change to `partToggleCache.ts`, `useJianpuWorker.ts`, or any other file
