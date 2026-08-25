Feature: Soundfont search modal

  Scenario: Fuzzy search narrows results to subsequence matches
    Given the soundfont-search-modal test fixture is loaded
    When I open the soundfont search modal for part "M"
    And I fill the soundfont search box with "vln"
    Then the soundfont search modal shows a button "40: Violin"
    And the soundfont search modal has no button "0: Acoustic Grand Piano"

  Scenario: Tag filter applies an AND-filter across the instrument list
    Given the soundfont-search-modal test fixture is loaded
    When I open the soundfont search modal for part "M"
    And I click the "#strings" tag on the "40: Violin" row in the soundfont search modal
    Then the "#strings" tag on the "40: Violin" row is highlighted as active
    And the soundfont search modal shows a button "40: Violin"
    And the soundfont search modal shows a button "47: Timpani"
    And the soundfont search modal has no button "0: Acoustic Grand Piano"
    When I click the "#strings" tag on the "40: Violin" row in the soundfont search modal
    Then the "#strings" tag on the "40: Violin" row is not highlighted as active
    And the soundfont search modal shows a button "0: Acoustic Grand Piano"

  Scenario: Instrument preview toggles play/pause state
    Given the scenario timeout is extended to 60 seconds, as seen in soundfont search modal
    And the soundfont-search-modal test fixture is loaded
    When I open the soundfont search modal for part "M"
    And I retry clicking the Preview instrument button for "40: Violin" in the soundfont search modal until it pauses
    Then clicking Pause preview for "40: Violin" in the soundfont search modal returns it to Preview instrument
