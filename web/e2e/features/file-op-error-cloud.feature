Feature: File-op failure handling for the cloud storage backend

  Background:
    # Dedicated login, not the shared default "e2e-test-user" -- this
    # scenario asserts no file named exactly "untitled" exists after a
    # failed create, but "untitled" is also the literal name
    # `files-cloud-backend.feature`'s "Creating a file persists it to the
    # cloud backend" scenario's own "New" click produces. Sharing an
    # account would make that scenario's genuine "untitled" file, created
    # concurrently in another `fullyParallel` worker, leak into this one's
    # assertion. See `mockGithubIdentity.mjs`'s doc comment on this login.
    Given an account is signed in as "e2e-test-user-create-error"

  Scenario: A failed create shows the error modal, resets pending state, and a retry succeeds
    Given a cloud file named "existing.jianpu" is seeded for the signed-in account
    And the first file-create request will fail with a 500 error
    When the app loads the cloud-backed file list for a failing create
    And I remember the currently active tab name
    And I click the "New" button to create a file that will fail
    Then the error modal is shown with message "Could not create file" containing "worker request failed with status 500"
    When I close the error modal
    Then the new-file button spinner clears and its label resets to "New"
    And no "untitled" tab exists
    And the active tab is unchanged from before the failed create
    When I retry the "New" button in the file actions menu
    Then the retried create succeeds and the active tab becomes "untitled"
