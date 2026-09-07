Feature: Note-lyric cross-row range selection crosses a part boundary within a single shared measure

  Scenario: Clicking a note in one part then a lyric syllable further along in another part, both in the same measure, selects only up to that position in each part
    Given the cross-part-within-measure note-lyric range-selection fixture is loaded and both rows have rendered
    When I click-and-click select Melody's note 0 then Harmony's lyric syllable 1
    Then Melody's and Harmony's notes at position 0 and 1 are range-selected
    And Melody's and Harmony's lyric syllables at position 0 and 1 are range-selected
    And Melody's and Harmony's notes at position 2 are not range-selected
    And Melody's and Harmony's lyric syllables at position 2 are not range-selected
