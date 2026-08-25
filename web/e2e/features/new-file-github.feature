Feature: New file via GitHub storage backend

  Scenario: Creating a new file persists via the GitHub storage backend
    Given the GitHub repo is seeded with a file named "scores/original.jianpu" for new-file creation
    And GitHub auth is seeded for the mocked owner
    When the app loads the GitHub-backed file list for new-file creation
    And I click the "New" button to create a new file
    Then the new-file button shows a pending spinner
    And the active tab becomes the new file "untitled.jianpu"
    And the new-file button disappears once the create resolves
    And the new-file create PUT for "scores/untitled.jianpu" carries no sha
    When I reload the page after creating
    Then the file list still shows the new file "untitled.jianpu" after reload
    And the file list still shows "original.jianpu" exactly once
