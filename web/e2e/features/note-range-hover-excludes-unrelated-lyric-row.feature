Feature: Note-range hover preview excludes an unrelated lyric row

  Scenario: Hovering a note in another part during a click-and-click gesture does not highlight a lyric row belonging only to that part
    Given the note-hover-lyric-exclusion fixture is loaded and both parts have rendered
    When I click Melody's first note then hover over Harmony's first note
    Then Harmony's lyric syllables are not range-selected
