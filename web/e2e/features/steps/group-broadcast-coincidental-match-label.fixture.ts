import { expect } from '@playwright/test'
import { focusEditor } from '../../fileSwitcherHelpers'

/**
 * Part declaration order used throughout this feature:
 *   Soprano 1 [S1] -> source_part_index 0
 *   Soprano 2 [S2] -> source_part_index 1 (follow[S1])
 *   Tenor [T]      -> source_part_index 2
 *
 * S1 and S2 merge into one row whenever their content is identical (which it
 * always is here, since both come from the same `[S]` broadcast); that
 * merged row's own `source_part_index` stays S1's (0), and the row-label
 * text this feature cares about is exposed as `PART_INDEX.Soprano`.
 */
export const PART_INDEX: Record<string, number> = {
  Soprano: 0,
  Tenor: 2,
}

export type MeasureSpec = {
  /** Note tokens the `[S]` group broadcast gives S1 and S2 for this measure. */
  sTokens: string
  /** Note tokens Tenor's own `[T]` line has for this measure. */
  tTokens: string
}

const SAME_TOKENS = '1 2 3 4'
const DIVERGENT_S_TOKENS = '5 6 7 1'
const DIVERGENT_T_TOKENS = '2 3 4 5'

export let state: {
  maxMeasuresPerSystem: number
  measures: MeasureSpec[]
} = { maxMeasuresPerSystem: 4, measures: [] }

export function resetState() {
  state = { maxMeasuresPerSystem: 4, measures: [] }
}

export function ensureMeasure(index: number): MeasureSpec {
  while (state.measures.length <= index) {
    state.measures.push({ sTokens: SAME_TOKENS, tTokens: SAME_TOKENS })
  }
  return state.measures[index]
}

export function setMatchingMeasure(index: number) {
  ensureMeasure(index).sTokens = SAME_TOKENS
  ensureMeasure(index).tTokens = SAME_TOKENS
}

export function setDivergentMeasure(index: number) {
  ensureMeasure(index).sTokens = DIVERGENT_S_TOKENS
  ensureMeasure(index).tTokens = DIVERGENT_T_TOKENS
}

function buildSource(): { source: string; firstMeasureLine: number } {
  const lines: string[] = [
    '# metadata',
    'title = "group broadcast coincidental match label test"',
    `max_measures_per_system = ${state.maxMeasuresPerSystem}`,
    '',
    '# parts',
    'Soprano 1 [S1] = notes',
    'Soprano 2 [S2] = follow[S1]',
    'Tenor [T] = notes',
    '',
    '# groups',
    'Soprano [S] = S1 S2',
    '',
    '# score',
  ]

  const firstMeasureLine = lines.length + 1

  const groups = state.measures.map(
    (measure) => `[S] ${measure.sTokens}\n[T] ${measure.tTokens}`,
  )

  lines.push(groups.join('\n\n'))
  return { source: lines.join('\n'), firstMeasureLine }
}

export async function loadFixture(
  page: import('@playwright/test').Page,
): Promise<number> {
  const { source, firstMeasureLine } = buildSource()
  await page.addInitScript((src) => {
    localStorage.setItem(
      'jianpu:files:v1',
      JSON.stringify({
        active: 'group-broadcast-coincidental-match-label-test.jianpu',
        userFiles: {
          'group-broadcast-coincidental-match-label-test.jianpu': src,
        },
        bin: {},
        fileIds: {
          'group-broadcast-coincidental-match-label-test.jianpu':
            'group-broadcast-coincidental-match-label-test-id-001',
        },
      }),
    )
  }, source)
  return firstMeasureLine
}

/** Same priming dance as `system-part-union-packing.fixture.ts`: jump the
 * cursor into the first measure's own source span so the SVG has settled
 * (measureSpans primed) before hit-testing. */
export async function primeMeasureSpans(
  page: import('@playwright/test').Page,
  line: number,
) {
  await focusEditor(page)
  await page.keyboard.press('Control+g')
  await page.keyboard.type(String(line))
  await page.keyboard.press('Enter')
  await expect(page.locator('button.play-measure-btn')).toHaveText(/Measure/, {
    timeout: 5_000,
  })
  await expect(
    page.locator('.preview-page [data-testid="measure-highlight"]').first(),
  ).toBeVisible({ timeout: 5_000 })
}

export function partLabelsFor(
  page: import('@playwright/test').Page,
  part: string,
) {
  return page.locator(
    `[data-tag="part-label"][data-part-index="${PART_INDEX[part]}"]`,
  )
}
