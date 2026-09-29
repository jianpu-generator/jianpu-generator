Feature: Section header CodeLens links
  The Edit Parts / Edit Metadata links appear exactly on the lines the parser
  treats as `# parts` / `# metadata` section headers.

  Scenario: A header followed by a comment still gets its link
    Given the section-header CodeLens fixture with commented headers is loaded
    Then the editor shows 1 "Edit Parts" CodeLens link
    And the editor shows 1 "Edit Metadata" CodeLens link
    When I click the "Edit Parts" CodeLens link
    Then the edit parts modal contains "Melody"

  Scenario: An indented header-like line gets no link
    Given the section-header CodeLens fixture with an indented header-like line is loaded
    Then the editor shows 1 "Edit Parts" CodeLens link
