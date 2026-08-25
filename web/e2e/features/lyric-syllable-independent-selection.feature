Feature: Lyric syllable independent selection

  Background:
    Given the multi-verse lyric independence fixture is loaded and both verses have rendered

  Scenario: Clicking one syllable selects only that syllable, no notes
    When I click syllable 1 of verse 0 without dragging
    Then only syllable 1 of verse 0 is drag-selected
    And no note is drag-selected by the syllable-level interaction

  Scenario: Dragging across syllables selects exactly those cells and the matching editor text
    When I drag from syllable 0 to syllable 2 of verse 0
    Then 3 lyric syllables in total are drag-selected
    And no note is drag-selected by the syllable-level interaction
    And the Monaco selection text is "do re mi"

  Scenario: Clicking a note directly selects just that note, no lyrics
    When I click near the top of note 1's click target without dragging
    Then exactly 1 note is drag-selected via the note click target
    And no lyric syllable is drag-selected

  Scenario: Cmd/Ctrl-clicking a note selects the whole measure, notes and every verse of lyrics alike
    When I Ctrl-click near the top of note 1's click target
    Then exactly 4 notes are drag-selected via the note click target
    And 8 lyric syllables in total are drag-selected

  Scenario: Verses select independently and each syllable maps to its own verse line
    When I click syllable 1 of verse 1 without dragging
    Then syllable 1 of verse 1 is drag-selected but syllable 1 of verse 0 is not
    And the Monaco selection text is "dos"
