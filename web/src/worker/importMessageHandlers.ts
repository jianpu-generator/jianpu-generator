import type { WorkerRequest, WorkerResponse } from './jianpu.worker'

export function handleImportFromFile(
  msg: Extract<WorkerRequest, { type: 'importFromFile' }>,
  extractSourceFromSvg: (svgBytes: Uint8Array) => string | undefined,
  extractSourceFromPdf: (pdfBytes: Uint8Array) => string | undefined,
): void {
  const bytes = new Uint8Array(msg.bytes)
  const source =
    msg.kind === 'svg'
      ? extractSourceFromSvg(bytes)
      : extractSourceFromPdf(bytes)

  if (source === undefined) {
    postMessage({ type: 'importErr', id: msg.id } satisfies WorkerResponse)
    return
  }

  postMessage({
    type: 'importOk',
    id: msg.id,
    source,
  } satisfies WorkerResponse)
}
