Feature: Lyric drag-select highlight

  Background:
    Given the lyric drag test fixture is loaded and the first measure has rendered

  Scenario: Dragging a marquee across lyric syllables selects the syllables, not their underlying notes
    When I drag a marquee from lyric syllable 0 to lyric syllable 2
    Then lyric syllables 0, 1 and 2 are drag-selected
    And no note is drag-selected

  Scenario: Clicking a single lyric syllable selects only that syllable, not the note
    When I click lyric syllable 1 without dragging
    Then only lyric syllable 1 is drag-selected
    And no note is drag-selected
