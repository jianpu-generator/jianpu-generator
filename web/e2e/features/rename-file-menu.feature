Feature: Rename file via the "⋯" file actions menu prompt

  Background:
    Given a local-storage-backed file "original.jianpu" is seeded for the rename menu test
    And the app loads the seeded rename-menu test file

  Scenario: Renaming via the "⋯" menu prompt updates the active tab and trigger
    Given the rename dialog will be accepted with "renamed"
    When I open file actions and click the Rename menu item
    Then the file switcher trigger shows "renamed" after the rename prompt

  Scenario: Cancelling the rename prompt leaves the filename unchanged
    Given the rename dialog will be dismissed
    When I open file actions and click the Rename menu item
    Then the file switcher trigger shows "original" after the rename prompt
