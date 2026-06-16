# Design: Cmd+Enter Global Play-Measure Shortcut

## Summary

Add a global `Cmd+Enter` (Mac) / `Ctrl+Enter` (Win/Linux) keyboard shortcut that plays the current measure, and update the `PlayMeasureButton` tooltip to show the shortcut.

## Shortcut Registration

- Install `@github/hotkey` as a runtime dependency.
- In `App.tsx`, register the shortcut in a `useEffect` using `install` from `@github/hotkey`.
- Bind both `Meta+Enter` (Mac Cmd key) and `Control+Enter` (Win/Linux Ctrl key) to the same handler.
- The handler calls `playSelectedMeasures()` only when the button is enabled (`selectedMeasureRange !== null && !measureAudioGenerating`).
- Return the `uninstall` cleanup function from the effect.
- Target: `document.body` (global).

## Tooltip Update

- Add a `shortcutLabel` prop to `PlayMeasureButton` (type `string`).
- Detect platform once at module level: `navigator.platform.includes('Mac')`.
- On Mac: `shortcutLabel = "⌘↵"`, on Win/Linux: `shortcutLabel = "Ctrl+↵"`.
- The `title` attribute when a measure is selected becomes: `"Play selected measure(s) (⌘↵)"` (or platform equivalent).
- When no measure is selected: `"Move cursor into a measure to enable"` (unchanged).

## Files Changed

- `web/package.json` — add `@github/hotkey`
- `web/src/App.tsx` — register/unregister shortcut, compute `shortcutLabel`, pass to `PlayMeasureButton`
- `web/src/components/PlayMeasureButton.tsx` — accept `shortcutLabel` prop, update `title`
