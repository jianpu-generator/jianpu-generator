Feature: A row merging two groups' broadcasts collapses each group's own label
  When a single row folds together members from *more than one* `# groups`
  broadcast (because every member's compiled content for that measure happens
  to be identical), each group's own members must still collapse to that
  group's abbreviation (see "Row label when members render as one unison
  row" in syntax.md) before being joined with the other group's label. The
  row label must not fall back to listing one group's members by their
  individual part abbreviations just because a second group's members are
  also folded into the same row.

  Background:
    Given parts Soprano 1 [S1], Soprano 2 [S2], Alto 1 [A1], Alto 2 [A2], Tenor [T] are declared in that order
    And groups Soprano [S] = S1 S2 and Alto [A] = A1 A2 are declared

  Scenario: Two groups' broadcasts and an outside part all merge into one row
    Given every measure's [S], [A], and [T] lines give all five parts the same notes
    When the multiple-groups score is laid out
    Then the merged row's part label reads "S A T"
