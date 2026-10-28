Feature: Presets

  # The fixture has a "Melody only" preset (visible_parts = M, Mv1), a
  # "Violin" preset (notation = violin) and the parts Melody [M], Melody
  # lyrics [Mv1] and Chords [C].

  Scenario: Choosing a preset shows only its parts
    Given the presets fixture is loaded
    Then the preset menu shows "Default"
    And the preview contains the chord content
    When I choose the "Melody only" preset
    Then the preset menu shows "Melody only"
    And the "C" part pill is off
    And the "M" part pill is on
    And the preview does not contain the chord content

  Scenario: Turning a pill off under a preset rewrites visible_parts, and undo restores it
    Given the presets fixture is loaded
    When I choose the "Melody only" preset
    And I turn the "Mv1" part pill off
    Then the "Melody only" preset has the line "visible_parts = M"
    And the stored file's "Melody only" preset has the line "visible_parts = M"
    And the "Mv1" part pill is off
    When I press undo in the main editor
    Then the "Melody only" preset has the line "visible_parts = M, Mv1"
    And the "Mv1" part pill is on

  Scenario: On Default, pills stay in this browser and the file doesn't change
    Given the presets fixture is loaded
    When I turn the "C" part pill off
    Then the "C" part pill is off
    And the preview does not contain the chord content
    And the stored source is the presets fixture

  Scenario: The chosen preset survives a reload
    Given the presets fixture is loaded
    When I choose the "Melody only" preset
    And I reload the presets page
    Then the preset menu shows "Melody only"
    And the "C" part pill is off

  Scenario: Renaming the chosen preset falls back to Default
    Given the presets fixture is loaded
    When I choose the "Melody only" preset
    And I replace "Melody only" with "Tune" in the editor
    Then the preset menu shows "Default"
    And the "C" part pill is on
    And the preview contains the chord content

  Scenario: New preset adds a section with the parts shown now and chooses it
    Given the presets fixture is loaded
    When I turn the "Mv1" part pill off
    And I create a new preset called "No lyrics"
    Then the "No lyrics" preset has the line "visible_parts = M, C"
    And the "No lyrics" preset has the line "notation = jianpu"
    And the preset menu shows "No lyrics"
    And the "Mv1" part pill is off

  Scenario: A new preset can't take a name already in use
    Given the presets fixture is loaded
    When I open the new preset dialog
    And I type "Melody only" as the preset name
    Then the new preset dialog says "A preset is already called Melody only."
    And the new preset dialog's Create button is disabled

  Scenario: Edit in source moves the cursor to the preset's header
    Given the presets fixture is loaded
    When I choose the "Melody only" preset
    And I choose "Edit in source" in the preset menu
    Then the editor cursor is on the "Melody only" preset's header line

  Scenario: A preset's notation line sets the preview's notation
    Given the presets fixture is loaded
    When I choose the "Violin" preset
    Then the preview shows a staff notation page
    When I choose "Default" in the preset menu
    Then the preview shows jianpu pages

  Scenario: Picking a notation under a preset rewrites its notation line
    Given the presets fixture is loaded
    When I choose the "Melody only" preset
    And I switch the preview notation to "cello"
    Then the "Melody only" preset has the line "notation = cello"
    And the preview shows a staff notation page

  Scenario: Export > PDF under a preset is named after it
    Given the presets fixture is loaded
    When I choose the "Melody only" preset
    And I export "PDF" and capture the download, as seen in export pdf
    Then the downloaded file is named "test (Melody only).pdf", as seen in export pdf

  Scenario: Export Parts > PDF (ZIP) under a preset holds only its parts
    Given the presets fixture is loaded
    When I choose the "Melody only" preset
    And I export "PDF (ZIP)" and capture the download, as seen in export pdf
    Then the downloaded file is named "test (Melody only).zip", as seen in export pdf
    And the downloaded zip holds only "test (Melody only) - Melody.pdf"
