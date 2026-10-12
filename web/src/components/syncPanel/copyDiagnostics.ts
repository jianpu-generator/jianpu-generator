import type { LaneView } from './syncPanelTypes'

/** Plain-text report of lane state. Never includes file contents. */
export function buildDiagnostics(lanes: LaneView[]): string {
  return lanes
    .map((lane) =>
      [
        `Lane: ${lane.fileName} (${lane.fileId})`,
        `Status: ${lane.status}`,
        ...(lane.retryInSeconds === undefined
          ? []
          : [`Retry in: ${lane.retryInSeconds}s`]),
        'Messages:',
        ...lane.messages.map(
          (m) => `  - ${m.kindLabel}, ${m.sizeLabel}, ${m.createdLabel}`,
        ),
        'Attempts:',
        ...lane.attempts.map((a) => `  - ${a.atLabel}: ${a.outcomeLabel}`),
      ].join('\n'),
    )
    .join('\n\n')
}

/** Writes text to the clipboard; if that rejects, selects the text for manual copy. */
export async function copyDiagnostics(
  text: string,
): Promise<'copied' | 'selected'> {
  try {
    await navigator.clipboard.writeText(text)
    return 'copied'
  } catch {
    const area = document.createElement('textarea')
    area.value = text
    document.body.appendChild(area)
    area.select()
    return 'selected'
  }
}
