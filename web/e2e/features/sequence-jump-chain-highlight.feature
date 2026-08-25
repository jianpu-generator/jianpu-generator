Feature: Sequence-chain drag selection highlights only its own measures

  Scenario: Dragging "C" to the repeated "A" selects both entries in the editor and highlights both measures in the preview, excluding "B"
    Given a sequence chain "A, B, C, A" over sections "A", "B", "C" is seeded for chain highlight
    When the app loads with the chain-highlight sequence toolbar ready
    When I drag from the "C" sequence entry to the repeated "A" sequence entry
    Then the editor selects exactly "C" and the repeated "A" as disjoint ranges
    And the preview highlights exactly the "C" and "A" measures, excluding "B"
