Feature: Note range selection crosses a part boundary within a single shared measure

  Scenario: Clicking a note in one part then a note further along in another part, both in the same measure, selects only up to that position in each part
    Given the cross-part-within-measure range-selection fixture is loaded and note click targets have rendered
    When I click-and-click select the note at index 0 then the note at index 4
    Then the notes at index 0, 1, 3 and 4 are all range-selected
    And the notes at index 2 and 5 are not range-selected
