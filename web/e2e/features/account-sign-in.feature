Feature: Unified account sign-in for cloud storage

  Scenario: Signing in via the popup makes cloud storage selectable and switches to it
    Given the GitHub authorization popup is mocked to redirect back successfully
    When the app loads for the account sign-in flow
    And I open the storage settings modal for sign-in
    And I click "Sign in with GitHub" in the storage settings modal
    Then the storage settings modal shows connected as "e2e-test-user"
    When I select the "Cloud storage" storage option
    Then the stored storage-backend preference is set to cloud

  Scenario: Signing out reverts to the local backend and stops saving to the cloud
    Given an account is signed in as "e2e-test-user"
    And a cloud file named "song.jianpu" is seeded for the signed-in account
    When the app loads the cloud-backed file list for disconnect
    And I select the "song" tab before disconnecting
    When I click the account chip
    And I click "Sign out" in the profile popover
    Then the stored account auth is cleared
    When I open the storage settings modal for sign-in
    Then the app no longer shows as connected
    And the "This browser" storage option is checked
    When I close the storage settings modal and attempt to edit and force-save
    Then no content request is sent to the worker after disconnecting
