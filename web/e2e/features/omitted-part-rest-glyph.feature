Feature: Omitted-part rest uses a distinct glyph from a written rest
  A part not mentioned in a measure is filled with rests (see "Not-mentioned
  parts" in syntax.md). Today that fill renders as an ordinary "0" rest
  glyph, identical to a rest the composer actually typed. That makes it look
  like the part was deliberately silenced for the bar, when really it just
  wasn't written for. This filled-in rest should instead render as an
  inverted-hat placeholder glyph, visually distinct from "0", so a reader can
  tell "not written" apart from "written as silent" at a glance.

  # Implementation note: the glyph must be drawn with SVG line/path
  # primitives (the way `render_multi_measure_rest` draws its bar), not as
  # a Unicode text character (the way `render_rest` draws "0"). A Unicode
  # rest symbol like U+1D13B depends on the viewer having a font that
  # covers that block, which isn't reliably true.

  Background:
    Given parts Melody [M], Harmony [H] are declared

  Scenario: An omitted part's filled rest renders as the inverted-hat glyph, not "0"
    Given hide_resting_parts is "no"
    And measure 0's Melody line has notes "1 2 3 4"
    And measure 0 has no Harmony line
    When the omitted-part-rest-glyph score is laid out
    Then the Harmony row in measure 0 shows the inverted-hat placeholder glyph
    And the Harmony row in measure 0 does not show a "0" rest glyph

  Scenario: A part explicitly written as resting still renders as an ordinary "0"
    Given hide_resting_parts is "no"
    And measure 0's Melody line has notes "1 2 3 4"
    And measure 0's Harmony line has notes "0 0 0 0"
    When the omitted-part-rest-glyph score is laid out
    Then the Harmony row in measure 0 shows a "0" rest glyph
    And the Harmony row in measure 0 does not show the inverted-hat placeholder glyph
