Feature: Dragging a part label clamps to its own system

  # Regression test: a part-label drag (see `Preview.tsx`'s
  # `getPartLabelAtPoint` in `previewLabelSelection.ts` and
  # `partLabelsInMarquee` in `previewLabelDragHighlights.ts`)
  # is meant to be a vertical shortcut for selecting more *parts within the
  # same system* the drag started in — every part label's click target only
  # ever covers its own system's measure range (`measureIndexStart`/
  # `measureIndexEnd`, one `PartLabelClickTarget` per system, see
  # `grid_layout::click_targets::compute_all_part_label_click_targets`).
  #
  # The marquee test in `partLabelsInMarquee` currently has no awareness of
  # system boundaries though: it just intersects the drag rectangle against
  # every part-label rect in the whole document, so a drag that happens to
  # travel far enough vertically to reach a *different* system's label row
  # picks that label up too — silently splicing together notes from two
  # unrelated systems (undefined/nonsensical as a "part selection"). The drag
  # must clamp to the system it started in instead.
  #
  # `max_measures_per_system = 1` forces each measure onto its own system, so
  # Melody's and Harmony's labels repeat twice, stacked vertically:
  #
  #   System 0 (measure 0): Melody "1 2", Harmony "5 6"
  #   System 1 (measure 1): Melody "3 4", Harmony "7 1'"

  Scenario: Dragging a part label past its own system does not select notes from the next system
    Given the part-label system-boundary fixture is loaded
    When I drag straight down from system 0's Melody label to system 1's Melody label
    Then 2 drag-selected notes belong to part index 0, as seen in part label drag system boundary
    And 2 drag-selected notes belong to part index 1, as seen in part label drag system boundary
    And 4 notes are drag-selected in total, as seen in part label drag system boundary
    And system 0's Melody label's click-target rect is marked drag-active, as seen in part label drag system boundary
    And system 0's Harmony label's click-target rect is marked drag-active, as seen in part label drag system boundary
    And system 1's Melody label's click-target rect is not marked drag-active
