Feature: Bar-line drag selects measures

  Background:
    Given the bar-line-drag test fixture is loaded

  Scenario: Hovering the bar line between two measures shows a drag cursor
    When I hover the bar line between measure 0 and measure 1
    Then the bar-line drag handle shows a col-resize cursor

  Scenario: Cmd/Ctrl-dragging from a bar line into a further measure selects every note in the full range
    When I Cmd/Ctrl-drag from the bar line before measure 1 into measure 2's interior
    Then 4 notes are drag-selected, as seen in bar line drag selects measures
    And the play-measure button reads Selection

  Scenario: Plain-dragging from a bar line into a further measure selects every note in the full range
    When I plain-drag from the bar line before measure 1 into measure 2's interior
    Then 4 notes are drag-selected, as seen in bar line drag selects measures
    And the play-measure button reads Selection
