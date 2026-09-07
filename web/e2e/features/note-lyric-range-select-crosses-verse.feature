Feature: Note-lyric cross-row range selection crosses a verse boundary

  Scenario: Clicking a note then a syllable in a later verse of the same part selects every verse in between too
    Given the note-lyric cross-verse range-selection fixture is loaded and both verses have rendered
    When I click-and-click select Melody's note 0 then verse 1's syllable 2
    Then Melody's notes 0 through 2 are range-selected
    And verse 0's syllables 0 through 2 are range-selected
    And verse 1's syllables 0 through 2 are range-selected
    And verse 0's syllable 3 is not range-selected
    And verse 1's syllable 3 is not range-selected
