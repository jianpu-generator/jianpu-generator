// `verovio` ships no type declarations; these cover only what
// `staffMessageHandlers.ts` calls.

declare module 'verovio/wasm' {
  export default function createVerovioModule(): Promise<unknown>
}

declare module 'verovio/esm' {
  export class VerovioToolkit {
    constructor(module: unknown)
    setOptions(options: Record<string, unknown>): void
    loadData(data: string): boolean
    getPageCount(): number
    renderToSVG(pageNo?: number): string
  }
}
