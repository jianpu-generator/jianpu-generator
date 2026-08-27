# Positional (unprefixed) lyrics attachment — see the design discussion this
# encodes: a lyric line no longer needs a `[Key]` prefix. A bare line right
# after a part's notes line attaches to that part (notes-following); a bare
# line with no part line above it yet in the measure is a standalone,
# adurational lyrics block instead.
#
# `notes+lyrics` / `lyrics` PartKinds are kept for backward compatibility
# during this migration (see the `standalone lyrics row` scenarios, which
# still declare a `= lyrics` part) and are expected to be phased out once
# this feature is complete.
#
# NOTE: this syntax is not implemented yet. Every scenario below is expected
# to FAIL until the parser/desugar work lands.

Feature: Positional lyrics attachment

  Scenario: A bare line right after a part's notes attaches to that part
    Given the score source:
      """
      # metadata
      title = "t"
      author = "a"

      # parts
      Alto = notes

      # score
      time=4/4 key=C4 bpm=120
      [Alto] 3213
      Ma rry had a
      """
    When it is compiled
    Then part "Alto" measure 1 has 4 note events
    And part "Alto" measure 1 has 1 lyric verse
    And part "Alto" measure 1 verse 1 has syllables "Ma, rry, had, a"

  Scenario: A held syllable on an attached line still stretches across notes
    Given the score source:
      """
      # metadata
      title = "t"
      author = "a"

      # parts
      Alto = notes

      # score
      time=4/4 key=C4 bpm=120
      [Alto] 1 1 2 3
      Glo - ry be
      """
    When it is compiled
    Then part "Alto" measure 1 verse 1 has syllables "Glo, -, ry, be"

  Scenario: A bare no-lyrics marker on an attached line means zero syllables
    Given the score source:
      """
      # metadata
      title = "t"
      author = "a"

      # parts
      Alto = notes

      # score
      time=4/4 key=C4 bpm=120
      [Alto] 1 2 3 4
      _
      """
    When it is compiled
    Then part "Alto" measure 1 has 1 lyric verse
    And part "Alto" measure 1 verse 1 has 0 syllables

  Scenario: A bare line after two part lines attaches to the nearer one only
    Given the score source:
      """
      # metadata
      title = "t"
      author = "a"

      # parts
      Alto = notes
      Tenor = notes

      # score
      time=4/4 key=C4 bpm=120
      [Alto]333
      [Tenor] 111
      Li ttle lamb
      """
    When it is compiled
    Then part "Tenor" measure 1 has 1 lyric verse
    And part "Tenor" measure 1 verse 1 has syllables "Li, ttle, lamb"
    And part "Alto" measure 1 has 0 lyric verses

  Scenario: Duplicating the line attaches it to both parts explicitly
    Given the score source:
      """
      # metadata
      title = "t"
      author = "a"

      # parts
      Alto = notes
      Tenor = notes

      # score
      time=4/4 key=C4 bpm=120
      [Alto]333
      Li ttle lamb
      [Tenor] 111
      Li ttle lamb
      """
    When it is compiled
    Then part "Alto" measure 1 verse 1 has syllables "Li, ttle, lamb"
    And part "Tenor" measure 1 verse 1 has syllables "Li, ttle, lamb"

  Scenario: Consecutive bare lines after one part's notes become verses 1, 2, ...
    Given the score source:
      """
      # metadata
      title = "t"
      author = "a"

      # parts
      Melody = notes

      # score
      time=4/4 key=C4 bpm=120
      [Melody] 1 2 3 4
      a b c d
      one two three four
      """
    When it is compiled
    Then part "Melody" measure 1 has 2 lyric verses
    And part "Melody" measure 1 verse 1 has syllables "a, b, c, d"
    And part "Melody" measure 1 verse 2 has syllables "one, two, three, four"

  Scenario: A bare line with no part line above it is a standalone lyrics row
    Given the score source:
      """
      # metadata
      title = "t"
      author = "a"

      # parts
      Caption = lyrics
      Alto = notes
      Tenor = notes

      # score
      time=4/4 key=C4 bpm=120
      a caption for this measure unrelated to any note
      [Alto] 1 2 3 4
      [Tenor] 5 6 7 1
      """
    When it is compiled
    Then part "Caption" measure 1 has 0 note events
    And part "Caption" measure 1 has 1 lyric verse
    And part "Caption" measure 1 verse 1 has syllables "a, caption, for, this, measure, unrelated, to, any, note"

  Scenario: A standalone lyrics row still supports multiple verses
    Given the score source:
      """
      # metadata
      title = "t"
      author = "a"

      # parts
      Caption = lyrics
      Alto = notes

      # score
      time=4/4 key=C4 bpm=120
      verse one caption
      verse two caption
      [Alto] 1 2 3 4
      """
    When it is compiled
    Then part "Caption" measure 1 has 2 lyric verses
    And part "Caption" measure 1 verse 1 has syllables "verse, one, caption"
    And part "Caption" measure 1 verse 2 has syllables "verse, two, caption"

  Scenario: A lyric line trailing different parts across measures gives two separate rows
    Given the score source:
      """
      # metadata
      title = "t"
      author = "a"

      # parts
      Alto = notes
      Tenor = notes

      # score
      time=4/4 key=C4 bpm=120
      [Alto] 3213
      Ma rry had a

      [Alto]333
      [Tenor] 111
      Li ttle lamb
      """
    When it is compiled
    Then part "Alto" measure 1 verse 1 has syllables "Ma, rry, had, a"
    And part "Tenor" measure 1 has 0 lyric verses
    And part "Tenor" measure 2 verse 1 has syllables "Li, ttle, lamb"
    And part "Alto" measure 2 has 0 lyric verses
    # No smart unification across measures: if the composer wants one
    # continuous row, they write it themselves (e.g. add `[Tenor] 0` to
    # measure 1 so the Tenor row exists in both measures). That row-merging
    # behavior is a grid_layout/system-packing concern, out of scope here.

  Scenario: Two parts can each carry their own simultaneous lyric row
    Given the score source:
      """
      # metadata
      title = "t"
      author = "a"

      # parts
      Soprano = notes
      Tenor = notes

      # score
      time=4/4 key=C4 bpm=120
      [Soprano] 1 2 3 4
      la la la la
      [Tenor] 1 2 3 4
      bum bum bum bum
      """
    When it is compiled
    Then part "Soprano" measure 1 verse 1 has syllables "la, la, la, la"
    And part "Tenor" measure 1 verse 1 has syllables "bum, bum, bum, bum"

  Scenario: A plain `notes` part accepts a trailing bare line with no special declaration
    Given the score source:
      """
      # metadata
      title = "t"
      author = "a"

      # parts
      Melody = notes

      # score
      time=4/4 key=C4 bpm=120
      [Melody] 1 2 3 4
      la la la la
      """
    When it is compiled
    Then part "Melody" measure 1 has 1 lyric verse
    And part "Melody" measure 1 verse 1 has syllables "la, la, la, la"

  Scenario: A composer's forgotten `[Key]` prefix on a notes line is not an error
    Given the score source:
      """
      # metadata
      title = "t"
      author = "a"

      # parts
      Alto = notes
      Tenor = notes

      # score
      time=4/4 key=C4 bpm=120
      [Alto] 1 2 3 4
      2 3 4 5
      """
    When it is compiled
    Then measure 1 has no diagnostics
    And part "Alto" measure 1 has 1 lyric verse
    And part "Alto" measure 1 verse 1 has syllables "2, 3, 4, 5"
    # This documents an accepted regression: today, a data line missing its
    # `[Key]` prefix is a hard, reported error. Under positional lyrics, any
    # bare line is always valid syntax (as lyrics), so a composer's typo
    # (forgetting `[Tenor]` here) is silently absorbed instead of caught.
