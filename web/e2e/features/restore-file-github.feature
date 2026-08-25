Feature: Restore file from bin via GitHub storage backend

  Scenario: Restoring a file persists via the GitHub storage backend
    Given the GitHub repo is seeded with only a binned file "trash/original.jianpu" for restoring
    And GitHub auth is seeded for the mocked owner
    When the app loads with the preview ready for the restore test
    And I open the file list to check the restore
    Then no "original" tab exists in the main list
    And the file actions bin trigger shows "Bin (1)" before restoring
    When I open the bin to restore a file
    Then the bin lists the restorable file "original"
    When I click the restore-from-bin button for "original.jianpu"
    Then the restore-from-bin button shows a pending spinner
    And the bin modal closes once the plain restore resolves
    When I open the file list to check the restore
    Then the "original" tab reappears within 5 seconds
    And the active tab is now the restored file "original"
    And the file actions bin trigger disappears after restoring
    When I reload the page after restoring
    Then the file list still shows "original" after the restore reload
    And the file actions bin trigger is gone after the restore reload
