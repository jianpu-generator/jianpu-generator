# Step 2: Create `web/src/components/EditMetadataModal.tsx`

## Goal
Create a new dialog component for editing metadata fields. It mirrors the structure of `EditPartsModal.tsx`. No existing files are modified.

## Prerequisite
Step 1 must already be done: `web/src/utils/metadataSource.ts` must exist and export `ParsedMetadataFields` and `MetadataKey`.

## Background
The existing Edit Parts dialog (`web/src/components/EditPartsModal.tsx`) is opened via a Code Lens in the Monaco editor. This new dialog follows the same pattern for the metadata section. Read `EditPartsModal.tsx` in full before implementing — replicate its visual style (same dialog structure, same `thStyle`/`tdStyle` constants, same Radix UI primitives).

## Props interface
```ts
export interface EditMetadataModalProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  metadata: ParsedMetadataFields
  onFieldChange: (key: MetadataKey, value: string | null) => void
}
```

## Dialog layout
Use `@radix-ui/react-dialog` (already installed — check `web/package.json`).

The dialog body is a two-column table (`Field` | `Value`), one row per metadata field:

| Field label | Control | Behavior |
|---|---|---|
| Title * | `<input type="text">` | Required; calls `onFieldChange('title', value)` on change |
| Subtitle | `<input type="text">` | Optional; empty string → `onFieldChange('subtitle', null)` |
| Author | `<input type="text">` | Optional; empty string → `onFieldChange('author', null)` |
| Row Height | `<input type="number" min="1">` | Optional; empty → `onFieldChange('row height', null)` |
| Max Columns | `<input type="number" min="1">` | Optional; empty → `onFieldChange('max columns', null)` |
| Label Width | `<input type="number" min="1">` | Optional; empty → `onFieldChange('label width', null)` |
| Note Number Width | `<input type="number" min="1">` | Optional; empty → `onFieldChange('note number width', null)` |

Changes are applied immediately on `onChange` (same as EditPartsModal — no Save button needed).

For number inputs, convert `metadata.rowHeight` (etc.) to string for the `value` prop, and `null` to `''`.

## Style notes (mirror EditPartsModal)
- Dialog overlay: `position: fixed; inset: 0; background: rgba(0,0,0,0.35); zIndex: 1000`
- Dialog content: centered with `transform: translate(-50%, -50%)`, `minWidth: 420px`, `fontFamily: var(--mono, monospace)`
- `thStyle`: `padding: 6px 10px; fontWeight: 600; fontSize: 12px; color: #444; borderBottom: 2px solid #ddd; background: #f5f5f5`
- `tdStyle`: `padding: 6px 10px; borderBottom: 1px solid #eee; verticalAlign: middle; fontSize: 13px`
- Input style: `fontSize: 12px; fontFamily: var(--mono, monospace); border: 1px solid #cbd5e0; borderRadius: 3px; padding: 2px 6px; width: 100%; boxSizing: border-box`

## Files to read before implementing
- `web/src/components/EditPartsModal.tsx` — full file, the visual and structural model
- `web/src/utils/metadataSource.ts` — for types (ParsedMetadataFields, MetadataKey)
- `web/package.json` — confirm @radix-ui/react-dialog is available
