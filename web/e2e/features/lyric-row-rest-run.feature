Feature: A lyric row with nothing to sing for several measures shows a multi-measure rest
  When a lyric part has no syllables for 2 or more consecutive measures, its
  row collapses those measures into one rest bar with the count centered above
  it (see "Multi-measure rests" in syntax.md) instead of leaving them blank —
  even though the notes part the lyrics sing along to keeps playing. Only the
  lyric row collapses; the notes row is untouched.

  # Step wording note: step definitions are global across features, so the
  # steps here are worded distinctly from blank-lyric-rows.feature.

  Background:
    Given parts Melody [M], Verse [V] = lyrics[M] are declared for the lyric-row-rest-run score

  Scenario: Blank lyric measures between sung ones collapse into one rest bar in the lyric row
    Given the lyric-row-rest-run score has 5 measures of Melody notes "1 1 1 1"
    And the Verse line is sung "la la la la" in measures 0 and 4 only
    When the lyric-row-rest-run score is laid out
    Then the lyric row shows one rest bar with the count "3"
    And the rest bar spans measures 1 to 3
    And the rest bar sits below the Melody notes
    And the Melody row still shows 20 notes

  Scenario: A single blank lyric measure stays uncollapsed
    Given the lyric-row-rest-run score has 3 measures of Melody notes "1 1 1 1"
    And the Verse line is sung "la la la la" in measures 0 and 2 only
    When the lyric-row-rest-run score is laid out
    Then the lyric row shows no rest bar
