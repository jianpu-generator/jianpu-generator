Feature: Export MP3

  # Plain "MP3" export shows an inline audio player instead of downloading
  # directly — see export-audio.feature. "MP3 (ZIP)" (parts export) has no
  # inline-player equivalent and stays a direct download.
  Scenario: Export Parts > MP3 (ZIP) produces a non-empty downloaded zip for a multi-part score
    Given the multi-part MP3 export source is loaded
    When I export "MP3 (ZIP)" and capture the download, as seen in export mp3
    Then the downloaded MP3 file is larger than 0 bytes
    And the downloaded file is named "test (MP3 parts).zip", as seen in export mp3

  Scenario: A toast with a loading indicator is visible while MP3 export is in progress, even after the export menu closes
    Given the export test timeout is extended to 60 seconds
    And the single-part export source is loaded
    When I open the export menu and choose "MP3"
    Then the export menu closes immediately after choosing "MP3"
    And the export toast is visible with a spinner and says "Generating MP3…"
    And the inline audio player eventually becomes visible
    And the export toast goes away once generation finishes

  Scenario: A toast with a loading indicator is visible while MP3 (ZIP) export is in progress, even after the export menu closes
    Given the export test timeout is extended to 60 seconds
    And the multi-part MP3 export source is loaded
    When I open the export menu and choose "MP3 (ZIP)"
    Then the export menu closes immediately after choosing "MP3 (ZIP)"
    And the export toast is visible with a spinner and says "Exporting MP3 (ZIP)…"
    When I wait for the download to finish, as seen in export mp3
    Then the export toast goes away once generation finishes
