Feature: Delete file via GitHub storage backend

  Scenario: Deleting a file persists via the GitHub storage backend
    Given the GitHub repo is seeded with a file named "scores/original.jianpu" for deletion
    And GitHub auth is seeded for the mocked owner
    When the app loads the GitHub-backed file list for deletion
    And I select the "original.jianpu" tab to delete it
    And I delete the active file via the file actions menu
    Then the delete button shows a pending spinner
    And the delete button disappears once the delete resolves
    And the file actions bin trigger shows "Bin (1)" after deleting
    And the bin lists the deleted file "original.jianpu"
    And the active tab falls back to "01-pitches.jianpu"
    When I reload the page after deleting
    Then the file list no longer shows the deleted file "original.jianpu" after reload
    And the file actions bin trigger shows "Bin (1)" after reload
    And the bin lists "original.jianpu" after reload
