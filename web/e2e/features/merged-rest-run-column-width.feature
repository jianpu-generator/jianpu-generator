Feature: Merged rest run width in column distribution
  A run of consecutive all-rest measures collapses into one
  `MultiMeasureRest` block (see "Multi-measure rests" in syntax.md), drawn
  as a horizontal bar with the run's collapsed measure count centered above
  it. A system's width is split among its measures by weight (see "Rod and
  spring" in ARCHITECTURE.md); a merged rest block's own weight and
  minimum-width floor are flat constants, so they never grow to match how
  wide its count label actually renders. When a multi-digit count shares a
  system with dense measures competing for the same width, the label can
  render wider than the space the layout actually reserved for it.

  Background:
    Given parts Melody [M] are declared

  Scenario: A merged rest run's count label fits within its own bar even when squeezed by dense neighboring measures
    Given the score's max_measures_per_system is 3
    And measure 0 is a dense measure of 16 sixteenth notes
    And measures 1 to 120 are all-rest, merging into one run
    And measure 121 is a dense measure of 16 sixteenth notes
    When the merged-rest-run-column-width score is laid out
    Then the merged rest run shows the count "120"
    And the count label's rendered width is no wider than the merged rest bar's rendered width
    And the count label stays within the merged rest measure's own click-target bounds
    And the merged rest bar keeps horizontal padding from the measure dividers on both sides
