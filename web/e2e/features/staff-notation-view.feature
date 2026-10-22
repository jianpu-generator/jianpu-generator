Feature: Staff notation view

  Background:
    Given a two-part score is loaded for the staff notation view

  Scenario: Switching the preview to staff notation shows Verovio-engraved staves
    Then the preview shows jianpu pages
    When I switch the preview notation to "staff"
    Then the preview shows a staff notation page
    And the staff notation page has a note for every source note

  Scenario: The staff view leaves out hidden parts
    When I switch the preview notation to "staff"
    And I hide the "A" part from the staff view
    Then the staff notation page has 4 notes

  Scenario: The chosen notation is remembered after a reload
    When I switch the preview notation to "staff"
    Then the preview shows a staff notation page
    When I reload the page
    Then the preview shows a staff notation page
    When I switch the preview notation to "jianpu"
    Then the preview shows jianpu pages
