Feature: Beforeunload warning for pending GitHub saves

  Scenario: Closing the tab warns while a GitHub save is still pending
    Given the GitHub repo is seeded with a file named "scores/pending.jianpu" for a pending save
    And I open and edit "pending.jianpu" with a fake clock installed
    Then the beforeunload status badge shows "Unsaved"
    When I close the page without letting the save land
    Then a beforeunload dialog is shown

  Scenario: Closing the tab does not warn once the GitHub save has landed
    Given the GitHub repo is seeded with a file named "scores/saved.jianpu" for a landed save
    And I open and edit "saved.jianpu" with a fake clock installed
    When I fast-forward the clock so the beforeunload save lands
    Then the beforeunload-tested PUT lands for "scores/saved.jianpu" containing "1 2 3 4 5"
    And the beforeunload status badge shows exactly "Saved"
    When I close the page after the save has landed
    Then no beforeunload dialog is shown
