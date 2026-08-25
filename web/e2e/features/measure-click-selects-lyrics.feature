Feature: Measure click selects lyrics

  Background:
    Given the measure-click lyric test fixture is loaded

  Scenario: Clicking a measure selects just the note under the pointer, with no lyric selection
    When I plain-click on the note row near the top of measure 1
    Then 1 note is drag-selected
    And 0 lyrics are drag-selected

  Scenario: Cmd/Ctrl-clicking a measure also selects the lyric syllables in that measure
    When I Cmd/Ctrl-click on the note row near the top of measure 1
    Then 2 notes are drag-selected, as seen in measure click selects lyrics
    And 2 lyrics are drag-selected
    And lyrics with note ids 4 and 5 are drag-selected

  Scenario: Cmd/Ctrl-dragging across measures selects every lyric syllable in the range
    When I Cmd/Ctrl-drag from measure 0's note row to measure 1's note row
    Then 6 notes are drag-selected, as seen in measure click selects lyrics
    And 6 lyrics are drag-selected
