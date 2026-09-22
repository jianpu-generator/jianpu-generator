Feature: Beforeunload warning for pending cloud saves

  Background:
    Given an account is signed in as "e2e-test-user"

  Scenario: Closing the tab warns while a cloud save is still pending
    Given a cloud file named "pending.jianpu" is seeded for the signed-in account
    And I open and edit "pending" with a fake clock installed
    Then the beforeunload status badge shows "Unsaved"
    When I close the page without letting the save land
    Then a beforeunload dialog is shown

  Scenario: Closing the tab does not warn once the cloud save has landed
    Given a cloud file named "saved.jianpu" is seeded for the signed-in account
    And I open and edit "saved" with a fake clock installed
    When I fast-forward the clock so the beforeunload save lands
    Then the content save lands for "saved.jianpu" containing "1 2 3 4 5"
    And the beforeunload status badge shows exactly "Saved"
    When I close the page after the save has landed
    Then no beforeunload dialog is shown
