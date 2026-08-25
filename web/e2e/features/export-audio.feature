Feature: Export audio (WAV)

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

  Scenario: Audio download link filename includes only the enabled parts when a part is hidden
    Given the export test timeout is extended to 60 seconds
    And the multi-part export source is loaded
    And I hide the "H" part via its eye toggle
    When I open the export menu and choose "WAV"
    Then the audio download link is visible with download name "test (Melody).wav"

  Scenario: Export Parts > WAV (ZIP) produces a non-empty downloaded zip for a multi-part score
    Given the export test timeout is extended to 60 seconds
    And the multi-part export source is loaded
    When I open the export menu and choose "WAV (ZIP)" and capture the download
    Then the downloaded zip file is larger than 1000 bytes
    And the downloaded file is named "test (WAV parts).zip"
