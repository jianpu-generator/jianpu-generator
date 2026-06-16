# Cmd+Enter Play-Measure Shortcut Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a global `Cmd+Enter` (Mac) / `Ctrl+Enter` (Win/Linux) shortcut to play the current measure, and update the `PlayMeasureButton` tooltip to show the shortcut.

**Architecture:** Use `@github/hotkey` with its element-based API — a hidden `<button>` with a `data-hotkey` attribute is installed via `install(el)` in a `useEffect`; pressing the hotkey synthesises a click on that button, which calls `playSelectedMeasures()` when enabled. Platform detection (`navigator.platform`) is computed once at module level to derive the tooltip label.

**Tech Stack:** React 19, `@github/hotkey` (new dep), Playwright (e2e tests)

---

## Files

| Action | Path | Responsibility |
|--------|------|----------------|
| Modify | `web/package.json` | add `@github/hotkey` dependency |
| Modify | `web/src/components/PlayMeasureButton.tsx` | accept `shortcutLabel` prop, update title |
| Modify | `web/src/App.tsx` | hidden hotkey button + install/uninstall |
| Create | `web/e2e/cmd-enter-play.spec.ts` | e2e test for the shortcut |

---

### Task 1: Install @github/hotkey

**Files:**
- Modify: `web/package.json`

- [ ] **Step 1: Install the package**

```bash
cd web && pnpm add @github/hotkey
```

Expected output: package added to `dependencies` in `package.json` and `pnpm-lock.yaml` updated.

- [ ] **Step 2: Verify types are available**

```bash
pnpm exec tsc --noEmit 2>&1 | head -20
```

Expected: no new type errors (zero output or pre-existing errors only).

---

### Task 2: Update PlayMeasureButton to accept shortcutLabel

**Files:**
- Modify: `web/src/components/PlayMeasureButton.tsx`

- [ ] **Step 1: Add `shortcutLabel` prop and update `title`**

Replace the entire file with:

```tsx
interface PlayMeasureButtonProps {
  disabled: boolean
  loading: boolean
  measureRange: { start: number; end: number } | null
  onClick: () => void
  shortcutLabel: string
}

function measureLabel(range: { start: number; end: number }): string {
  if (range.start === range.end) {
    return `▶ Measure ${range.start + 1}`
  }
  return `▶ Measures ${range.start + 1}–${range.end + 1}`
}

export function PlayMeasureButton({
  disabled,
  loading,
  measureRange,
  onClick,
  shortcutLabel,
}: PlayMeasureButtonProps) {
  const label = measureRange !== null ? measureLabel(measureRange) : null
  return (
    <button
      type="button"
      className="play-measure-btn"
      disabled={disabled}
      onClick={onClick}
      title={
        measureRange === null
          ? 'Move cursor into a measure to enable'
          : `Play selected measure(s) (${shortcutLabel})`
      }
      aria-label={label ?? 'Play selected measure(s)'}
    >
      {loading ? (
        <span className="play-measure-spinner" aria-hidden="true" />
      ) : label !== null ? (
        label
      ) : (
        '▶'
      )}
    </button>
  )
}
```

- [ ] **Step 2: Type-check**

```bash
cd web && pnpm exec tsc --noEmit 2>&1 | head -30
```

Expected: error in `App.tsx` about missing `shortcutLabel` prop (confirms the prop was added). No other new errors.

---

### Task 3: Wire up shortcut and shortcutLabel in App.tsx

**Files:**
- Modify: `web/src/App.tsx`

- [ ] **Step 1: Add imports and platform constant at top of file**

After the existing imports block, add:

```tsx
import { install, uninstall } from '@github/hotkey'

const isMac = navigator.platform.startsWith('Mac')
const shortcutLabel = isMac ? '⌘↵' : 'Ctrl+↵'
```

- [ ] **Step 2: Add refs for the two hidden hotkey buttons**

Inside the `App` function, after the existing refs, add:

```tsx
const hotkeyMetaRef = useRef<HTMLButtonElement>(null)
const hotkeyCtrlRef = useRef<HTMLButtonElement>(null)
```

- [ ] **Step 3: Add useEffect to install/uninstall the hotkeys**

After the existing `useEffect` blocks in `App`, add:

```tsx
useEffect(() => {
  const metaEl = hotkeyMetaRef.current
  const ctrlEl = hotkeyCtrlRef.current
  if (metaEl) install(metaEl)
  if (ctrlEl) install(ctrlEl)
  return () => {
    if (metaEl) uninstall(metaEl)
    if (ctrlEl) uninstall(ctrlEl)
  }
}, [])
```

