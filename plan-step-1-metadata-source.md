# Step 1: Create `web/src/utils/metadataSource.ts`

## Goal
Create a new utility file that parses and updates the `# metadata` section of a `.jianpu` source string. This is a new file — no existing files are modified.

## Background
The app uses a plain-text `.jianpu` format with three sections: `# metadata`, `# parts`, `# score`. A parallel utility already exists for the parts section at `web/src/utils/partSource.ts` — read it first as the model to follow.

The metadata section looks like:
```
# metadata
title = "Testing"
author = "Mozart"
row height = 24
max columns = 20
```

String fields are quoted (`title = "Foo"`), numeric fields are unquoted (`row height = 24`). The Rust parser strips quotes with `trim_matches('"')` so either works, but follow the existing convention.

## Fields (from `src/ast/parsed.rs` lines 151–160 and `syntax.md`)
| Key (exact string in file) | TypeScript field | Type |
|---|---|---|
| `title` | `title` | `string` (required) |
| `subtitle` | `subtitle` | `string \| null` |
| `author` | `author` | `string \| null` |
| `row height` | `rowHeight` | `number \| null` |
| `max columns` | `maxColumns` | `number \| null` |
| `label width` | `labelWidth` | `number \| null` |
| `note number width` | `noteNumberWidth` | `number \| null` |

## What to implement

File: `web/src/utils/metadataSource.ts`

### Types
```ts
export type MetadataKey =
  | 'title'
  | 'subtitle'
  | 'author'
  | 'row height'
  | 'max columns'
  | 'label width'
  | 'note number width'

export interface ParsedMetadataFields {
  title: string
  subtitle: string | null
  author: string | null
  rowHeight: number | null
  maxColumns: number | null
  labelWidth: number | null
  noteNumberWidth: number | null
}
```

### `parseMetadata(source: string): ParsedMetadataFields`
- Split source by `\n`
- Find the line where `line.trim() === '# metadata'`
- Iterate lines after it; stop when a line starts with `#` (next section)
- Skip blank lines
- For each `key = value` line: strip quotes from value with `replace(/^"(.*)"$/, '$1')`
- Map the 7 keys to the interface fields (numeric fields use `parseInt`)
- Return defaults (`title: ''`, all optionals `null`) if section not found

### `updateMetadataField(source: string, key: MetadataKey, value: string | null): string`
- Find the `# metadata` section in lines
- Find end of section (next line starting with `#`)
- Search for existing line whose key (left of `=`, trimmed) matches `key`
- If `value` is `null` or `''`: remove the matching line (if it exists)
- If value is non-null:
  - For string keys (`title`, `subtitle`, `author`): format as `key = "value"`
  - For numeric keys: format as `key = value`
  - If line exists: replace it; if not: insert before the end of the section
- Return modified source

## Files to read before implementing
- `web/src/utils/partSource.ts` — follow this structure and style
- `src/ast/parsed.rs` lines 151–160 — field names
- `syntax.md` lines 32–43 — field descriptions
