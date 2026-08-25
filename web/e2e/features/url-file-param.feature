Feature: The ?file= URL param tracks and selects the active file

  Scenario: Switching the active file updates the ?file= URL param
    Given local files "a.jianpu" and "b.jianpu" are seeded for the URL param test
    When the app loads at the root URL
    Then the URL has the "file" param set to "a.jianpu"
    When I open the file list and select "b.jianpu"
    Then the file switcher trigger shows "b.jianpu"
    And the URL has the "file" param set to "b.jianpu"

  Scenario: Loading with a ?file= URL param selects that file
    Given local files "a.jianpu" and "b.jianpu" are seeded for the URL param test
    When the app loads with the URL param "file=b.jianpu"
    Then the file switcher trigger shows "b.jianpu"
    And the active tab in the file list is named "b.jianpu"

  Scenario: Loading with a non-ASCII ?file= URL param selects that file without reverting
    Given local files "a.jianpu" and "今山古道.jianpu" are seeded for the URL param test
    When the app loads with the URL param naming "今山古道.jianpu"
    Then the file switcher trigger shows "今山古道.jianpu"
    And the active tab in the file list is named "今山古道.jianpu"
    And the URL's "file" param still names "今山古道.jianpu"
