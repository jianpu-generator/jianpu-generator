# Part Toggles Redesign Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the current multi-pill part toggle layout with a single segmented pill per part showing `[ABBR | 👁 | 🎧 | 🎤]`.

**Architecture:** Rewrite `PartToggles.tsx` and `PartToggles.css` to render one segmented pill per part. Add `@radix-ui/react-tooltip` for tooltips on icon segments. `TooltipProvider` lives inside `PartToggles` so no other file needs changing.

**Tech Stack:** React 19, lucide-react, @radix-ui/react-tooltip, CSS custom properties from the existing design system.

## Global Constraints

- Icon size: `14` (matches existing usage of lucide-react icons in this file)
- Use existing CSS custom properties: `--muted`, `--border`, `--accent`, `--accent-selected-bg`, `--accent-selected-border`, `--accent-selected-text`, `--accent-selected-bg-hover`, `--editor-bg`, `--preview-bg`, `--mono`
- Amber solo colour values: `#f59e0b` (border/active), `color-mix(in srgb, #f59e0b 12%, var(--editor-bg))` (bg), `#b45309` (icon colour) — same as current
- The existing e2e test locates checkboxes via `.part-toggles input[type="checkbox"]` — hidden checkboxes must remain in the DOM
- Lyrics icon: `Mic` from lucide-react
- Tooltip labels: `"Show/Hide"`, `"Solo"`, `"Lyrics"` (no part name)

---

### Task 1: Install @radix-ui/react-tooltip

**Files:**
- Modify: `web/package.json` (pnpm adds it automatically)
- Modify: `web/pnpm-lock.yaml` (auto-updated)

**Interfaces:**
- Produces: `@radix-ui/react-tooltip` importable as `import * as Tooltip from '@radix-ui/react-tooltip'`

- [ ] **Step 1: Install the package**

```bash
cd web && pnpm add @radix-ui/react-tooltip
```

Expected output: something like `+ @radix-ui/react-tooltip 1.x.x` with no errors.

- [ ] **Step 2: Verify the import resolves**

```bash
cd web && node -e "require.resolve('@radix-ui/react-tooltip')" 2>/dev/null && echo OK || echo FAIL
```

Expected: `OK`

- [ ] **Step 3: Commit**

```bash
git add web/package.json web/pnpm-lock.yaml
git commit -m "chore(web): add @radix-ui/react-tooltip"
```

---

### Task 2: Rewrite PartToggles.css

**Files:**
- Modify: `web/src/components/PartToggles.css` (full rewrite)

**Interfaces:**
- Produces CSS classes consumed by Task 3:
  - `.part-toggles` — fieldset wrapper (flex row, wraps)
  - `.part-toggles-label` — "Parts" legend
  - `.part-toggles-status` — "Updating…" text
  - `.part-toggles-list` — `<ul>` of pills
  - `.part-toggle-pill` — the segmented pill `<div>` wrapping one part
  - `.part-toggle-abbr` — the non-interactive abbreviation left segment
  - `.part-toggle-segment` — shared class for every icon `<label>` segment
  - `.part-toggle-segment--eye` — visibility toggle segment
  - `.part-toggle-segment--headphones` — solo toggle segment
  - `.part-toggle-segment--mic` — lyrics toggle segment
  - `.part-toggle-tooltip-content` — Radix tooltip bubble

- [ ] **Step 1: Replace the entire file content**

Open `web/src/components/PartToggles.css` and replace everything with:

