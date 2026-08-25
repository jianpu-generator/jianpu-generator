Feature: Cmd/Ctrl-clicking a part label selects the whole system

  # Cmd/Ctrl-click(-drag) on a part label elevates the selection from "this
  # one part's system" (a plain click/drag, see
  # `part-label-drag-system-boundary.feature`) to "every part in every system
  # the gesture touches" — see `PreviewDragState`'s 'part-label-system' doc
  # comment and `partLabelsInMarqueeAcrossSystems`.
  #
  # `max_measures_per_system = 1` forces each measure onto its own system, so
  # Melody's and Harmony's labels repeat twice, stacked vertically:
  #
  #   System 0 (measure 0): Melody "1 2", Harmony "5 6"
  #   System 1 (measure 1): Melody "3 4", Harmony "7 1'"

  Background:
    Given the cmd-click system fixture is loaded

  Scenario: Cmd/Ctrl-clicking one part label selects every part in that label's system
    When I Ctrl-click system 0's Melody part label
    Then 2 drag-selected notes belong to part index 0, as seen in part label cmd click selects whole system
    And 2 drag-selected notes belong to part index 1, as seen in part label cmd click selects whole system
    And 4 notes are drag-selected in total, as seen in part label cmd click selects whole system
    And system 0's Melody label's click-target rect is marked drag-active
    And system 0's Harmony label's click-target rect is marked drag-active

  Scenario: Cmd/Ctrl-dragging from one system's part label into another system selects every part across both systems
    When I Ctrl-drag from system 0's Melody label to system 1's Melody label
    Then 4 drag-selected notes belong to part index 0, as seen in part label cmd click selects whole system
    And 4 drag-selected notes belong to part index 1, as seen in part label cmd click selects whole system
    And 8 notes are drag-selected in total, as seen in part label cmd click selects whole system
    And system 1's Melody label's click-target rect is marked drag-active
    And system 1's Harmony label's click-target rect is marked drag-active
