Feature: Duplicate file via GitHub storage backend

  Scenario: Duplicating a file persists via the GitHub storage backend
    Given the GitHub repo is seeded with a file named "scores/original.jianpu" for duplication
    And GitHub auth is seeded for the mocked owner
    When the app loads the GitHub-backed file list for duplication
    And I select the "original.jianpu" tab to duplicate it
    And I capture the active editor's source content
    And I click the "Duplicate" button to duplicate the file
    Then the duplicate button shows a pending spinner
    And the active tab becomes the duplicate "original 2.jianpu"
    And the duplicate button spinner clears and its label resets to "Duplicate"
    And the duplicated editor content matches the captured source content
    And the duplicate-create PUT for "scores/original 2.jianpu" carries no sha
    When I reload the page after duplicating
    Then the file list still shows both "original.jianpu" and "original 2.jianpu" tabs
    And the reloaded duplicate's editor content matches the captured source content
