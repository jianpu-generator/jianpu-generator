# Step 3: Add "Edit Metadata" Code Lens to `web/src/components/Editor.tsx`

## Goal
Add an `onEditMetadataClick` optional prop to the Editor component and register a Code Lens that appears above the `# metadata` line, triggering that callback. This is a backward-compatible change — the new prop is optional, so all existing callers (`App.tsx`) continue to compile unchanged.

## Background
The editor already has an "Edit Parts" Code Lens. Read `Editor.tsx` lines 21–33 (EditorProps), 120–139 (refs), 315–372 (handleMount — command registration and provideCodeLenses) carefully before making changes.

Key existing pattern to mirror:
1. `onEditPartsClickRef` is a ref that tracks the `onEditPartsClick` prop (line 136)
2. A `useEffect` syncs the prop into the ref (look for the effect that sets `onEditPartsClickRef.current`)
3. In `handleMount`, `ed.addCommand(0, () => onEditPartsClickRef.current?.())` registers the command (line 346)
4. `provideCodeLenses` scans lines for `'# parts'` and emits a lens (lines 354–368)

## Changes to make

### 1. `EditorProps` interface (around line 33)
Add:
```ts
onEditMetadataClick?: () => void
```

### 2. Destructure in component function (around line 126)
Add `onEditMetadataClick` to the destructured props.

### 3. Add ref (after line 136)
```ts
const onEditMetadataClickRef = useRef(onEditMetadataClick)
```

### 4. Add useEffect to sync ref (mirror the existing effect for `onEditPartsClickRef`)
```ts
useEffect(() => {
  onEditMetadataClickRef.current = onEditMetadataClick
}, [onEditMetadataClick])
```

### 5. In `handleMount` — register command (after the editPartsCommandId block, ~line 348)
```ts
const editMetadataCommandId = ed.addCommand(0, () => {
  onEditMetadataClickRef.current?.()
})
```

### 6. In `provideCodeLenses` — add lens for `# metadata` (inside the existing loop)
After emitting the parts lens, also check for `'# metadata'`:
```ts
if (model.getLineContent(line).trim() === '# metadata') {
  lenses.push({
    range: new monacoApi.Range(line, 1, line, 1),
    command: {
      id: editMetadataCommandId ?? '',
      title: 'Edit Metadata',
    },
  })
}
```
Remove the `break` after the parts lens so both sections can be found in one pass.

## Files to read before implementing
- `web/src/components/Editor.tsx` — full file (especially lines 21–33, 120–145, 310–375)