```css
.part-toggles {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 0.5rem 0.75rem;
  margin: 0;
  padding: 0;
  border: 0;
  min-inline-size: 0;
}

.part-toggles-label {
  font-size: 0.75rem;
  font-weight: 600;
  color: var(--muted);
  text-transform: uppercase;
  letter-spacing: 0.04em;
  padding: 0;
  margin: 0;
}

.part-toggles-status {
  font-size: 0.75rem;
  color: var(--muted);
  font-style: italic;
}

.part-toggles-list {
  display: flex;
  flex-wrap: wrap;
  gap: 0.35rem 0.75rem;
  margin: 0;
  padding: 0;
  list-style: none;
}

/* Segmented pill */
.part-toggle-pill {
  display: inline-flex;
  align-items: stretch;
  border: 1px solid color-mix(in srgb, var(--muted) 45%, var(--border));
  border-radius: 4px;
  overflow: hidden;
  background: var(--editor-bg);
}

/* Non-interactive abbreviation segment */
.part-toggle-abbr {
  display: inline-flex;
  align-items: center;
  padding: 0.25rem 0.55rem;
  font-family: var(--mono);
  font-size: 0.75rem;
  color: var(--muted);
  user-select: none;
  border-right: 1px solid color-mix(in srgb, var(--muted) 30%, var(--border));
}

/* Icon segments */
.part-toggle-segment {
  position: relative;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  padding: 0.25rem 0.45rem;
  border-left: 1px solid color-mix(in srgb, var(--muted) 30%, var(--border));
  cursor: pointer;
  color: var(--muted);
  user-select: none;
  transition:
    background 0.15s ease,
    color 0.15s ease;
}

/* Hide native checkbox — it stays in the DOM for e2e tests */
.part-toggle-segment input[type="checkbox"] {
  position: absolute;
  opacity: 0;
  width: 0;
  height: 0;
  pointer-events: none;
}

/* Hover (unchecked, enabled) */
.part-toggle-segment:hover:not(:has(input:disabled)):not(:has(input:checked)) {
  background: var(--preview-bg);
  color: var(--muted);
}

/* Focus ring */
.part-toggle-segment:has(input:focus-visible) {
  outline: 2px solid color-mix(in srgb, var(--accent) 50%, transparent);
  outline-offset: -2px;
}

/* Disabled */
.part-toggle-segment:has(input:disabled) {
  opacity: 0.4;
  cursor: not-allowed;
}

/* Eye: active = part is visible (blue) */
.part-toggle-segment--eye:has(input:checked) {
  background: var(--accent-selected-bg);
  color: var(--accent-selected-text);
}

.part-toggle-segment--eye:has(input:checked):hover:not(:has(input:disabled)) {
  background: var(--accent-selected-bg-hover);
}

/* Headphones: active = soloed (amber) */
.part-toggle-segment--headphones:has(input:checked) {
  background: color-mix(in srgb, #f59e0b 12%, var(--editor-bg));
  color: #b45309;
}

.part-toggle-segment--headphones:has(input:checked):hover:not(:has(input:disabled)) {
  background: color-mix(in srgb, #f59e0b 20%, var(--editor-bg));
}

/* Mic: active = lyrics visible (blue, same as eye) */
.part-toggle-segment--mic:has(input:checked) {
  background: var(--accent-selected-bg);
  color: var(--accent-selected-text);
}

.part-toggle-segment--mic:has(input:checked):hover:not(:has(input:disabled)) {
  background: var(--accent-selected-bg-hover);
}

/* Radix tooltip bubble */
.part-toggle-tooltip-content {
  background: var(--editor-bg, #fff);
  color: var(--muted);
  border: 1px solid var(--border);
  border-radius: 4px;
  padding: 0.2rem 0.5rem;
  font-size: 0.7rem;
  box-shadow: 0 2px 6px rgba(0, 0, 0, 0.12);
  user-select: none;
  z-index: 100;
}
```

- [ ] **Step 2: Commit**

```bash
git add web/src/components/PartToggles.css
git commit -m "style(web): rewrite PartToggles CSS for segmented pill layout"
```

---

### Task 3: Rewrite PartToggles.tsx

**Files:**
- Modify: `web/src/components/PartToggles.tsx` (full rewrite)

**Interfaces:**
- Consumes CSS classes from Task 2: `.part-toggle-pill`, `.part-toggle-abbr`, `.part-toggle-segment`, `.part-toggle-segment--eye`, `.part-toggle-segment--headphones`, `.part-toggle-segment--mic`, `.part-toggle-tooltip-content`
- Consumes: `@radix-ui/react-tooltip` — `Provider`, `Root`, `Trigger`, `Portal`, `Content`
- Consumes: lucide-react — `Eye`, `EyeOff`, `Headphones`, `Mic`
- Preserves props interface `PartTogglesProps` — no changes to callers

- [ ] **Step 1: Replace the entire file content**

Open `web/src/components/PartToggles.tsx` and replace everything with:

