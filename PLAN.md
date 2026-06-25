# Plan: Rust/WASM `update_part_declaration` Function

## Context

The frontend currently rewrites part declarations in JS (`web/src/utils/partSource.ts`).
This plan replaces that JS logic with a Rust function exposed via WASM, so the source
rewriting is authoritative and shares parsing logic with the compiler.

---

## Goal

Expose a WASM function:

```ts
update_part_declaration(
  source: string,
  abbreviation: string,
  new_mode: string,        // "chord" | "notes" | "notes lyrics" | "follow[<target>]"
) => string
```

The WASM boundary uses a single `new_mode` string (JS can't pass Rust enums). The internal
Rust function uses a typed `PartMode` enum. `"follow[<target>]"` encodes the follow target
inline so no extra parameter is needed.

Returns the updated source with only the matched part's RHS replaced. Preserves:
- LHS (display name + abbreviation bracket)
- Spacing around `=`
- Existing `soundfont=` token on the line

---

## Files to change

### 1. New file: `src/source_edit.rs`

Define a `PartMode` enum:

```rust
pub enum PartMode {
    Chord,
    Notes,
    NotesLyrics,
    Follow { target: String },
}
```

Parsing from the WASM string input:

```rust
impl PartMode {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "chord"        => Some(Self::Chord),
            "notes"        => Some(Self::Notes),
            "notes lyrics" => Some(Self::NotesLyrics),
            _ if s.starts_with("follow[") && s.ends_with(']') => {
                let target = s["follow[".len()..s.len()-1].to_owned();
                Some(Self::Follow { target })
            }
            _ => None,
        }
    }

    pub fn to_rhs_str(&self) -> String {
        match self {
            Self::Chord        => "chord".to_owned(),
            Self::Notes        => "notes".to_owned(),
            Self::NotesLyrics  => "notes lyrics".to_owned(),
            Self::Follow { target } => format!("follow[{target}]"),
        }
    }
}
```

Main function:

```rust
pub fn update_part_declaration(
    source: &str,
    abbreviation: &str,
    new_mode: PartMode,
) -> Option<String>
```

Implementation steps:

1. Split `source` by `'\n'`.
2. Find the line index of `# parts` (trimmed equality).
3. Iterate subsequent lines; skip blank lines; stop at the next `# ` section header.
4. For each non-blank part line, find the first `=` to split LHS / RHS.
5. Match the part by abbreviation:
   - If LHS (trimmed) ends with `[something]`, `something` is the abbreviation.
   - Otherwise, trim the whole LHS and compare as abbreviation.
6. When matched, reconstruct the line:
   - Preserve everything up to and including the `=` character.
   - Extract the existing `soundfont=\w+` token from the old RHS (if present).
   - Build the new RHS from `new_mode.to_rhs_str()`.
   - Append `soundfont=…` suffix if it was present in old RHS.
   - New line: `"{lhs_with_eq} {new_rhs_mode}{soundfont_suffix}"`
7. Replace the line in the split vec; rejoin with `'\n'`; return `Some(result)`.
8. If no matching line found, return `None`.

### 2. `src/lib.rs`

Add `pub mod source_edit;`.

### 3. `crates/jianpu-wasm/src/lib.rs`

Add the exported function. Parse the string into `PartMode` at the WASM boundary:

```rust
#[wasm_bindgen]
pub fn update_part_declaration(
    source: &str,
    abbreviation: &str,
    new_mode: &str,
) -> String {
    let Some(mode) = jianpu::source_edit::PartMode::from_str(new_mode) else {
        return source.to_owned();
    };
    jianpu::source_edit::update_part_declaration(source, abbreviation, mode)
        .unwrap_or_else(|| source.to_owned())
}
```

The TypeScript signature is generated automatically by `wasm-bindgen` — do not edit `pkg/jianpu_wasm.d.ts` manually.

### 4. Frontend: `web/src/utils/partSource.ts`

Once the WASM function is available, replace the `updatePartDeclaration` JS
implementation with a thin wrapper that calls the WASM function:

```ts
import { update_part_declaration } from 'jianpu-wasm'

export function updatePartDeclaration(
  source: string,
  abbreviation: string,
  newMode: PartMode,
  newFollowTarget: string | null,
): string {
  const modeStr = newMode === 'follow'
    ? `follow[${newFollowTarget ?? ''}]`
    : newMode
  return update_part_declaration(source, abbreviation, modeStr)
}
```

The rest of `partSource.ts` (`parsePartDeclarations`, types) stays as-is.

---

## Tests

Add `src/source_edit/tests.rs` (separate file per project convention):

| Case | Input | Expected |
|------|-------|----------|
| Basic chord → notes | `"main = chord"` | `"main = notes"` |
| notes → notes lyrics | `"Melody [M] = notes"` | `"Melody [M] = notes lyrics"` |
| notes lyrics → follow | `"Alto [A] = notes lyrics"`, mode `Follow { target: "M" }` | `"Alto [A] = follow[M]"` |
| Preserves soundfont | `"Piano [P] = notes soundfont=piano"` → `Chord` | `"Piano [P] = chord soundfont=piano"` |
| No match → source unchanged | abbreviation not in file | returns original source |
| Multi-part file | only the target line changes | other lines identical |

---

## What is NOT needed

- Changes to the parser, AST, renderer, or MIDI layers.
- New syntax in `.jianpu` files.
- Changes to `syntax.md` or `ARCHITECTURE.md`.
