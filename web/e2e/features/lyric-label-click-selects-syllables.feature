Feature: Lyric label click selects syllables

  Background:
    Given the lyric label click test fixture is loaded and measure spans are primed

  Scenario: Clicking a verse label selects every syllable that verse sings across the system
    When I click the verse 0 lyric label without dragging
    Then verse 0's 4 syllables are drag-selected and verse 1's are not
    And the verse 0 label stays visually active but the verse 1 label does not

  Scenario: Dragging from one verse label to another selects both verses syllables
    When I drag from the verse 0 lyric label to the verse 1 lyric label
    Then verse 0's and verse 1's syllables are all drag-selected
    And both the verse 0 and verse 1 labels stay visually active
