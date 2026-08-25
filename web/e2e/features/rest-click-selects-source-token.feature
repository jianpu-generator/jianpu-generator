Feature: Rest click selects source token

  Scenario: Selecting a rest in the SVG preview selects its "0" in the editor
    Given the rest-click test fixture is loaded
    When I drag just past the note-drag arm threshold inside the rest's own click target
    Then 1 note is drag-selected, as seen in rest click selects source token
    And the play-measure button reads Selection, as seen in rest click selects source token
    And the editor's selected text is "0"
