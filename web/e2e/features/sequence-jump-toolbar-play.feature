Feature: Sequence jump toolbar play

  Background:
    Given a sequence-test source "A, B, B" is loaded with the disk cache workaround

  Scenario: Clicking play on a selected sequence entry starts and finishes playback
    When I select the first sequence toolbar entry
    Then the sequence playback button aria-label says "Play sequence from Measure 1"
    And the play-from-current-measure button becomes enabled once the soundfont loads
    When I click the play-from-current-measure button
    Then the play-from-current-measure button shows the playing state
    And the play-from-current-measure button eventually stops showing the playing state
