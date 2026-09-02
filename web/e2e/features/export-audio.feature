Feature: Export audio (WAV and MP3)

  Scenario: Export > WAV produces a playable inline audio player and Export > WAV (regenerate) replaces it
    Given the export test timeout is extended to 60 seconds
    And the single-part export source is loaded
    When I open the export menu and choose "WAV"
    Then the inline audio player is visible with a blob src
    And the inline audio player has decoded playable audio with positive duration
    When I open the export menu and choose "WAV (regenerate)"
    Then the inline audio player's blob src changes to a new blob url
    When I type " 5" at the end of the editor, as seen in export audio
    Then the inline audio player keeps showing the regenerated audio
    When I open the export menu
    Then the export menu shows a "WAV (regenerate)" item

  Scenario: Export > MP3 produces a playable inline audio player and Export > MP3 (regenerate) replaces it
    Given the export test timeout is extended to 60 seconds
    And the single-part export source is loaded
    When I open the export menu and choose "MP3"
    Then the inline audio player is visible with a blob src
    And the inline audio player has decoded playable audio with positive duration
    When I open the export menu and choose "MP3 (regenerate)"
    Then the inline audio player's blob src changes to a new blob url
    When I type " 5" at the end of the editor, as seen in export audio
    Then the inline audio player keeps showing the regenerated audio
    When I open the export menu
    Then the export menu shows a "MP3 (regenerate)" item

  Scenario: Switching between WAV and MP3 replaces the inline audio player rather than stacking both
    Given the export test timeout is extended to 60 seconds
    And the single-part export source is loaded
    When I open the export menu and choose "WAV"
    Then the inline audio player is visible with a blob src
    When I open the export menu and choose "MP3"
    Then only one inline audio player is visible
    And the inline audio player's blob src changes to a new blob url

  Scenario: Audio download button's rename modal filename includes only the enabled parts when a part is hidden
    Given the export test timeout is extended to 60 seconds
    And the multi-part export source is loaded
    And I hide the "H" part via its eye toggle
    When I open the export menu and choose "WAV"
    And I click the inline audio player's Download button
    Then the rename modal shows the input pre-filled with "test (Melody).wav"
    When I click the modal's Cancel button
    And I open the export menu and choose "MP3"
    And I click the inline audio player's Download button
    Then the rename modal shows the input pre-filled with "test (Melody).mp3"
    When I click the modal's Cancel button

  Scenario: Export Parts > WAV (ZIP) produces a non-empty downloaded zip for a multi-part score
    Given the export test timeout is extended to 60 seconds
    And the multi-part export source is loaded
    When I open the export menu and choose "WAV (ZIP)" and capture the download
    Then the downloaded zip file is larger than 1000 bytes
    And the downloaded file is named "test (WAV parts).zip"

  Scenario: A toast with a loading indicator is visible while WAV (ZIP) export is in progress, even after the export menu closes
    Given the export test timeout is extended to 60 seconds
    And the multi-part export source is loaded
    When I open the export menu and choose "WAV (ZIP)"
    Then the export menu closes immediately after choosing "WAV (ZIP)"
    And the export toast is visible with a spinner and says "Exporting WAV (ZIP)…"
    When I wait for the download to finish, as seen in export audio
    Then the export toast goes away once generation finishes
