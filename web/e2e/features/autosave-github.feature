Feature: Autosave via GitHub storage backend

  Scenario: Editing a file schedules a debounced autosave to the GitHub storage backend
    Given the GitHub repo is seeded with a file named "scores/auto.jianpu" for autosave
    And GitHub auth is seeded for the mocked owner
    And a fake clock is installed before navigating to test autosave
    When the app loads the GitHub-backed file list for autosave
    And I select the "auto" tab to test autosave
    And I append " 5" to the editor to trigger an autosave
    Then no autosave PUT has been sent yet for the debounced edit
    And the autosave status badge shows "Unsaved"
    When I fast-forward the clock past the autosave debounce interval to trigger it
    Then the autosave PUT lands for "scores/auto.jianpu" containing "1 2 3 4 5"
    And the autosave status badge shows "Saved"
    When I reload the page after the autosave
    Then the autosaved file list still shows "auto" after reload
    And the reloaded editor still contains the autosaved edit "1 2 3 4 5"
