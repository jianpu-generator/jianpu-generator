Feature: Export MIDI

  Scenario: Export > MIDI produces a non-empty downloaded file
    Given the single-part MIDI export source is loaded
    When I export "MIDI" and capture the download
    Then the downloaded MIDI file is larger than 0 bytes
    And the downloaded file is named "test.mid", as seen in export midi

  Scenario: Export Parts > MIDI (ZIP) produces a non-empty downloaded zip for a multi-part score
    Given the multi-part MIDI export source is loaded
    When I export "MIDI (ZIP)" and capture the download
    Then the downloaded MIDI file is larger than 0 bytes
    And the downloaded file is named "test (MIDI parts).zip", as seen in export midi

  Scenario: Export > MIDI filename includes only the enabled parts when a part is hidden
    Given the multi-part MIDI export source is loaded
    And I hide the "H" part via its eye toggle, as seen in export midi
    When I export "MIDI" and capture the download
    Then the downloaded file is named "test (Melody).mid", as seen in export midi