- [ ] **Step 4: Add the hidden hotkey buttons and pass shortcutLabel to PlayMeasureButton**

Inside the `toolbar` JSX (where `<PlayMeasureButton>` is rendered), add two hidden buttons (one per platform key) and the new prop. Replace:

```tsx
<PlayMeasureButton
  disabled={
    selectedMeasureRange === null ||
    measureAudioGenerating
  }
  loading={measureAudioGenerating}
  measureRange={selectedMeasureRange}
  onClick={playSelectedMeasures}
/>
```

with:

```tsx
<>
  {/* Meta+Enter = Cmd+Enter on Mac */}
  <button
    ref={hotkeyMetaRef}
    type="button"
    hidden
    data-hotkey="Meta+Enter"
    onClick={() => {
      if (selectedMeasureRange !== null && !measureAudioGenerating) {
        playSelectedMeasures()
      }
    }}
  />
  {/* Control+Enter = Ctrl+Enter on Win/Linux */}
  <button
    ref={hotkeyCtrlRef}
    type="button"
    hidden
    data-hotkey="Control+Enter"
    onClick={() => {
      if (selectedMeasureRange !== null && !measureAudioGenerating) {
        playSelectedMeasures()
      }
    }}
  />
  <PlayMeasureButton
    disabled={
      selectedMeasureRange === null ||
      measureAudioGenerating
    }
    loading={measureAudioGenerating}
    measureRange={selectedMeasureRange}
    onClick={playSelectedMeasures}
    shortcutLabel={shortcutLabel}
  />
</>
```

- [ ] **Step 5: Type-check**

```bash
cd web && pnpm exec tsc --noEmit 2>&1 | head -30
```

Expected: zero errors.

- [ ] **Step 6: Commit**

```bash
git add web/package.json web/pnpm-lock.yaml web/src/components/PlayMeasureButton.tsx web/src/App.tsx
git commit -m "feat: cmd+enter global play-measure shortcut with tooltip"
```

---

### Task 4: E2e test for the shortcut

**Files:**
- Create: `web/e2e/cmd-enter-play.spec.ts`

- [ ] **Step 1: Write the test**

```typescript
import { expect, test } from '@playwright/test'

/**
 * The default demo source has measure 1 starting at line 12.
 * Pressing Meta+Enter (or Control+Enter on Linux) while the cursor
 * is inside a measure should trigger playback (the play button enters
 * its loading/spinner state).
 */
test('Meta+Enter triggers play when cursor is inside a measure', async ({
  page,
}) => {
  await page.goto('/')
  await page.waitForSelector('.editor-toolbar', { timeout: 15_000 })

  // Click into the editor and navigate to line 12 (first note line of measure 1).
  await page.click('.monaco-editor .view-lines')
  await page.keyboard.press('Control+g')
  await page.keyboard.type('12')
  await page.keyboard.press('Enter')

  // Wait for the play button to reflect the selected measure.
  const playBtn = page.locator('.play-measure-btn')
  await expect(playBtn).not.toBeDisabled({ timeout: 5_000 })

  // Press the global shortcut. Use Meta+Enter (Mac); Playwright maps this
  // correctly on all platforms in headed mode.
  await page.keyboard.press('Meta+Enter')

  // The button should enter a loading/spinner state momentarily.
  const spinner = page.locator('.play-measure-spinner')
  await expect(spinner).toBeVisible({ timeout: 5_000 })
})

test('Meta+Enter does nothing when cursor is outside all measures', async ({
  page,
}) => {
  await page.goto('/')
  await page.waitForSelector('.editor-toolbar', { timeout: 15_000 })

  // Click into the editor and navigate to line 1 (metadata — outside measures).
  await page.click('.monaco-editor .view-lines')
  await page.keyboard.press('Control+g')
  await page.keyboard.type('1')
  await page.keyboard.press('Enter')

  // Play button should be disabled.
  const playBtn = page.locator('.play-measure-btn')
  await expect(playBtn).toBeDisabled({ timeout: 5_000 })

  // Press shortcut — nothing should happen (no spinner appears).
  await page.keyboard.press('Meta+Enter')
  await page.waitForTimeout(500)

  const spinner = page.locator('.play-measure-spinner')
  await expect(spinner).not.toBeVisible()
})
```

- [ ] **Step 2: Run the tests**

```bash
cd web && pnpm exec playwright test e2e/cmd-enter-play.spec.ts --headed
```

Expected: both tests pass. If the spinner check is too fast, increase the timeout.

- [ ] **Step 3: Commit**

```bash
git add web/e2e/cmd-enter-play.spec.ts
git commit -m "test: e2e tests for cmd+enter play-measure shortcut"
```
