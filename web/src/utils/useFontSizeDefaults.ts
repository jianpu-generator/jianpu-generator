import { useEffect, useState } from 'react'
import {
  defaultAuthorFontSize,
  defaultLyricsFontSize,
  defaultPageNumberFontSize,
  defaultPartLegendFontSize,
  defaultSubtitleFontSize,
  defaultTitleFontSize,
} from './metadataDefaults'

export interface FontSizeDefaults {
  lyricsFontSizeDefault: number | null
  titleFontSizeDefault: number | null
  subtitleFontSizeDefault: number | null
  authorFontSizeDefault: number | null
  partLegendFontSizeDefault: number | null
  pageNumberFontSizeDefault: number | null
}

/** Resolves the `row_height`-derived font-size defaults (`lyrics_font_size`,
 * `title_font_size`, etc.) for the Edit Metadata modal. Re-resolves whenever
 * `rowHeight` changes; returns all-`null` until `rowHeight` is known. */
export function useFontSizeDefaults(
  rowHeight: number | null,
): FontSizeDefaults {
  const [defaults, setDefaults] = useState<FontSizeDefaults>({
    lyricsFontSizeDefault: null,
    titleFontSizeDefault: null,
    subtitleFontSizeDefault: null,
    authorFontSizeDefault: null,
    partLegendFontSizeDefault: null,
    pageNumberFontSizeDefault: null,
  })

  useEffect(() => {
    if (rowHeight === null) return
    Promise.all([
      defaultLyricsFontSize(rowHeight),
      defaultTitleFontSize(rowHeight),
      defaultSubtitleFontSize(rowHeight),
      defaultAuthorFontSize(rowHeight),
      defaultPartLegendFontSize(rowHeight),
      defaultPageNumberFontSize(rowHeight),
    ]).then(
      ([
        lyricsFontSizeDefault,
        titleFontSizeDefault,
        subtitleFontSizeDefault,
        authorFontSizeDefault,
        partLegendFontSizeDefault,
        pageNumberFontSizeDefault,
      ]) =>
        setDefaults({
          lyricsFontSizeDefault,
          titleFontSizeDefault,
          subtitleFontSizeDefault,
          authorFontSizeDefault,
          partLegendFontSizeDefault,
          pageNumberFontSizeDefault,
        }),
    )
  }, [rowHeight])

  return defaults
}
