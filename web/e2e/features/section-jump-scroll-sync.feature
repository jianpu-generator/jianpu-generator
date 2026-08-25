Feature: Section jump scroll sync

  Background:
    Given a long sectioned source with sections "A" and "B" is loaded

  Scenario: Clicking a section button scrolls the SVG preview to that section
    When I click the section jump button labeled "B" to scroll to that section
    Then the selected measure range shows the last measure selected
    And the SVG preview scrolls to bring the last measure into view
