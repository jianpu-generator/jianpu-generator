Feature: Note range selection crosses a measure boundary where rests shift note positions

  Scenario: Clicking a note at the very start of one measure then a later-positioned note in a following measure selects the whole first measure and only up to that position in the following measure
    Given the cross-measure-rest range-selection fixture is loaded and note click targets have rendered
    When I click-and-click select the note at index 0 then the note at index 13
    Then the notes at indexes "0,1,2,3,4,5,6,7,8,9,12,13" are all range-selected
    And the notes at indexes "10,11,14,15" are not range-selected
