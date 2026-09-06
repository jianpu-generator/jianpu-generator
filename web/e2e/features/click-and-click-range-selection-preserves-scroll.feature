Feature: Committing a click-and-click range selection does not re-scroll the preview

  Scenario: Scrolling the second note into view and clicking it keeps the preview at that scroll position
    Given the scroll-preserving range-selection fixture is loaded and note click targets have rendered
    When I click-and-click select the first note then scroll a far-away note into view and click it
    Then the preview's scroll position is unchanged from right after the manual scroll
