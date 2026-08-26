Feature: Playback cursor across a rest run created by hiding a part

  Scenario: Hiding the only part with notes in the leading measures merges them into one rest for everyone else, and the cursor must still land on the right note afterward
    Given the playback-cursor rest-collapse hidden-part test fixture is loaded
    When I hide the Harmony part, as seen in playback cursor rest collapse
    Then measures 0 and 1 render as a single multi-measure-rest for Melody
    When I click the Play All button, as seen in playback cursor rest collapse
    Then the merged multi-measure-rest shows the playback cursor highlight shortly after playback starts
    Then Melody's first note in measure 2 shows the playback cursor highlight
    And only Melody's first note in measure 2 shows the playback cursor highlight
