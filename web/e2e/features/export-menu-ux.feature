Feature: Export menu UX

  Scenario: Export menu has no "All Parts" section when the score has only one part
    Given the single-part export menu source is loaded
    When I open the export menu, as seen in export menu ux
    Then the export menu is visible
    And the export menu has no "All Parts" section
    And the export menu has no "PDF (ZIP)" item

  Scenario: Export menu lists PDF, WAV, MIDI, and MP3 for a single-part score
    Given the single-part export menu source is loaded
    When I open the export menu, as seen in export menu ux
    Then the export menu is visible
    And the export menu items are exactly "PDF, WAV, MIDI, MP3"

  Scenario: Export menu lists Visible Parts and All Parts sections for a multi-part score
    Given the multi-part export menu source is loaded
    When I open the export menu, as seen in export menu ux
    Then the export menu is visible
    And the export menu shows a "Visible Parts" section
    And the export menu shows an "All Parts" section
    And the export menu items are exactly "PDF, WAV, MIDI, MP3, PDF (ZIP), WAV (ZIP), MIDI (ZIP), MP3 (ZIP)"

  Scenario: Pressing Escape closes an open export menu
    Given the single-part export menu source is loaded
    When I open the export menu, as seen in export menu ux
    Then the export menu is visible
    When I press Escape
    Then the export menu is closed and the button is collapsed

  Scenario: Clicking outside an open export menu closes it
    Given the single-part export menu source is loaded
    When I open the export menu, as seen in export menu ux
    Then the export menu is visible
    When I click outside the export menu on the preview pages
    Then the export menu is closed and the button is collapsed
