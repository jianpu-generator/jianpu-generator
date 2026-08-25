Feature: Resolving a GitHub save conflict

  Scenario: Overwriting mine re-pushes the in-memory edit and clears the conflict banner
    Given a GitHub save conflict is set up on "scores/conflict.jianpu" for overwrite-mine
    When I click the conflict-resolution button "Overwrite mine"
    Then the conflict banner is gone
    And the last PUT for the conflict contains "1 2 3 4 5"
    And the conflict status badge shows exactly "Saved"
    And the editor still contains "1 2 3 4 5"

  Scenario: Discarding mine reloads the remote content and clears the conflict banner
    Given a GitHub save conflict is set up on "scores/conflict.jianpu" for discard-mine
    And the remote file has since changed to the conflicting content
    When I click the conflict-resolution button "Discard mine"
    Then the conflict banner is gone
    And the conflict status badge shows exactly "Saved"
    And the editor now shows the remote content "5 6 7 1"
    And the editor no longer contains "1 2 3 4 5"
