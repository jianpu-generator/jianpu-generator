Feature: Sequence-chain drag selection scrolls the preview to the drag's endpoint

  Scenario: Dragging from "Intro" to "C" scrolls the preview to "C", not "Intro"
    Given a sequence chain "Intro, C" padded with filler measures is seeded for preview reveal target
    When the app loads with the preview-reveal sequence toolbar ready
    Then the last measure is not in the preview viewport
    When I drag from the "Intro" sequence entry to the "C" sequence entry, as seen in preview reveal target
    Then the last measure scrolls into the preview viewport
