Feature: A lyric part with nothing to sing is a resting part like any other
  A lyric part with no syllables in a measure follows the same
  hide_resting_parts rule as a notes part with only rests, whatever other
  parts are shown, hidden or soloed. When every visible part in a measure is
  resting, all of them are kept (as for an all-rest notes measure).

  # Step wording note: step definitions are global across features, so the
  # steps that would collide with omitted-part-rest-glyph.feature
  # (hide_resting_parts / Melody notes) are worded distinctly here.

  Background:
    Given parts Melody [M], Verse 1 [V1] = lyrics[M], Verse 2 [V2] = lyrics[M], Chords [C] are declared

  Scenario: A blank lyric part before a written one is hidden when hide_resting_parts is on
    Given the blank-lyric-rows score sets hide_resting_parts to "yes"
    And measure 0's Melody notes are "1 2 3 4"
    And measure 0's Verse 2 line has syllables "la la la la"
    And measure 0 has no Verse 1 line
    When the blank-lyric-rows score is laid out
    Then measure 0 has rows "M", "V2"

  Scenario: A blank lyric part is still drawn when hide_resting_parts is off
    Given the blank-lyric-rows score sets hide_resting_parts to "no"
    And measure 0's Melody notes are "1 2 3 4"
    And measure 0's Verse 2 line has syllables "la la la la"
    And measure 0 has no Verse 1 line
    When the blank-lyric-rows score is laid out
    Then measure 0 has rows "M", "V1", "V2", "C"
    And the Verse 1 row in measure 0 shows no syllables

  Scenario: Soloing lyrics and chords hides a blank lyric part
    Given the blank-lyric-rows score sets hide_resting_parts to "yes"
    And measure 0's Melody notes are "1 2 3 4"
    And measure 0's Verse 2 line has syllables "la la la la"
    And measure 0's Chords line has chords "1 - - -"
    And measure 0 has no Verse 1 line
    When the blank-lyric-rows score is laid out
    And Verse 1, Verse 2 and Chords are soloed
    Then measure 0 has rows "V2", "C"

  Scenario: Hiding the notes part leaves a written lyric part at the notes' column positions
    Given the blank-lyric-rows score sets hide_resting_parts to "yes"
    And measure 0's Melody notes are "1 2 3 4"
    And measure 0's Verse 1 line has syllables "la la la la"
    When the blank-lyric-rows score is laid out
    And the Melody part is hidden
    Then the Verse 1 syllables sit at the same columns as before hiding

  Scenario: A measure whose only visible parts are blank lyric parts keeps them all
    Given the blank-lyric-rows score sets hide_resting_parts to "yes"
    And measure 0's Melody notes are "1 2 3 4"
    And measure 0 has no Verse 1 line
    And measure 0 has no Verse 2 line
    When the blank-lyric-rows score is laid out
    And Verse 1 and Verse 2 are soloed
    # Two identical blank rows are merged into one drawn row whose label
    # lists both parts, exactly as two all-rest notes parts would be.
    Then measure 0 has rows "V1 V2"

  Scenario: Hiding a blank lyric part does not shift the written one's label
    Given the blank-lyric-rows score sets hide_resting_parts to "yes"
    And measure 0's Melody notes are "1 2 3 4"
    And measure 0's Verse 2 line has syllables "la la la la"
    And measure 0 has no Verse 1 line
    When the blank-lyric-rows score is laid out
    Then the row under Melody is labelled "V2"
