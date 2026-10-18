Feature: Lyrics under a chord part
  A lyrics part can target a chords part (`l = lyrics[c]`). The lyric line
  carries its own timing: each chord attack pairs with one bracketed item,
  and unbracketed syllables flow freely between them (see "Lyrics under a
  chord part" in syntax.md).

  # These scenarios fail until the feature is implemented (tasks 3-6 of
  # HANDOFF-chord-lyrics.md).

  Scenario: Brackets pair with chord attacks and the chord sits above its bracket
    Given a chord-lyrics score with chords "1 -- 7" and lyrics "(0_) 只剩下鋼琴 （陪）我"
    When the chord-lyrics score is rendered
    Then the chord-lyrics score has no error
    And the chord-lyrics preview shows "(0)"
    And the chord-lyrics preview does not show "(0_)"
    And the chord-lyrics preview draws 1 rest glyph with 1 underline
    And the chord-lyrics preview shows "(陪)"
    And the chord "7" is drawn above the bracket "(陪)"

  Scenario: Full-width brackets are drawn as ASCII brackets
    Given a chord-lyrics score with chords "1 -- 7" and lyrics "(0_) 只剩下鋼琴 （陪）我"
    When the chord-lyrics score is rendered
    Then the chord-lyrics preview shows "(陪)"
    And the chord-lyrics preview does not show "（"
    And the chord-lyrics preview does not show "）"

  Scenario: A pickup run before the first bracket is allowed when the first chord attack is not on beat 1
    Given a chord-lyrics score with chords "0 1 - -" and lyrics "我 (愛) 你"
    When the chord-lyrics score is rendered
    Then the chord-lyrics score has no error

  Scenario: A bracket count that differs from the chord attack count is an error
    Given a chord-lyrics score with chords "1 - 4 -" and lyrics "(啦) 啦啦"
    When the chord-lyrics score is rendered
    Then the chord-lyrics score has an error

  Scenario: Several syllables in one bracket is an error
    Given a chord-lyrics score with chords "1 - - -" and lyrics "（陪我）"
    When the chord-lyrics score is rendered
    Then the chord-lyrics score has an error

  Scenario: An unbracketed run before a beat-1 chord attack is an error
    Given a chord-lyrics score with chords "1 - - -" and lyrics "啦 (啦)"
    When the chord-lyrics score is rendered
    Then the chord-lyrics score has an error

  Scenario: A chord rest does not count as an attack
    Given a chord-lyrics score with chords "1 0 4 -" and lyrics "(a) (b) (c)"
    When the chord-lyrics score is rendered
    Then the chord-lyrics score has an error

  Scenario: Two brackets match two attacks around a chord rest
    Given a chord-lyrics score with chords "1 0 4 -" and lyrics "(a) (b)"
    When the chord-lyrics score is rendered
    Then the chord-lyrics score has no error
