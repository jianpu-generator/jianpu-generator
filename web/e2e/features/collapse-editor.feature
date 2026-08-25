Feature: Collapse editor

  Background:
    Given the collapse-editor test fixture is loaded

  Scenario: Hides the editor pane and expands the preview when toggled
    Then the editor pane is expanded with nonzero width
    When I click the pane-divider toggle button
    Then the editor pane is collapsed
    And the pane-divider toggle button title is "Show editor"
    And the editor pane width shrinks to less than 2 pixels

  Scenario: Restores the editor pane when toggled again
    When I click the pane-divider toggle button
    Then the editor pane is collapsed
    When I click the pane-divider toggle button
    Then the editor pane is expanded
    And the pane-divider toggle button title is "Hide editor"
    And the editor pane width grows to more than 50 pixels
    And the Monaco editor is visible
