Feature: Part hide/solo/lyrics toggles

  # The "M — Melody" / "C — Chords" legend in the preview header lists only
  # parts currently enabled by hide/solo state — a hidden or non-soloed part's
  # entry disappears from the legend along with its score content.

  Scenario: Hiding a part removes its content from the preview and hides its lyrics toggle
    Given the two-part melody-chords fixture is loaded
    Then the preview contains "Melody" "lyrics"
    And the preview contains the chord content
    When I hide the "Melody" part via its eye toggle, as seen in part toggles
    Then the preview does not contain "Melody" "notes"
    And the preview does not contain "Melody" "lyrics"
    And the "Melody" part pill has no mic toggle
    And the preview contains the chord content

  Scenario: Unhiding a part restores its content in the preview
    Given the two-part melody-chords fixture is loaded
    When I hide the "Melody" part via its eye toggle, as seen in part toggles
    Then the preview does not contain "Melody" "notes"
    When I hide the "Melody" part via its eye toggle, as seen in part toggles
    Then the preview contains "Melody" "notes"
    And the preview contains "Melody" "lyrics"
    And the "Melody" part pill has a mic toggle

  Scenario: Soloing a part hides all other parts in the preview
    Given the two-part melody-chords fixture is loaded
    Then the preview contains the chord content
    When I solo the "Melody" part
    Then the preview contains "Melody" "notes"
    And the preview contains "Melody" "lyrics"
    And the preview does not contain the chord content

  Scenario: Un-soloing restores previously enabled parts
    Given the two-part melody-chords fixture is loaded
    When I solo the "Melody" part
    Then the preview does not contain the chord content
    When I solo the "Melody" part
    Then the preview contains "Melody" "notes"
    And the preview contains the chord content

  Scenario: Soloing multiple parts keeps both visible and hides the rest
    Given the three-part melody-harmony-chords fixture is loaded
    Then the preview contains "Harmony" "notes"
    And the preview contains the chord content
    When I solo the "Harmony" part
    And I solo the "Chords" part
    Then the preview does not contain "Melody" "notes"
    And the preview does not contain "Melody" "lyrics"
    And the preview contains "Harmony" "notes"
    And the preview contains the chord content

  Scenario: Toggling lyrics off removes lyric text but keeps notes rendered
    Given the two-part melody-chords fixture is loaded
    Then the preview contains "Melody" "lyrics"
    When I toggle the "Melody" part's lyrics off
    Then the preview does not contain "Melody" "lyrics"
    And the preview contains "Melody" "notes"

  Scenario: Toggling lyrics back on restores lyric text
    Given the two-part melody-chords fixture is loaded
    When I toggle the "Melody" part's lyrics off
    Then the preview does not contain "Melody" "lyrics"
    When I toggle the "Melody" part's lyrics off
    Then the preview contains "Melody" "lyrics"

  Scenario: Hiding a part removes its entry from the part-list legend
    Given the two-part melody-chords fixture is loaded
    Then the preview contains the "Melody" legend entry
    And the preview contains the "Chords" legend entry
    When I hide the "Melody" part via its eye toggle, as seen in part toggles
    Then the preview does not contain the "Melody" legend entry
    And the preview contains the "Chords" legend entry

  Scenario: Unhiding a part restores its part-list legend entry
    Given the two-part melody-chords fixture is loaded
    When I hide the "Melody" part via its eye toggle, as seen in part toggles
    Then the preview does not contain the "Melody" legend entry
    When I hide the "Melody" part via its eye toggle, as seen in part toggles
    Then the preview contains the "Melody" legend entry

  Scenario: Soloing a part hides other parts legend entries
    Given the two-part melody-chords fixture is loaded
    Then the preview contains the "Chords" legend entry
    When I solo the "Melody" part
    Then the preview contains the "Melody" legend entry
    And the preview does not contain the "Chords" legend entry

  Scenario: Un-soloing restores previously enabled parts legend entries
    Given the two-part melody-chords fixture is loaded
    When I solo the "Melody" part
    Then the preview does not contain the "Chords" legend entry
    When I solo the "Melody" part
    Then the preview contains the "Melody" legend entry
    And the preview contains the "Chords" legend entry

  Scenario: Soloing multiple parts keeps both legend entries and hides the rest
    Given the three-part melody-harmony-chords fixture is loaded
    Then the preview contains the "Harmony" legend entry
    And the preview contains the "Chords" legend entry
    When I solo the "Harmony" part
    And I solo the "Chords" part
    Then the preview does not contain the "Melody" legend entry
    And the preview contains the "Harmony" legend entry
    And the preview contains the "Chords" legend entry
