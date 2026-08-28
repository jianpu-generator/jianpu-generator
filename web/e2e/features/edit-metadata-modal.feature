Feature: Edit Metadata modal

  Scenario: CodeLens Edit Metadata link opens the modal
    Given the edit-metadata-modal test fixture is loaded
    When I open the Edit Metadata modal
    Then the edit metadata modal contains "Edit Metadata"
    And the first text input in the metadata modal has value "Test"

  Scenario: Editing the title field updates the source
    Given the edit-metadata-modal test fixture is loaded
    When I open the Edit Metadata modal
    And I fill the title field with "New Title"
    And I close the metadata modal with Escape
    Then the editor source and stored source both contain "title = \"New Title\""

  Scenario: Editing a numeric field updates the source
    Given the edit-metadata-modal test fixture is loaded
    When I open the Edit Metadata modal
    And I fill the Row Height numeric field with "30"
    And I close the metadata modal with Escape
    Then the editor source and stored source both contain "row_height = 30"

  Scenario: Clearing an optional field removes it from the source
    Given the edit-metadata-modal test fixture is loaded
    When I open the Edit Metadata modal
    And I clear the second text input in the metadata modal
    And I close the metadata modal with Escape
    Then the editor source and stored source no longer contain "subtitle"

  Scenario: Unchecking merge_duplicate_measures_across_parts writes = no to the source
    Given the edit-metadata-modal test fixture is loaded
    When I open the Edit Metadata modal
    And I uncheck the merge_duplicate_measures_across_parts checkbox
    And I close the metadata modal with Escape
    Then the editor source and stored source both contain "merge_duplicate_measures_across_parts = no"

  Scenario: Re-checking merge_duplicate_measures_across_parts writes = yes to the source
    Given the edit-metadata-modal test fixture is loaded
    When I open the Edit Metadata modal
    And I uncheck then re-check the merge_duplicate_measures_across_parts checkbox
    And I close the metadata modal with Escape
    Then the editor source and stored source both contain "merge_duplicate_measures_across_parts = yes"

  Scenario: Unchecking hide_resting_parts writes = no to the source
    Given the edit-metadata-modal test fixture is loaded
    When I open the Edit Metadata modal
    And I uncheck the hide_resting_parts checkbox
    And I close the metadata modal with Escape
    Then the editor source and stored source both contain "hide_resting_parts = no"

  Scenario: Re-checking hide_resting_parts writes = yes to the source
    Given the edit-metadata-modal test fixture is loaded
    When I open the Edit Metadata modal
    And I uncheck then re-check the hide_resting_parts checkbox
    And I close the metadata modal with Escape
    Then the editor source and stored source both contain "hide_resting_parts = yes"

  Scenario: Checking hide_system_dividers writes = yes to the source
    Given the edit-metadata-modal test fixture is loaded
    When I open the Edit Metadata modal
    And I check the hide_system_dividers checkbox
    And I close the metadata modal with Escape
    Then the editor source and stored source both contain "hide_system_dividers = yes"

  Scenario: Unchecking hide_system_dividers writes = no to the source
    Given the edit-metadata-modal test fixture is loaded
    When I open the Edit Metadata modal
    And I check then uncheck the hide_system_dividers checkbox
    And I close the metadata modal with Escape
    Then the editor source and stored source both contain "hide_system_dividers = no"

  Scenario: Editing part_label_width_pt updates the source
    Given the edit-metadata-modal test fixture is loaded
    When I open the Edit Metadata modal
    And I fill the Part Label Width (pt) numeric field with "60"
    And I close the metadata modal with Escape
    Then the editor source and stored source both contain "part_label_width_pt = 60"

  Scenario: Editing measure_number_font_size updates the source
    Given the edit-metadata-modal test fixture is loaded
    When I open the Edit Metadata modal
    And I fill the Measure Number Font Size numeric field with "14"
    And I close the metadata modal with Escape
    Then the editor source and stored source both contain "measure_number_font_size = 14"

  Scenario: Editing section_label_font_size updates the source
    Given the edit-metadata-modal test fixture is loaded
    When I open the Edit Metadata modal
    And I fill the Section Label Font Size numeric field with "16"
    And I close the metadata modal with Escape
    Then the editor source and stored source both contain "section_label_font_size = 16"

  Scenario: Editing part_label_font_size updates the source
    Given the edit-metadata-modal test fixture is loaded
    When I open the Edit Metadata modal
    And I fill the Part Label Font Size numeric field with "18"
    And I close the metadata modal with Escape
    Then the editor source and stored source both contain "part_label_font_size = 18"

  Scenario: Editing page_number_font_size updates the source
    Given the edit-metadata-modal test fixture is loaded
    When I open the Edit Metadata modal
    And I fill the Page Number Font Size numeric field with "20"
    And I close the metadata modal with Escape
    Then the editor source and stored source both contain "page_number_font_size = 20"

  Scenario: Editing directive_row_offset writes "x y" to the source
    Given the edit-metadata-modal test fixture is loaded
    When I open the Edit Metadata modal
    And I fill the directive_row_offset field with "0 12"
    And I close the metadata modal with Escape
    Then the editor source and stored source both contain "directive_row_offset = 0 12"

  Scenario: Modal stays within the editor pane and does not cover the preview pane
    Given the edit-metadata-modal test fixture is loaded with viewport 1400 by 900
    When I open the Edit Metadata modal
    Then the metadata modal stays within the editor pane and does not cover the preview pane

  Scenario: Preview pane stays scrollable while the modal is open
    Given a jianpu score with 40 measures and row_height 200 is loaded, and the viewport is 1400 by 900
    Then the preview pane is scrollable
    When I open the Edit Metadata modal
    And I hover the preview pane and scroll the mouse wheel down by 400
    Then the preview pane scroll position is greater than 0
