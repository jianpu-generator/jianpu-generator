Feature: Play All audio playback

  Scenario: Clicking Play All starts and finishes playback of the whole score
    Given a single-part four-note score is loaded with the disk cache workaround
    Then the Play All button is visible
    And the Play All button becomes enabled once the soundfont loads
    When I click the Play All button
    Then the Play All button shows the playing state
    And the Play All button eventually stops showing the playing state
