Feature: Share links

  Scenario: Opens a shared score preview without saving it, then imports on demand
    Given local storage is cleared, as seen in share
    When I open the share URL for "shared-test.jianpu"
    Then the shared preview banner shows "shared-test.jianpu"
    And the file switcher is hidden entirely
    And the preview contains "Shared Score"
    When I reload the page
    Then the file switcher is hidden entirely
    When I open the share URL for "shared-test.jianpu" again
    And I click "Import to my scores"
    Then the file switcher shows "shared-test"
    And the shared preview banner is gone

  Scenario: Collapses the editor pane and hides its toggle when viewing a shared score
    Given local storage is cleared, as seen in share
    When I open the share URL for "shared-test.jianpu"
    Then the editor pane is collapsed, as seen in share
    And the pane-divider toggle is hidden
    When I click "Discard"
    Then the pane-divider toggle is visible again
    When I click the pane-divider toggle, as seen in share
    Then the editor pane is expanded, as seen in share

  Scenario: Discarding a shared preview does not save it
    Given local storage is cleared, as seen in share
    When I open the share URL for "shared-test.jianpu"
    Then the shared preview banner is visible
    When I click "Discard"
    Then the shared preview banner is gone
    And the file switcher no longer shows "shared-test"

  Scenario: Opens legacy uncompressed share links
    Given local storage is cleared, as seen in share
    When I navigate to a legacy uncompressed share link for "shared-test.jianpu"
    Then the shared preview banner shows "shared-test.jianpu"

  Scenario: Share button copies a compressed link that opens as a preview
    Given clipboard permissions are granted, as seen in share
    And a user file "shared-test.jianpu" is seeded in local storage
    When the app loads, as seen in share
    And I open the file actions menu and click the share button
    Then the share button shows "Link copied"
    And the copied share URL matches the expected compressed hash for "shared-test.jianpu"
    When I navigate fresh to the copied share URL
    Then the shared preview banner shows "shared-test.jianpu"
    And the preview contains "Shared Score"
