Feature: Export WAV progress toast

  Scenario: A toast with a loading indicator is visible while WAV export is in progress, even after the export menu closes
    Given the export test timeout is extended to 60 seconds, as seen in export wav toast
    And the single-part WAV toast export source is loaded
    When I open the export menu and choose "WAV", as seen in export wav toast
    Then the export menu closes immediately after choosing "WAV"
    And the export toast is visible with a spinner and says "Generating WAV…"
    And the inline audio player eventually becomes visible
    And the export toast goes away once generation finishes
