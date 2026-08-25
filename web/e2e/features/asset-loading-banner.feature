Feature: Asset loading banner

  Scenario: Shows soundfont, fonts, and wasm progress, then hides once all are ready
    Given delayed asset loading routes for soundfont, fonts, and wasm
    When the app loads with delayed asset routes
    Then the soundfont, fonts, and wasm rows are all visible
    And the fonts and wasm rows disappear before the soundfont row
    And the soundfont row disappears once it finishes loading
    And the Monaco editor view-lines become visible

  Scenario: Shows an error state when an asset fails to load
    Given the soundfont asset route is aborted
    When the app loads with the soundfont route aborted
    Then the soundfont row shows an error state
    And the fonts and wasm rows disappear while only the soundfont row remains
