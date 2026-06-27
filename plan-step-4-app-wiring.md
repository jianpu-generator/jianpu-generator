# Step 4: Wire up Edit Metadata in `web/src/App.tsx`

## Goal
Connect all the pieces built in Steps 1–3: parse metadata from source, handle field changes, pass the new prop to Editor, and render the EditMetadataModal.

## Prerequisites
Steps 1–3 must already be complete:
- `web/src/utils/metadataSource.ts` exports `parseMetadata`, `updateMetadataField`, `ParsedMetadataFields`, `MetadataKey`
- `web/src/components/EditMetadataModal.tsx` exports `EditMetadataModal`
- `web/src/components/Editor.tsx` accepts `onEditMetadataClick?: () => void`

## Changes to `web/src/App.tsx`

Read the full `App.tsx` first. Key locations:
- Line 65: `const [editPartsOpen, setEditPartsOpen] = useState(false)` — add `editMetadataOpen` right after it
- Lines 274–277: `partDeclarations` memo — add `parsedMetadata` memo after it
- Lines 279–296: `handlePartDeclarationChange` callback — add `handleMetadataFieldChange` after it
- Line 375: `onEditPartsClick={() => setEditPartsOpen(true)}` on `<Editor>` — add `onEditMetadataClick` here
- Lines 434–443: `<EditPartsModal .../>` — add `<EditMetadataModal .../>` right after it

### 1. Imports to add (top of file, near existing imports)
```ts
import { EditMetadataModal } from './components/EditMetadataModal'
import type { MetadataKey } from './utils/metadataSource'
import { parseMetadata, updateMetadataField } from './utils/metadataSource'
```

### 2. State (after line 65)
```ts
const [editMetadataOpen, setEditMetadataOpen] = useState(false)
```

### 3. Memo (after the `partDeclarations` memo, ~line 277)
```ts
const parsedMetadata = useMemo(() => parseMetadata(source), [source])
```

### 4. Callback (after `handlePartDeclarationChange`, ~line 296)
```ts
const handleMetadataFieldChange = useCallback(
  (key: MetadataKey, value: string | null) => {
    handleSourceChange(updateMetadataField(source, key, value))
  },
  [source, handleSourceChange],
)
```

### 5. Editor prop (on `<Editor>` component, ~line 375, alongside existing `onEditPartsClick`)
```ts
onEditMetadataClick={() => setEditMetadataOpen(true)}
```

### 6. Render modal (right after `<EditPartsModal .../>`, ~line 443)
```tsx
<EditMetadataModal
  open={editMetadataOpen}
  onOpenChange={setEditMetadataOpen}
  metadata={parsedMetadata}
  onFieldChange={handleMetadataFieldChange}
/>
```

## Verification
After all changes, run:
```sh
cd web && npm run build
```
It should compile with no TypeScript errors. Then run the dev server and confirm:
1. "Edit Metadata" Code Lens appears above `# metadata` in the editor
2. Clicking it opens the dialog with current values populated
3. Changing a field updates the source text live
4. Clearing an optional field removes its line from the source
