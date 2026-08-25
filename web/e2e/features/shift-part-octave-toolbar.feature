Feature: Shift part octave from the Edit Parts toolbar

  Scenario: Clicking notation octave up shifts only the target part in the editor text
    Given the melody-bass fixture is loaded and the app has navigated home
    And I open the Edit Parts modal, as seen in shift part octave toolbar
    When I click the notation octave-up control for part "M"
    Then the editor source contains "[M] 1' 2' 3' 4'"
    And the stored source contains "[M] 1' 2' 3' 4'"
    And the editor source still contains "[B] 5 6 7 1"

  Scenario: Notation octave down control is hidden for a follow part
    Given the melody-follow-chords fixture is loaded and the app has navigated home
    And I open the Edit Parts modal, as seen in shift part octave toolbar
    Then the notation octave-up control for part "M" is visible
    And there is no notation octave-up control for part "C"
