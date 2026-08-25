Feature: Cmd+Enter play shortcut

  Scenario: Meta+Enter does nothing when cursor is outside all measures
    Given the app is loaded and the editor is focused
    When I jump to line 1 via Ctrl+g
    Then the play-measure button is disabled
    When I press Meta+Enter
    Then the play-measure button does not enter the playing state
