Feature: Harmonica tab notation view

  Background:
    Given a score in C major is loaded for the harmonica notation view

  Scenario: Switching the preview to Harmonica replaces degree numbers with harmonica tabs
    When I switch the preview notation to "harmonica"
    Then the preview's note heads read "4, -4, 5, -5, 6, -6, -7, 7, 1, 8, 10"
    And the preview flags the unplayable note heads "?"
    When I switch the preview notation to "jianpu"
    Then the preview's note heads read "1, 2, 3, 4, 5, 6, 7, 1, 5, 1, 3, 1"

  Scenario: The Harmonica view is remembered after a reload
    When I switch the preview notation to "harmonica"
    And I reload the page
    Then the preview's note heads read "4, -4, 5, -5, 6, -6, -7, 7, 1, 8, 10"

  Scenario: Exporting a PDF from the Harmonica view writes harmonica tabs
    When I export "PDF" from the preview, keeping it as the jianpu PDF
    And I switch the preview notation to "harmonica"
    And I export "PDF" from the preview, keeping it as the harmonica PDF
    Then the harmonica PDF differs from the jianpu PDF
