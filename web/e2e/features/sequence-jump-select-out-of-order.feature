Feature: Out-of-order sequence-chain drag selection

  Scenario: Drag-selecting the out-of-order "C, A" chain selects both entries as disjoint ranges, excluding "B"
    Given a sequence chain "C, A" over sections "A", "B", "C" is seeded for out-of-order selection
    When the app loads with the out-of-order sequence toolbar ready
    When I drag from the "C" sequence entry to the "A" sequence entry
    Then the selected measure range shows "0-2"
    And the editor selects exactly "C" and "A" as disjoint ranges, excluding "B"
