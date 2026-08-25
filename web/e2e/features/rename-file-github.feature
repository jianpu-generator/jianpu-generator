Feature: Rename file via GitHub storage backend

  Scenario: Renaming a file persists via the GitHub storage backend
    Given the GitHub repo is seeded with a file named "scores/original.jianpu" for renaming
    And GitHub auth is seeded for the mocked owner
    When the app loads the GitHub-backed file list
    And I select the "original.jianpu" tab from the file list
    And I rename the active tab to "renamed.jianpu"
    Then the active tab shows a pending rename spinner
    And the active tab is renamed to "renamed.jianpu"
    And the renamed file's preview is visible
    When I reload the page after renaming
    Then the file list still shows "renamed.jianpu" after reload
    And the file list no longer shows "original.jianpu" after reload
