Feature: Tab switch force-save via GitHub storage backend

  Scenario: Switching the active file tab force-flushes a pending debounced GitHub save
    Given the GitHub repo is seeded with files "scores/a.jianpu" and "scores/b.jianpu" for a tab-switch save
    And GitHub auth is seeded for the mocked owner
    And a fake clock is installed to prevent an autosave race with the tab switch
    When the app loads the GitHub-backed file list for a tab-switch save
    And I select the "a.jianpu" tab to test the tab switch
    And I append " 5" to the editor to trigger a tab-switch save
    Then no PUT has been sent yet before the tab switch
    And the tab-switch status badge shows "Unsaved"
    When I switch to the "b.jianpu" tab from the file list
    Then the tab-switch PUT lands for "scores/a.jianpu" containing "1 2 3 4 5"
    When I switch to the "a.jianpu" tab from the file list
    And I reload the page after the tab-switch save
    Then the tab-switch-saved file list still shows "a.jianpu" after reload
    And the reloaded editor still contains the tab-switch-saved edit "1 2 3 4 5"
