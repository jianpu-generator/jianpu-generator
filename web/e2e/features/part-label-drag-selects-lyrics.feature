Feature: Dragging across part labels selects lyrics too

  # Regression test: drag-selecting vertically across a part label is supposed
  # to select every note/rest *and* every lyric syllable that part sounds
  # across the whole system the label sits in — mirroring how 'measure' mode's
  # click/drag resolves both `noteCellsInMeasureRange` and
  # `lyricCellsInMeasureRange` together (see
  # `measure-click-selects-lyrics.spec.ts`). A *plain click* (no drag) is
  # deliberately narrower and selects only the notes row, not the lyric row —
  # see `part-label-click-selects-notes.feature`'s
  # "plain click does not also select the lyric row" scenario.
  #
  # `usePreviewDragSelection.ts`'s `'part-label'` mode used to resolve only
  # `noteCellsForPartLabels` and never a lyric-side counterpart, so a
  # part-label drag silently skipped every lyric row underneath the swept
  # parts. `lyricCellsForPartLabels` (in `previewLabelSelection.ts`) is the fix.

  Scenario: Dragging vertically across part labels selects both parts notes and the lyrics under them
    Given the part-label lyric-drag fixture is loaded
    When I drag from the Melody part label to the Harmony part label, as seen in part label drag selects lyrics
    Then 8 notes are drag-selected in total, as seen in part label drag selects lyrics
    And 4 lyrics are drag-selected in total, as seen in part label drag selects lyrics
