Feature: Note-lyric cross drag select

  Background:
    Given the note-lyric cross drag test fixture is loaded and both rows have rendered

  Scenario: Dragging a marquee that starts on a note down across lyric syllables also selects those syllables
    When I drag a marquee from note 0's click target down and across to lyric syllable 2
    Then 3 notes are drag-selected by the cross-row marquee
    And 3 lyric syllables are drag-selected by the cross-row marquee

  Scenario: Dragging a marquee that starts on a lyric syllable up across notes also selects those notes
    When I drag a marquee from lyric syllable 0 up and across to note 2's click target
    Then 3 lyric syllables are drag-selected by the cross-row marquee
    And 3 notes are drag-selected by the cross-row marquee
