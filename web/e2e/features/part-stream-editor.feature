Feature: Part stream editor

  # Edits one part of the `# score` as a single stream (no barlines), shown
  # as a diff against the extracted stream, and writes it back on Apply.

  Background:
    Given the part stream editor fixture is loaded

  Scenario: Shifting a note forward re-bars every later measure
    When I open the part stream editor for part "M"
    Then the original part stream is "1 2 3 4 5 6 7 1"
    When I replace the edited part stream with "0 1 2 3 4 5 6 7 1"
    Then the part stream footer says "3 measures changed"
    And the part stream Apply button is enabled
    When I apply the part stream
    Then the part stream editor is closed
    And the main editor source is the shifted melody source

  Scenario: Deleting a note fills the last measure with rests
    When I open the part stream editor for part "M"
    And I replace the edited part stream with "2 3 4 5 6 7 1"
    And I apply the part stream
    Then the main editor source is the shortened melody source

  Scenario: A barline is an error and disables Apply
    When I open the part stream editor for part "M"
    And I replace the edited part stream with "1 2 3 4 | 5 6 7 1"
    Then the edited part stream has an error marker
    And the part stream Apply button is disabled

  Scenario: A follow part shows its copied measures as @ and round-trips unchanged
    When I open the part stream editor for part "A"
    Then the original part stream is "@ 3 4 5 6"
    And the edited part stream is "@ 3 4 5 6"
    And the part stream footer says "0 measures changed"
    And the part stream Apply button is disabled

  Scenario: A lyric part is re-split by its target's notes
    When I open the part stream editor for part "W"
    Then the original part stream is "a b c d e f g h"
    When I replace the edited part stream with "z a b c d e f g"
    Then the part stream footer says "2 measures changed"
    When I apply the part stream
    Then the main editor source is the shifted lyrics source

  Scenario: A lyric part bound to a chords part is listed but disabled with a reason
    Given the chord-bound lyrics fixture is loaded instead
    When I open the part stream picker
    Then the part stream picker row for part "C" is disabled
    And the part stream picker row for part "C" explains "chords part"
    And the part stream picker row for part "H" is enabled

  Scenario: Apply is undoable from the main editor
    When I open the part stream editor for part "M"
    And I replace the edited part stream with "0 1 2 3 4 5 6 7 1"
    And I apply the part stream
    Then the main editor source is the shifted melody source
    When I press undo in the main editor
    Then the main editor source is the original fixture source

  Scenario: Escape closes the editor without changing the source
    When I open the part stream editor for part "M"
    And I replace the edited part stream with "0 1 2 3 4 5 6 7 1"
    And I press Escape in the part stream editor
    Then the part stream editor is closed
    And the main editor source is the original fixture source

  Scenario: Cancel closes the editor without changing the source
    When I open the part stream editor for part "M"
    And I replace the edited part stream with "0 1 2 3 4 5 6 7 1"
    And I click Cancel in the part stream editor
    Then the part stream editor is closed
    And the main editor source is the original fixture source

  Scenario: The diff layout can be toggled and the choice is remembered
    When I open the part stream editor for part "M"
    Then the part stream diff is shown side by side
    When I toggle the part stream diff layout
    Then the part stream diff is shown inline
    When I replace the edited part stream with "0 1 2 3 4 5 6 7 1"
    Then the part stream footer says "3 measures changed"
    When I click Cancel in the part stream editor
    And I open the part stream editor for part "M"
    Then the part stream diff is shown inline

  Scenario: The diff is inline by default on a phone screen
    When I resize the window to a phone screen
    And I open the part stream editor for part "M"
    Then the part stream diff is shown inline
    When I toggle the part stream diff layout
    Then the part stream diff is shown side by side
