Feature: Metronome during playback

  Scenario: The metronome toggle starts off and switches on and off
    Given a single-part four-note score is loaded with the disk cache workaround
    Then the metronome toggle is visible
    And the metronome toggle becomes enabled once the soundfont loads
    And the metronome toggle is off
    When I click the metronome toggle
    Then the metronome toggle is on
    When I click the metronome toggle
    Then the metronome toggle is off

  Scenario: Play All still plays to the end with the metronome on
    Given a single-part four-note score is loaded with the disk cache workaround
    And the Play All button becomes enabled once the soundfont loads
    When I click the metronome toggle
    And I click the Play All button
    Then the Play All button shows the playing state
    And the Play All button eventually stops showing the playing state