```tsx
import * as Tooltip from '@radix-ui/react-tooltip'
import { Eye, EyeOff, Headphones, Mic } from 'lucide-react'
import type { PartInfo } from '../types'
import './PartToggles.css'

interface PartTogglesProps {
  parts: PartInfo[]
  disabledParts: ReadonlySet<string>
  disabledLyrics: ReadonlySet<string>
  soloedParts: ReadonlySet<string>
  onPartToggle: (abbreviation: string, enabled: boolean) => void
  onLyricsToggle: (abbreviation: string, enabled: boolean) => void
  onSoloToggle: (abbreviation: string, soloed: boolean) => void
  loading?: boolean
}

export function PartToggles({
  parts,
  disabledParts,
  disabledLyrics,
  soloedParts,
  onPartToggle,
  onLyricsToggle,
  onSoloToggle,
  loading = false,
}: PartTogglesProps) {
  if (parts.length === 0) {
    return null
  }

  return (
    <Tooltip.Provider delayDuration={400}>
      <fieldset className="part-toggles">
        <legend className="part-toggles-label">Parts</legend>
        {loading ? <span className="part-toggles-status">Updating…</span> : null}
        <ul className="part-toggles-list">
          {parts.map((part) => {
            const enabled = !disabledParts.has(part.abbreviation)
            const lyricsEnabled = !disabledLyrics.has(part.abbreviation)
            const soloed = soloedParts.has(part.abbreviation)

            return (
              <li key={part.abbreviation}>
                <div className="part-toggle-pill">
                  <span className="part-toggle-abbr">{part.abbreviation}</span>

                  <Tooltip.Root>
                    <Tooltip.Trigger asChild>
                      <label className="part-toggle-segment part-toggle-segment--eye">
                        <input
                          type="checkbox"
                          checked={enabled}
                          onChange={(event) =>
                            onPartToggle(part.abbreviation, event.target.checked)
                          }
                        />
                        {enabled ? (
                          <Eye size={14} aria-hidden="true" />
                        ) : (
                          <EyeOff size={14} aria-hidden="true" />
                        )}
                      </label>
                    </Tooltip.Trigger>
                    <Tooltip.Portal>
                      <Tooltip.Content className="part-toggle-tooltip-content" sideOffset={4}>
                        Show/Hide
                      </Tooltip.Content>
                    </Tooltip.Portal>
                  </Tooltip.Root>

                  <Tooltip.Root>
                    <Tooltip.Trigger asChild>
                      <label className="part-toggle-segment part-toggle-segment--headphones">
                        <input
                          type="checkbox"
                          checked={soloed}
                          onChange={(event) =>
                            onSoloToggle(part.abbreviation, event.target.checked)
                          }
                        />
                        <Headphones size={14} aria-hidden="true" />
                      </label>
                    </Tooltip.Trigger>
                    <Tooltip.Portal>
                      <Tooltip.Content className="part-toggle-tooltip-content" sideOffset={4}>
                        Solo
                      </Tooltip.Content>
                    </Tooltip.Portal>
                  </Tooltip.Root>

                  {part.has_lyrics && enabled ? (
                    <Tooltip.Root>
                      <Tooltip.Trigger asChild>
                        <label className="part-toggle-segment part-toggle-segment--mic">
                          <input
                            type="checkbox"
                            checked={lyricsEnabled}
                            onChange={(event) =>
                              onLyricsToggle(part.abbreviation, event.target.checked)
                            }
                          />
                          <Mic size={14} aria-hidden="true" />
                        </label>
                      </Tooltip.Trigger>
                      <Tooltip.Portal>
                        <Tooltip.Content className="part-toggle-tooltip-content" sideOffset={4}>
                          Lyrics
                        </Tooltip.Content>
                      </Tooltip.Portal>
                    </Tooltip.Root>
                  ) : null}
                </div>
              </li>
            )
          })}
        </ul>
      </fieldset>
    </Tooltip.Provider>
  )
}
```

- [ ] **Step 2: Run the TypeScript compiler to check for type errors**

```bash
cd web && npx tsc --noEmit
```

Expected: no output (zero errors).

- [ ] **Step 3: Commit**

```bash
git add web/src/components/PartToggles.tsx
git commit -m "feat(web): redesign part toggles as segmented pills with Radix tooltips"
```

---

### Task 4: Verify e2e tests pass

**Files:** none modified

- [ ] **Step 1: Start the dev server in the background and run e2e tests**

```bash
cd web && pnpm exec playwright test
```

Expected: all tests pass. The existing test in `e2e/part-toggle-while-measure-focused.spec.ts` locates `.part-toggles input[type="checkbox"]` — these hidden checkboxes remain in the DOM so the test should still work.

- [ ] **Step 2: If any test fails, check selector assumptions**

The test uses:
```ts
const firstPartCheckbox = page
  .locator('.part-toggles input[type="checkbox"]')
  .first()
await firstPartCheckbox.uncheck()
```

In the new layout the first `input[type="checkbox"]` inside `.part-toggles` is the Eye checkbox for the first part. Unchecking it calls `onPartToggle` with `enabled=false`, which is the same semantic as before (hides the part). The test only checks that the SVG re-renders — it should pass unchanged.

- [ ] **Step 3: Commit if you made any test fixes**

```bash
git add web/e2e/
git commit -m "fix(e2e): update part toggle selectors for segmented pill layout"
```

(Skip this step if no test fixes were needed.)
