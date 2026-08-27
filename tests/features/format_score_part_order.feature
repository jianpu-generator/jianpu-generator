Feature: Format score sorts parts within each measure by declaration order

  Scenario: Reorders a measure group's out-of-order part lines to match declaration order
    Given the score source:
      """
      # parts
      Melody = notes
      Bass = notes

      # score
      [Bass] 5 6 7 1
      [Melody] 1 2 3 4
      """
    When it is formatted
    Then the formatted source is:
      """
      # parts
      Melody = notes
      Bass = notes

      # score
      [Melody] 1 2 3 4
      [Bass] 5 6 7 1
      """

  Scenario: Sorts each measure group independently
    Given the score source:
      """
      # parts
      Melody = notes
      Bass = notes
      Drums = percussion

      # score
      [Bass] 5 6 7 1
      [Drums] x x x x
      [Melody] 1 2 3 4

      [Bass] 2 2 2 2
      [Melody] 5 5 5 5
      """
    When it is formatted
    Then the formatted source is:
      """
      # parts
      Melody = notes
      Bass = notes
      Drums = percussion

      # score
      [Melody] 1 2 3 4
      [Bass] 5 6 7 1
      [Drums] x x x x

      [Melody] 5 5 5 5
      [Bass] 2 2 2 2
      """

  Scenario: Moves a key's multiple explicitly-prefixed lines together as a contiguous block, preserving their relative order
    Given the score source:
      """
      # parts
      Melody = notes+lyrics
      Bass = notes

      # score
      [Melody] 1 2 3 4
      [Bass] 5 6 7 1
      [Melody] la la la la
      """
    When it is formatted
    Then the formatted source is:
      """
      # parts
      Melody = notes+lyrics
      Bass = notes

      # score
      [Melody] 1 2 3 4
      [Melody] la la la la
      [Bass] 5 6 7 1
      """

  Scenario: A positional lyrics line moves with its part's note line
    Given the score source:
      """
      # parts
      Melody = notes
      Bass = notes

      # score
      [Bass] 5 6 7 1
      [Melody] 1 2 3 4
      la la la la
      """
    When it is formatted
    Then the formatted source is:
      """
      # parts
      Melody = notes
      Bass = notes

      # score
      [Melody] 1 2 3 4
      la la la la
      [Bass] 5 6 7 1
      """

  Scenario: Multiple consecutive positional verse lines stay together, in order, with their part
    Given the score source:
      """
      # parts
      Melody = notes
      Bass = notes

      # score
      [Bass] 5 6 7 1
      [Melody] 1 2 3 4
      la la la la
      na na na na
      """
    When it is formatted
    Then the formatted source is:
      """
      # parts
      Melody = notes
      Bass = notes

      # score
      [Melody] 1 2 3 4
      la la la la
      na na na na
      [Bass] 5 6 7 1
      """

  Scenario: Each part's own positional lyrics line follows its nearest preceding [Key] line, even after both parts reorder
    Given the score source:
      """
      # parts
      Melody = notes
      Bass = notes

      # score
      [Bass] 5 6 7 1
      bass words
      [Melody] 1 2 3 4
      melody words
      """
    When it is formatted
    Then the formatted source is:
      """
      # parts
      Melody = notes
      Bass = notes

      # score
      [Melody] 1 2 3 4
      melody words
      [Bass] 5 6 7 1
      bass words
      """

  Scenario: A standalone-caption bare line (no preceding [Key] line) is not resolved to its target part's position — it stays in the unattributable-line fallback, sorted after every recognised part
    Given the score source:
      """
      # parts
      Caption = lyrics
      Alto = notes
      Tenor = notes

      # score
      a caption for this measure unrelated to any note
      [Tenor] 5 6 7 1
      [Alto] 1 2 3 4
      """
    When it is formatted
    Then the formatted source is:
      """
      # parts
      Caption = lyrics
      Alto = notes
      Tenor = notes

      # score
      [Alto] 1 2 3 4
      [Tenor] 5 6 7 1
      a caption for this measure unrelated to any note
      """
