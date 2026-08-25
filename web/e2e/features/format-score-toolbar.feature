Feature: Format score toolbar

  Background:
    Given the format-score-toolbar test fixture is loaded

  Scenario: Drops redundant lines and collapses whitespace, keeping the caret in place when its position is still valid
    When I focus the editor and place the caret at line 6 column 5
    And I click the format-score toolbar toggle
    Then the editor source is reformatted to the expected output
    And the caret remains at line 6 column 5 after formatting

  Scenario: Clamps the caret to a valid position when its line is dropped by formatting
    # Line 9 is "[Melody] 0 0 0 0" (17 chars), and column 18 is its end —
    # that whole line is dropped by formatting, so this position can't
    # survive as-is.
    When I focus the editor and place the caret at line 9 column 18
    And I click the format-score toolbar toggle
    Then the editor source is reformatted to the expected output
    And the caret is clamped to a valid position within the formatted source
