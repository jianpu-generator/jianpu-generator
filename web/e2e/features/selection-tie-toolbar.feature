Feature: Tie or untie the selected range from the editor toolbar

  Scenario: Tie/Untie ties the selected same-pitch notes
    Given the single-measure repeated-pitch fixture is loaded
    When I precisely select "3 3" on the Melody line
    And I click the "Tie/Untie" editor toolbar button
    Then the editor source contains "[Melody] 1 3~ 3 4"
    And the stored source contains "[Melody] 1 3~ 3 4"
    And the editor source still contains "[Bass] 5 5 7 1"

  Scenario: Clicking Tie/Untie again removes the tie it just added
    Given the single-measure repeated-pitch fixture is loaded
    When I precisely select "3 3" on the Melody line
    And I click the "Tie/Untie" editor toolbar button
    And I click the "Tie/Untie" editor toolbar button
    Then the editor source contains "[Melody] 1 3 3 4"
    And the stored source contains "[Melody] 1 3 3 4"

  Scenario: A selection spanning two parts ties each part separately
    Given the single-measure repeated-pitch fixture is loaded
    When I precisely select from "3 3 4" on the Melody line to "5 5" on the Bass line
    And I click the "Tie/Untie" editor toolbar button
    Then the editor source contains "[Melody] 1 3~ 3 4"
    And the editor source contains "[Bass] 5~ 5 7 1"
    And the stored source contains "[Melody] 1 3~ 3 4"
    And the stored source contains "[Bass] 5~ 5 7 1"

  Scenario: Tie/Untie leaves notes of different pitches untied
    Given the single-measure melody-bass fixture is loaded
    When I precisely select "1 2 3 4" on the Melody line
    And I click the "Tie/Untie" editor toolbar button
    Then the editor source contains "[Melody] 1 2 3 4"

  Scenario: Tie/Untie is disabled when there is no range selected
    Given the single-measure repeated-pitch fixture is loaded
    When I place the caret on the Melody line without selecting a range
    Then the "Tie/Untie" editor toolbar button is disabled
