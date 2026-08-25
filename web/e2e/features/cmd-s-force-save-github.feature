Feature: Cmd/Ctrl+S force-save via GitHub storage backend

  Scenario: Cmd/Ctrl+S force-flushes a pending debounced GitHub save immediately
    Given the GitHub repo is seeded with a file named "scores/save.jianpu" for a force save
    And GitHub auth is seeded for the mocked owner
    And a fake clock is installed to prevent an autosave race with force save
    When the app loads the GitHub-backed file list for a force save
    And I select the "save" tab to test the force save
    And I append " 5" to the editor to trigger a force save
    Then no PUT has been sent yet before the force save
    And the force-save status badge shows "Unsaved"
    When I press Cmd/Ctrl+S
    Then the force-save PUT lands for "scores/save.jianpu" containing "1 2 3 4 5"
    When I reload the page after the force save
    Then the force-saved file list still shows "save" after reload
    And the reloaded editor still contains the force-saved edit "1 2 3 4 5"
