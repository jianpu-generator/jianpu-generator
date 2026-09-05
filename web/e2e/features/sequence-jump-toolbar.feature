Feature: Sequence jump toolbar

  Background:
    Given a source with a repeating sequence "A, B, B" is loaded

  Scenario: Sequence jump toolbar renders one button per resolved sequence entry, in playback order
    Then the sequence toolbar shows buttons "A, B, B" in order

  Scenario: Clicking the "A" entry enables playback from measure 1
    Then the play-from-current-measure button is hidden
    When I click the sequence toolbar button at index 0
    Then the play-from-current-measure button aria-label says "Play sequence from Measure 1"

  Scenario: Clicking the first "B" entry enables playback from measure 2
    When I click the sequence toolbar button at index 1
    Then the play-from-current-measure button aria-label says "Play sequence from Measure 2"

  Scenario: Clicking the second "B" occurrence highlights only that button, not the first "B"
    When I click the sequence toolbar button at index 2
    Then sequence toolbar button 2 is highlighted as active
    And sequence toolbar button 1 is not highlighted as active

  Scenario: Dragging from the "A" entry to the "B" entry selects the merged range and highlights both buttons
    When I drag from sequence toolbar button 0 to sequence toolbar button 1
    And I release the mouse button on the sequence toolbar
    Then the play-from-current-measure button aria-label says "Play sequence from Measure 1"
    And sequence toolbar button 0 is highlighted as active
    And sequence toolbar button 1 is highlighted as active

  Scenario: Touch-dragging from the "A" entry to the "B" entry selects the merged range and highlights both buttons
    When I touch-drag from sequence toolbar button 0 to sequence toolbar button 1
    Then the play-from-current-measure button aria-label says "Play sequence from Measure 1"
    And sequence toolbar button 0 is highlighted as active
    And sequence toolbar button 1 is highlighted as active
