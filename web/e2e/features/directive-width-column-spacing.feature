Feature: Directive width affects measure column spacing

  Directives (bar number, label, key, tempo, time signature) drawn above a
  measure have their own rendered width, which can exceed the width of the
  measure's notes. Column width distribution must reserve enough space for
  the wider of the two, so a measure's directives never overflow into the
  next measure's column and collide with its directives.

  Background:
    Given the directive-width-overflow test fixture is loaded

  Scenario: A measure's directives are wider than its notes
    Then the first measure's directive line does not overlap the second measure's directive line

  Scenario: Every measure in a system carries its own directive line
    Given the two-adjacent-directives test fixture is loaded
    Then no measure's directive line overlaps the next measure's directive line
