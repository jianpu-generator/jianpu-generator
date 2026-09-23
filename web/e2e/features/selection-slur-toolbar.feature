Feature: Slur or unslur the selected range from the editor toolbar

  Scenario: Slur/Unslur wraps the selected notes in a slur
    Given the single-measure melody-bass fixture is loaded
    When I select "2 3" on the Melody line
    And I click the "Slur/Unslur" editor toolbar button
    Then the editor source contains "[Melody] 1 (2 3) 4"
    And the stored source contains "[Melody] 1 (2 3) 4"
    And the editor source still contains "[Bass] 5 6 7 1"

  Scenario: Clicking Slur/Unslur again removes the slur it just added
    Given the single-measure melody-bass fixture is loaded
    When I select "2 3" on the Melody line
    And I click the "Slur/Unslur" editor toolbar button
    And I click the "Slur/Unslur" editor toolbar button
    Then the editor source contains "[Melody] 1 2 3 4"
    And the stored source contains "[Melody] 1 2 3 4"

  Scenario: A selection spanning two parts slurs each part separately
    Given the single-measure melody-bass fixture is loaded
    When I precisely select from "3 4" on the Melody line to "5 6" on the Bass line
    And I click the "Slur/Unslur" editor toolbar button
    Then the editor source contains "[Melody] 1 2 (3 4)"
    And the editor source contains "[Bass] (5 6) 7 1"
    And the stored source contains "[Melody] 1 2 (3 4)"
    And the stored source contains "[Bass] (5 6) 7 1"

  Scenario: Clicking a part label slurs that part across every measure in the system
    Given the two-measure melody-harmony click-test fixture is loaded
    When I plain-click the Melody part label
    And I click the "Slur/Unslur" editor toolbar button
    Then the editor source contains "[M] (1 2"
    And the editor source contains "[M] 3 4)"
    And the editor source still contains "[H] 5 6"
    And the editor source still contains "[H] 7 1"

  Scenario: Slur/Unslur keeps the same SVG notes highlighted
    Given the two-measure melody-harmony click-test fixture is loaded
    When I plain-click the Melody part label
    And I remember which notes are highlighted in the SVG preview
    And I click the "Slur/Unslur" editor toolbar button
    Then the same notes are still highlighted in the SVG preview

  Scenario: Slur/Unslur is disabled when there is no range selected
    Given the single-measure melody-bass fixture is loaded
    When I place the caret on the Melody line without selecting a range
    Then the "Slur/Unslur" editor toolbar button is disabled
