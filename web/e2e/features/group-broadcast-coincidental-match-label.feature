Feature: A coincidental content match doesn't widen a group's row label
  When two members of a `# groups` entry receive identical notes via a
  `[GroupAbbrev]` broadcast, their rows merge into one row labeled with the
  group's own abbreviation (see "Row label when members render as one
  unison row" in syntax.md). A part outside the group can happen to have
  the same notes as that merged row in one measure of the system. That
  coincidence must not widen the group's label into a concatenation of both
  parts' abbreviations: the outside part still gets its own persistent row
  for the whole system once any other measure tells the two apart, so the
  group's row should keep reading as just the group's abbreviation.

  Background:
    Given parts Soprano 1 [S1], Soprano 2 [S2], Tenor [T] are declared in that order
    And group Soprano [S] = S1 S2

  Scenario: Tenor coincidentally matches the group's broadcast only in the system's first measure
    Given measure 0's [S] broadcast gives S1 and S2 the same notes as Tenor's own line
    And measure 1's [S] broadcast gives S1 and S2 different notes from Tenor's own line
    When the group-broadcast score is laid out
    Then Soprano's part label reads "S"
    And Tenor's part label spans measures 0 to 1 across the system
    And measure 0's Tenor row shows a real note glyph
