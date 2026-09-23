Feature: Account chip in the header

  Scenario: Signed out, the header shows a Sign in button that starts the same GitHub popup flow
    Given the GitHub authorization popup is mocked to redirect back successfully
    When the app loads for the account chip flow
    Then the header shows a "Sign in" button and no account chip
    When I click "Sign in" in the header
    Then the header shows the account chip labeled "@e2e-test-user"
    And the header no longer shows a "Sign in" button

  Scenario: Signed in, opening the chip shows a profile popover with a link to the GitHub profile and a Sign out action
    Given an account is signed in as "e2e-test-user"
    When the app loads for the account chip flow
    And I click the account chip
    Then the profile popover shows "@e2e-test-user"
    And the profile popover has a "View GitHub profile" link to "https://github.com/e2e-test-user"

  Scenario: A file currently syncing shows a live indicator on the chip
    Given clipboard permissions are granted
    And the owner is signed in with GitHub as "e2e-test-user"
    And the file store is seeded with the synced score
    When the owner loads the app and clicks "Sync"
    Then the synced link is copied
    And the header account chip shows the syncing indicator

  Scenario: Signing out from the profile popover clears the account everywhere at once
    Given clipboard permissions are granted
    And the owner is signed in with GitHub as "e2e-test-user"
    And the file store is seeded with the synced score
    And the GitHub grant-revocation endpoint is mocked
    When the owner loads the app and clicks "Sync"
    Then the synced link is copied
    When I close the share modal
    And I click the account chip
    And I click "Sign out" in the profile popover
    Then the header shows a "Sign in" button and no account chip
    And the worker was asked to revoke the GitHub grant for "e2e-fake-synced-share-token"
    When the owner reopens the share modal
    Then the sign-in prompt is shown
    When I close the share modal
    And I open the storage settings modal for sign-in
    Then the storage settings modal shows the sign-in prompt
    And the "This browser" storage option is checked

  Scenario: Signing out while cloud storage is active falls back to local storage and stops saving to the cloud
    Given an account is signed in as "e2e-test-user"
    And a cloud file named "song.jianpu" is seeded for the signed-in account
    And the stored storage-backend preference is "cloud"
    When the app loads the cloud-backed file list for the account chip flow
    And I select the "song" tab
    And I click the account chip
    And I click "Sign out" in the profile popover
    Then the stored account auth is cleared
    And the stored storage-backend preference is set to local
    When I edit and force-save
    Then no content request is sent to the worker after signing out
