/**
 * Part declaration order used throughout this feature:
 *   Soprano 1 [S1] -> source_part_index 0
 *   Soprano 2 [S2] -> source_part_index 1
 *   Alto 1   [A1]  -> source_part_index 2
 *   Alto 2   [A2]  -> source_part_index 3
 *   Tenor    [T]   -> source_part_index 4
 *
 * All five parts broadcast identical notes for the one measure in this
 * fixture, so they all fold into a single row. That merged row's own
 * `source_part_index` stays S1's (0), the first member in declaration order.
 */
export const MERGED_ROW_PART_INDEX = 0

const SOURCE = [
  '# metadata',
  'title = "group broadcast multiple groups merged row label test"',
  '',
  '# parts',
  'Soprano 1 [S1] = notes',
  'Soprano 2 [S2] = notes',
  'Alto 1 [A1] = notes',
  'Alto 2 [A2] = notes',
  'Tenor [T] = notes',
  '',
  '# groups',
  'Soprano [S] = S1 S2',
  'Alto [A] = A1 A2',
  '',
  '# score',
  '[S] 6 6 6 6',
  '[A] 6 6 6 6',
  '[T] 6 6 6 6',
].join('\n')

export async function loadFixture(
  page: import('@playwright/test').Page,
): Promise<void> {
  await page.addInitScript((src) => {
    localStorage.setItem(
      'jianpu:files:v1',
      JSON.stringify({
        active: 'group-broadcast-multiple-groups-merged-row-label-test.jianpu',
        userFiles: {
          'group-broadcast-multiple-groups-merged-row-label-test.jianpu': src,
        },
        bin: {},
        fileIds: {
          'group-broadcast-multiple-groups-merged-row-label-test.jianpu':
            'group-broadcast-multiple-groups-merged-row-label-test-id-001',
        },
      }),
    )
  }, SOURCE)
}
