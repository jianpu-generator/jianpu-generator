Feature: Measure autoscroll

  Scenario: Auto-scrolls the preview to the highlighted measure when the caret moves off-screen
    Given a 60-measure autoscroll test fixture is loaded
    When I jump the caret to the first measure's line
    Then the preview has not scrolled away from the top
    When I jump the caret to the last measure's line
    Then the preview scrolls so the last measure's highlight is visible in the viewport
