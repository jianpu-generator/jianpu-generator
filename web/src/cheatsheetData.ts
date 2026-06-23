import rawData from '../../cheatsheet-examples.toml'

export type CheatsheetExample =
  | { kind: 'note'; description: string; syntax: string }
  | { kind: 'chord'; description: string; syntax: string }
  | { kind: 'line'; description: string; syntax: string; notes_line: string }
  | { kind: 'score'; description: string; syntax: string; source: string; show_decorations?: boolean }
  | { kind: 'directives'; description: string; syntax: string; source: string }

export interface CheatsheetSection {
  title: string
  examples: CheatsheetExample[]
}

export const cheatsheetSections: CheatsheetSection[] = (
  rawData as { section: CheatsheetSection[] }
).section
