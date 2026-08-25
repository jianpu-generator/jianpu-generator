Feature: Sequence-chain drag selection scrolls the editor to the drag's endpoint

  Scenario: Dragging from "C" to the repeated "Intro" reveals Intro's written lines, not C's
    Given a sequence chain "Intro, C, Intro" padded with filler measures is seeded for editor reveal target
    When the app loads with the editor-reveal sequence toolbar ready
    When I drag from the "C" sequence entry to the repeated "Intro" sequence entry, as seen in editor reveal target
    Then the editor scrolls to reveal Intro's written lines
