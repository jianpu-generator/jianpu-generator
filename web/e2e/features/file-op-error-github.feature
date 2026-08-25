Feature: File-op failure handling for the GitHub storage backend

  Scenario: A failed create shows the error modal, resets pending state, and a retry succeeds
    Given the GitHub repo is seeded with a file named "scores/original.jianpu" for a failing create
    And the first create PUT will fail with a 500 error
    And GitHub auth is seeded for the mocked owner
    When the app loads the GitHub-backed file list for a failing create
    And I remember the currently active tab name
    And I click the "New" button to create a file that will fail
    Then the new-file button shows a pending spinner before the create fails
    And the error modal is shown with message "Could not create file" containing "Internal Server Error"
    When I close the error modal
    Then the new-file button spinner clears and its label resets to "New"
    And no "untitled" tab exists
    And the active tab is unchanged from before the failed create
    When I retry the "New" button in the file actions menu
    Then the retried create succeeds and the active tab becomes "untitled"
    And the new-file button has no pending spinner
