Feature: Restoring a bin file that collides with an active file via GitHub storage backend

  Scenario: Restoring a file that collides with an active file renames it via the GitHub storage backend
    Given the GitHub repo has "scores/original.jianpu" active and "trash/original.jianpu" binned, for a restore collision
    And GitHub auth is seeded for the mocked owner
    When the app loads with the preview ready for the collision test
    And I open the file list
    Then exactly one "original" tab exists and no "original 2" tab exists
    And the file actions bin trigger shows "Bin (1)" before the collision restore
    When I open the bin
    Then the bin lists the colliding restorable file "original"
    When I click the restore button for "original.jianpu"
    Then the colliding restore button shows a pending spinner
    And the bin modal closes once the colliding restore resolves
    When I open the file list
    Then a "original 2" tab appears within 5 seconds
    And both "original" and "original 2" tabs exist exactly once each
    And the active tab is now the renamed restored file "original 2"
    And the file actions bin trigger is gone after the collision restore
    And no PUT was ever sent to "scores/original.jianpu" during the restore
    When I reload the page after the collision restore
    Then both "original" and "original 2" tabs exist exactly once each after reload
    And the file actions bin trigger is gone after reload
    And the pre-existing tab's content still shows "Existing Active File"
