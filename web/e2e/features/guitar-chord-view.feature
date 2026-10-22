Feature: Guitar chord view

  Background:
    Given a chord progression in C major is loaded for the guitar chord view

  Scenario: Switching the preview to Guitar draws a chord diagram at each chord symbol
    Then the preview shows no guitar chord diagrams
    When I switch the preview notation to "guitar"
    Then the preview's guitar chord diagrams read "x 3 2 0 1 0, x 0 2 2 1 0, 1 3 3 2 1 1, 3 2 0 0 0 3"
    And the preview's chord symbols read "1 6m 4 5"
    When I switch the preview notation to "jianpu"
    Then the preview shows no guitar chord diagrams

  Scenario: The guitar chord view is remembered after a reload
    When I switch the preview notation to "guitar"
    And I reload the page
    Then the preview's guitar chord diagrams read "x 3 2 0 1 0, x 0 2 2 1 0, 1 3 3 2 1 1, 3 2 0 0 0 3"

  Scenario: Exporting a PDF from the guitar chord view draws the diagrams
    When I export "PDF" from the preview, keeping it as the jianpu PDF
    And I switch the preview notation to "guitar"
    And I export "PDF" from the preview, keeping it as the guitar PDF
    Then the guitar PDF differs from the jianpu PDF
