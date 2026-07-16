/** Query param used to remember the active file across page reloads. */
const FILE_PARAM = 'file'

export function readFileNameFromUrl(): string | null {
  return new URLSearchParams(window.location.search).get(FILE_PARAM)
}

/** Reflects `name` into the `?file=` query param via `replaceState`, so
 * switching files doesn't grow browser history but a reload lands back on
 * the same file. No-ops when the param is already up to date. */
export function writeFileNameToUrl(name: string): void {
  const params = new URLSearchParams(window.location.search)
  if (params.get(FILE_PARAM) === name) return
  params.set(FILE_PARAM, name)
  const url = `${window.location.pathname}?${params.toString()}${window.location.hash}`
  window.history.replaceState(null, '', url)
}
