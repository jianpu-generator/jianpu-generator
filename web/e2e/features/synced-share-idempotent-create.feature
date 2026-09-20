Feature: Idempotent Synced Share creation for GitHub-backed files

  # A GitHub-backed file's share is keyed by (GitHub account, owner/repo/path)
  # server-side (crates/live-share-worker's docs.external_file_id), not the
  # per-browser localStorage cache alone -- so re-sharing the same file as
  # the same GitHub account always resolves to the same link, even from a
  # different browser context or origin. Local-only files (no GitHub storage
  # backend) are unaffected -- see synced-share-button.feature's own
  # "Re-syncing on the same file reproduces the same link" for that case.

  Background:
    Given clipboard permissions are granted

  Scenario: Sharing the same GitHub-backed file from a second browser context, signed in as the same GitHub account, reproduces the same link
    Given the GitHub repo is seeded with a file named "scores/idempotent.jianpu" for idempotent sharing
    And GitHub auth is seeded for the mocked owner
    And the owner is signed in with GitHub as "e2e-test-user"
    When the app loads the GitHub-backed file list for idempotent sharing
    And I select the "idempotent" tab to test idempotent sharing
    And the owner loads the app and clicks "Sync"
    Then the synced link is copied
    When a separate browser context loads the same GitHub-backed file, signed in as the same GitHub account "e2e-test-user"
    And the owner clicks "Sync" in that separate browser context
    Then the synced link copied in the separate browser context is identical to the original link

  Scenario: Sharing the same GitHub-backed file as a different GitHub account does not reuse the first account's link
    Given the GitHub repo is seeded with a file named "scores/idempotent.jianpu" for idempotent sharing
    And GitHub auth is seeded for the mocked owner
    And the owner is signed in with GitHub as "e2e-test-user"
    When the app loads the GitHub-backed file list for idempotent sharing
    And I select the "idempotent" tab to test idempotent sharing
    And the owner loads the app and clicks "Sync"
    Then the synced link is copied
    When a separate browser context loads the same GitHub-backed file, signed in as a different GitHub account "e2e-test-user-two"
    And the owner clicks "Sync" in that separate browser context
    Then the synced link copied in the separate browser context is different from the original link

  Scenario: Sharing two different GitHub-backed files as the same GitHub account produces two different links
    Given the GitHub repo is seeded with a file named "scores/idempotent-a.jianpu" for idempotent sharing
    And the GitHub repo is seeded with a file named "scores/idempotent-b.jianpu" for idempotent sharing
    And GitHub auth is seeded for the mocked owner
    And the owner is signed in with GitHub as "e2e-test-user"
    When the app loads the GitHub-backed file list for idempotent sharing
    And I select the "idempotent-a" tab to test idempotent sharing
    And the owner loads the app and clicks "Sync"
    Then the synced link is copied
    When I select the "idempotent-b" tab to test idempotent sharing
    And the owner clicks "Sync" again
    Then the synced link is copied
    And the copied sync link is different from the original link

  Scenario: A local-only file (no GitHub storage backend) is unaffected -- re-sharing from a second browser context still mints a new link
    Given local storage is cleared
    And the owner is signed in with GitHub as "e2e-test-user"
    When the owner loads the app and clicks "Sync"
    Then the synced link is copied
    When a separate browser context loads the app, signed in as the same GitHub account "e2e-test-user"
    And the owner clicks "Sync" in that separate browser context
    Then the synced link copied in the separate browser context is different from the original link

  Scenario: Renaming a GitHub-backed file after sharing keeps the active sync alive, but a fresh share from elsewhere afterward gets a new link
    Given the GitHub repo is seeded with a file named "scores/idempotent.jianpu" for idempotent sharing
    And GitHub auth is seeded for the mocked owner
    And the owner is signed in with GitHub as "e2e-test-user"
    When the app loads the GitHub-backed file list for idempotent sharing
    And I select the "idempotent" tab to test idempotent sharing
    And the owner loads the app and clicks "Sync"
    Then the synced link is copied
    When the owner renames the active file to "renamed"
    Then the share modal still shows the stop-sync button
    And the synced link is unchanged after the rename
    When a separate browser context loads the GitHub-backed file at its renamed path, signed in as the same GitHub account "e2e-test-user"
    And the owner clicks "Sync" in that separate browser context
    Then the synced link copied in the separate browser context is different from the original link
