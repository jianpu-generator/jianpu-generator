Feature: Storage settings error banner for GitHub autosave failures

  Scenario: A rate-limited autosave shows the rate-limit banner, which clears once a save succeeds
    Given the GitHub repo is seeded with a file named "scores/banner.jianpu" for a rate-limit banner
    And the first autosave PUT will fail with a 403 rate-limit response
    And I open and edit "banner.jianpu" with suffix " 5" and a fake clock installed
    When I fast-forward the clock past the autosave debounce interval for the error banner
    And I open the storage settings modal to check the error banner
    Then the status banner is visible and mentions "rate limit"
    When I close the storage settings modal
    And I append " 6" to the editor to trigger a recovering autosave
    Then the editor contains "1 2 3 4 5 6"
    When I fast-forward the clock past the autosave debounce interval for the error banner
    Then the recovering-autosave PUT lands for "scores/banner.jianpu" containing "1 2 3 4 5 6"
    When I open the storage settings modal to check the error banner
    Then the status banner is gone

  Scenario: A network-failed autosave shows the offline banner, which clears once a save succeeds
    Given the GitHub repo is seeded with a file named "scores/banner.jianpu" for an offline banner
    And the first autosave PUT will be aborted as a network failure
    And I open and edit "banner.jianpu" with suffix " 5" and a fake clock installed
    When I fast-forward the clock past the autosave debounce interval for the error banner
    And I open the storage settings modal to check the error banner
    Then the status banner is visible and mentions "offline"
    When I close the storage settings modal
    And I append " 6" to the editor to trigger a recovering autosave
    Then the editor contains "1 2 3 4 5 6"
    When I fast-forward the clock past the autosave debounce interval for the error banner
    Then the recovering-autosave PUT lands for "scores/banner.jianpu" containing "1 2 3 4 5 6"
    When I open the storage settings modal to check the error banner
    Then the status banner is gone
