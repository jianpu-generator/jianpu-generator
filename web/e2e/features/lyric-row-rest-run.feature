Feature: Blank lyric measures collapse into a multi-measure rest like any other resting part
  A measure collapses into a multi-measure rest only when every visible part
  rests in it (see "Multi-measure rests" in syntax.md). A lyric part with no
  syllables counts as resting, so when only lyric parts are visible, a run of
  blank measures shows one rest bar with the count — the same as an all-rest
  notes part. While a part that still plays is visible in those measures,
  nothing collapses.

  # Step wording note: step definitions are global across features, so the
  # steps here are worded distinctly from blank-lyric-rows.feature.

  Background:
    Given parts Melody [M], Verse [V] = lyrics[M] are declared for the lyric-row-rest-run score

  Scenario: Only the lyric part visible, a run of blank measures shows one rest bar
    Given the lyric-row-rest-run score has 5 measures of Melody notes "1 1 1 1"
    And the Verse line is sung "la la la la" in measures 0 and 4 only
    When the lyric-row-rest-run score is laid out
    And the lyric-row-rest-run Verse part is soloed
    Then the lyric row shows one rest bar with the count "3"

  Scenario: A visible notes part keeps blank lyric measures uncollapsed
    Given the lyric-row-rest-run score has 5 measures of Melody notes "1 1 1 1"
    And the Verse line is sung "la la la la" in measures 0 and 4 only
    When the lyric-row-rest-run score is laid out
    Then the lyric row shows no rest bar
