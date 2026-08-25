Feature: Note click target excludes lyric row

  Scenario: A note-with-lyric note click target stops above the lyric row, not covering it
    Given the note click target lyric row test fixture is loaded and lyrics have rendered
    Then note 0's click-target rect ends at or above its lyric row's top edge
