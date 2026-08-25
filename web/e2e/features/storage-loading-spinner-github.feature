Feature: Storage settings modal loading spinner when switching to GitHub

  Scenario: Switching to GitHub repository storage shows a loading spinner while files load
    Given the local storage backend is active with GitHub auth already seeded
    And the mocked GitHub user exists for the mocked owner
    And the GitHub Contents API is mocked with a seeded file for the storage-switch spinner
    And the GitHub directory listing GET is delayed by 1 second when switching backend
    When the app loads on the local backend with the editor ready
    And I open the storage settings modal to switch backend
    And I select the "GitHub repository" storage radio option
    Then the github loading spinner is visible
    And the github loading spinner disappears once loading finishes
    And the storage settings modal shows connected as the mocked owner
