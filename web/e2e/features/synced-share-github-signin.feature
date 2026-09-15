Feature: Synced Share GitHub sign-in

  # Covers the sign-in UI itself (task 8's popup OAuth flow, "Signed in as
  # @username" identity row) and the full-screen verification-failure
  # dialog (task 9) -- GitHub sign-in is now a hard requirement to create or
  # own a Synced Share (no anonymous ownership path). The rest of the
  # Synced Share scenarios (synced-share-button.feature) pre-seed a signed-in
  # connection instead of exercising this UI, to keep their focus on the
  # sync/viewer behavior. Every scenario opens the share modal's Synced-link
  # tab first, via the shared "the owner loads the app and clicks 'Sync'"
  # step (see synced-share-button.steps.ts's `openSyncedTab`).

  Scenario: Clicking Sync without a GitHub connection shows a sign-in prompt, and completing the popup lets a second click start syncing with an identity row shown
    Given clipboard permissions are granted
    And the GitHub authorization popup is mocked to redirect back successfully
    When the owner loads the app and clicks "Sync"
    Then the sign-in prompt is shown
    When the owner clicks "Sign in with GitHub" in the prompt
    Then the identity row shows signed in as "e2e-test-user"
    When the owner clicks "Sync" again
    Then the synced link is copied
    And the synced share identity row reads "Synced as @e2e-test-user"

  Scenario: The GitHub sign-in popup closes itself automatically once sign-in completes
    Given clipboard permissions are granted
    And the GitHub authorization popup is mocked to redirect back successfully
    When the owner loads the app and clicks "Sync"
    Then the sign-in prompt is shown
    When the owner clicks "Sign in with GitHub" in the prompt
    Then the identity row shows signed in as "e2e-test-user"
    And the GitHub sign-in popup has closed itself

  Scenario: The GitHub sign-in still succeeds when the callback page's effect runs twice (React StrictMode double-invoke)
    Given clipboard permissions are granted
    And the GitHub authorization popup is mocked to redirect back successfully
    When the owner loads the app and clicks "Sync"
    Then the sign-in prompt is shown
    When the owner clicks "Sign in with GitHub" in the prompt
    Then the identity row shows signed in as "e2e-test-user"

  Scenario: An unparseable response from the GitHub token exchange fails cleanly instead of leaving the popup stuck "Signing in…"
    Given clipboard permissions are granted
    And the GitHub authorization popup is mocked to redirect back successfully
    And the Synced Share worker returns an unparseable body from the next GitHub token exchange
    When the owner loads the app and clicks "Sync"
    Then the sign-in prompt is shown
    When the owner clicks "Sign in with GitHub" in the prompt
    Then the sign-in prompt shows a sign-in error containing "unreadable response"
    And the "Sign in with GitHub" button is no longer stuck signing in

  Scenario: A blocked GitHub sign-in popup returns to idle with no share created
    Given clipboard permissions are granted
    And the GitHub sign-in popup is blocked by the browser
    When the owner loads the app and clicks "Sync"
    Then the sign-in prompt is shown
    When the owner clicks "Sign in with GitHub" in the prompt
    Then the sign-in prompt is shown
    And the "Sign in with GitHub" button is no longer stuck signing in

  Scenario: Logging out disconnects GitHub and stops the sync
    Given clipboard permissions are granted
    And the owner is signed in with GitHub as "e2e-test-user"
    When the owner loads the app and clicks "Sync"
    Then the synced link is copied
    When the owner clicks "Log out" in the identity row
    Then the sign-in prompt is shown

  Scenario: Logging out of the Synced Share GitHub connection revokes the grant, so a later sign-in genuinely needs fresh consent
    Given clipboard permissions are granted
    And the owner is signed in with GitHub as "e2e-test-user"
    And the GitHub grant-revocation endpoint is mocked
    When the owner loads the app and clicks "Sync"
    And the owner clicks "Log out" in the identity row
    Then the worker was asked to revoke the GitHub grant for "e2e-fake-synced-share-token"

  Scenario: A GitHub verification failure while creating a share shows the full-screen error dialog, with no automatic retry
    Given clipboard permissions are granted
    And the owner is signed in with GitHub as "e2e-test-user"
    And the Synced Share worker rejects the next create-share request with a verification failure
    When the owner loads the app and clicks "Sync"
    Then the synced share error dialog is shown with a "file a GitHub issue" link
    And the synced share error dialog shows the reason "GitHub verification failed"
    When the owner dismisses the synced share error dialog
    Then the synced share error dialog is gone
    When the owner reopens the share modal
    Then the sign-in prompt is shown

  Scenario: A GitHub verification failure clears the stale connection, so retrying prompts a fresh sign-in instead of repeating the same failure
    Given clipboard permissions are granted
    And the owner is signed in with GitHub as "e2e-test-user"
    And the GitHub authorization popup is mocked to redirect back successfully
    And the Synced Share worker rejects the next create-share request with a verification failure
    When the owner loads the app and clicks "Sync"
    Then the synced share error dialog is shown with a "file a GitHub issue" link
    When the owner dismisses the synced share error dialog
    When the owner reopens the share modal
    Then the sign-in prompt is shown
    When the owner clicks "Sign in with GitHub" in the prompt
    Then the identity row shows signed in as "e2e-test-user"
