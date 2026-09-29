Feature: Edit Parts modal with out-of-range part settings

  Scenario: An out-of-range volume and octave offset are reported and shown clamped
    Given the edit-parts-modal test fixture with an out-of-range volume and octave offset is loaded
    Then the editor shows the diagnostic "volume 150% is out of range; valid range is 0% to 100%; clamped to 100%"
    And the editor shows the diagnostic "octave offset +5 is out of range; valid range is -4 to +4; clamped to +4"
    When I open the Edit Parts modal, as seen in edit parts modal
    Then the volume value for part "M" shows "100%"
    And the octave select for part "M" shows "+4"
