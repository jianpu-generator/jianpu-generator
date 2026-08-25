Feature: Mobile workspace layout

  Background:
    Given a seeded score and a mobile viewport

  Scenario: Shows only the preview by default below the mobile breakpoint
    Then the editor pane is collapsed and the preview pane fills the mobile viewport

  Scenario: Header scrolls horizontally instead of wrapping to a new row
    Then the app header overflows horizontally instead of wrapping to a second row

  Scenario: Toggling swaps to the editor and hides the preview
    When I click the pane-divider toggle
    Then the editor pane is shown and the preview pane is collapsed
    When I click the pane-divider toggle again
    Then the editor pane is collapsed and the preview pane is shown again

  Scenario: Export dropdown items are reachable on a mobile viewport
    When I open the Export dropdown menu
    Then every export menu item is within the mobile viewport
