Feature: Storage settings error banner for cloud autosave failures

  Background:
    Given an account is signed in as "e2e-test-user"

  Scenario: A network-failed autosave shows the offline banner, which clears once a save succeeds
    Given a cloud file named "banner.jianpu" is seeded for the signed-in account
    And the first content-save request will be aborted as a network failure
    And I open and edit "banner" with suffix " 5" and a fake clock installed
    When I fast-forward the clock past the autosave debounce interval for the error banner
    And I open the storage settings modal to check the error banner
    Then the status banner is visible and mentions "offline"
    When I close the storage settings modal
    And I append " 6" to the editor to trigger a recovering autosave
    Then the editor contains "1 2 3 4 5 6"
    When I fast-forward the clock past the autosave debounce interval for the error banner
    Then the content save lands for "banner.jianpu" containing "1 2 3 4 5 6"
    When I open the storage settings modal to check the error banner
    Then the status banner is gone

  Scenario: An expired sign-in on autosave shows the reconnect banner
    Given a cloud file named "banner-reconnect.jianpu" is seeded for the signed-in account
    And the first content-save request will fail with a 401 response
    And I open and edit "banner-reconnect" with suffix " 5" and a fake clock installed
    When I fast-forward the clock past the autosave debounce interval for the error banner
    And I open the storage settings modal to check the error banner
    Then the status banner is visible and mentions "reconnect"
