Feature: Viewing and importing a shared folder

  # A shared folder opens read-only for anyone with the link. Every scenario
  # uses its own folder names because the signed-in account is shared by the
  # whole suite run.
  Background:
    Given clipboard permissions are granted
    And an account is signed in as "e2e-test-user"

  Scenario: A viewer of a synced folder link sees a tree rooted at the folder and can open a nested file
    Given a cloud file named "ViewOne/Advent/o-come.jianpu" titled "O Come Emmanuel" is seeded for the signed-in account
    And a cloud file named "ViewOne/amazing.jianpu" titled "Amazing Grace" is seeded for the signed-in account
    And the cloud app is open
    When I open the share dialog of the folder "ViewOne"
    And I open the synced tab of the share dialog
    And I start sync in the share dialog
    Then the share dialog shows a synced link
    When a viewer opens the synced link of the share dialog
    Then the viewer banner says "Synced folder: ViewOne"
    And the viewer banner says "Shared by @e2e-test-user"
    And the viewer's preview contains "O Come Emmanuel"
    When the viewer opens the file list
    Then the viewer's file list shows the file "amazing"
    And the viewer's file list has no Demo folder
    And the viewer's file list offers no editing actions
    When the viewer expands the folder "Advent"
    Then the viewer's file list shows the file "o-come"
    When the viewer selects the file "amazing"
    Then the viewer's preview contains "Amazing Grace"
    And the viewer has no editor
    And the viewer's page URL has the synced share param

  Scenario: Changes by the owner appear after the viewer reloads
    Given a cloud file named "ViewTwo/keep.jianpu" titled "Keep Me" is seeded for the signed-in account
    And a cloud file named "ViewTwo/rename-me.jianpu" titled "Old Name Song" is seeded for the signed-in account
    And a cloud file named "ViewTwo/edit-me.jianpu" titled "Before Edit" is seeded for the signed-in account
    And a cloud file named "ViewTwo/delete-me.jianpu" titled "Doomed Song" is seeded for the signed-in account
    And the cloud app is open
    When I open the share dialog of the folder "ViewTwo"
    And I open the synced tab of the share dialog
    And I start sync in the share dialog
    Then the share dialog shows a synced link
    When a viewer opens the synced link of the share dialog
    And the viewer opens the file list
    Then the viewer's file list shows the file "delete-me"
    When the owner adds the cloud file "ViewTwo/brand-new.jianpu" titled "Brand New Song"
    And the owner renames the cloud file "ViewTwo/rename-me.jianpu" to "ViewTwo/renamed.jianpu"
    And the owner edits the cloud file "ViewTwo/edit-me.jianpu" to be titled "After Edit"
    And the owner deletes the cloud file "ViewTwo/delete-me.jianpu"
    Then the viewer's file list shows the file "delete-me"
    When the viewer reloads the page
    And the viewer opens the file list
    Then the viewer's file list shows the file "brand-new"
    And the viewer's file list shows the file "renamed"
    And the viewer's file list shows no file "rename-me"
    And the viewer's file list shows no file "delete-me"
    When the viewer selects the file "edit-me"
    Then the viewer's preview contains "After Edit"

  Scenario: A viewer sees that a stopped share is no longer shared, after reloading
    Given a cloud file named "ViewFour/song.jianpu" titled "Short Lived" is seeded for the signed-in account
    And the cloud app is open
    When I open the share dialog of the folder "ViewFour"
    And I open the synced tab of the share dialog
    And I start sync in the share dialog
    Then the share dialog shows a synced link
    When a viewer opens the synced link of the share dialog
    Then the viewer's preview contains "Short Lived"
    When the owner stops sharing the folder
    And the viewer reloads the page
    Then the viewer sees the "no longer shared" screen

  Scenario: Importing a synced folder copies it into the viewer's own files
    Given a cloud file named "ViewFive/Advent/o-come.jianpu" titled "Import One" is seeded for the signed-in account
    And a cloud file named "ViewFive/amazing.jianpu" titled "Import Two" is seeded for the signed-in account
    And the cloud app is open
    When I open the share dialog of the folder "ViewFive"
    And I open the synced tab of the share dialog
    And I start sync in the share dialog
    Then the share dialog shows a synced link
    When a viewer opens the synced link of the share dialog
    And the viewer imports the folder
    Then the viewer's page URL has no synced share param
    And the viewer sees no shared folder banner

  Scenario: A folder name already taken in the viewer's files becomes a numbered name
    Given a cloud file named "ViewSix/song.jianpu" titled "Clash Song" is seeded for the signed-in account
    And the cloud app is open
    When I open the share dialog of the folder "ViewSix"
    And I open the synced tab of the share dialog
    And I start sync in the share dialog
    Then the share dialog shows a synced link
    When a viewer who already has the local folder "ViewSix" opens the synced link of the share dialog
    And the viewer imports the folder
    Then the viewer's page URL has no synced share param
    And the viewer's folder list has the folder "ViewSix 2"

  Scenario: A viewer signed in to the cloud imports the folder into the cloud
    Given a cloud file named "ViewSeven/song.jianpu" titled "Cloud Copy" is seeded for the signed-in account
    And the cloud app is open
    When I open the share dialog of the folder "ViewSeven"
    And I open the synced tab of the share dialog
    And I start sync in the share dialog
    Then the share dialog shows a synced link
    When a cloud viewer opens the synced link of the share dialog
    And the viewer imports the folder
    Then the cloud account holds the file "ViewSeven 2/song.jianpu"

  Scenario: A static folder link opens read-only and can be discarded
    Given the file tree is seeded with files "Static/Advent/a.jianpu", "Static/b.jianpu" and "c.jianpu" and an empty folder "Spare" with "c.jianpu" active
    When the file tree app loads
    And I open the share dialog of the folder "Static"
    And a viewer opens the static link of the share dialog
    Then the viewer banner says "Shared folder: Static"
    When the viewer opens the file list
    Then the viewer's file list shows the file "b"
    And the viewer's file list offers no editing actions
    When the viewer discards the folder
    Then the viewer's page URL has no share hash
    And the viewer sees no shared folder banner

  Scenario: A static folder link can be imported
    Given the file tree is seeded with files "Static/Advent/a.jianpu", "Static/b.jianpu" and "c.jianpu" and an empty folder "Spare" with "c.jianpu" active
    When the file tree app loads
    And I open the share dialog of the folder "Static"
    And a viewer opens the static link of the share dialog
    And the viewer imports the folder
    Then the viewer's page URL has no share hash
    And the viewer's folder list has the folder "Static"

  Scenario: On a phone the folder picker and export stay on screen, ahead of the banner's Import button
    Given a cloud file named "ViewEight/song.jianpu" titled "Phone Song" is seeded for the signed-in account
    And the cloud app is open
    When I open the share dialog of the folder "ViewEight"
    And I open the synced tab of the share dialog
    And I start sync in the share dialog
    Then the share dialog shows a synced link
    When a viewer opens the synced link of the share dialog on a phone
    Then the viewer's folder picker is fully inside the screen
    And the viewer's export button is fully inside the screen
    And the viewer's import button is after the export button

  Scenario: On a phone a single synced file's export stays on screen, ahead of the banner's Import button
    Given the owner is signed in with GitHub as "e2e-test-user"
    And the file store is seeded with the synced score
    When the owner loads the app and clicks "Sync"
    Then the synced link is copied
    When a viewer opens the copied sync link in a new page
    And the viewer resizes to a phone screen
    Then the viewer's export button is fully inside the screen
    And the viewer's import button is after the export button
