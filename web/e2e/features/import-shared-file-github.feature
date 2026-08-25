Feature: Import a shared score via GitHub storage backend

  Scenario: Importing a shared score persists via the GitHub storage backend
    Given the GitHub Contents API is mocked with no files for a shared import
    And GitHub auth is seeded for the mocked owner
    When I navigate to the share URL for "shared-test.jianpu"
    Then the shared-preview banner shows "shared-test.jianpu"
    When I click the "Import to my scores" button
    Then the active tab becomes the imported file "shared-test.jianpu"
    And the shared-preview banner is gone
    And the import-create PUT for "scores/shared-test.jianpu" carries no sha
    When I reload the page after importing
    Then the file list shows "shared-test.jianpu" after reload
