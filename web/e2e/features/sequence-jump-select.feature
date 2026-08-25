Feature: Sequence jump select

  Background:
    Given a two-section source with a passthrough sequence "A, B" is loaded

  Scenario: Clicking the "A" sequence entry selects measures 0-1 and highlights lines 11-14 in Monaco
    When I click sequence entry button 0
    Then the sequence entry selected measure range is "0-1"
    And the sequence entry Monaco selection spans lines 11 to 14

  Scenario: Clicking the "B" sequence entry selects measures 2-3 and highlights lines 16-19 in Monaco
    When I click sequence entry button 1
    Then the sequence entry selected measure range is "2-3"
    And the sequence entry Monaco selection spans lines 16 to 19
