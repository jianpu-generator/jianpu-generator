Feature: File switcher loading spinner for the GitHub storage backend

  Scenario: Header file switcher shows a loading spinner while GitHub files load
    Given the GitHub repo is seeded with a file named "scores/loading.jianpu" for the file-switcher spinner
    And GitHub auth is seeded for the mocked owner
    And the GitHub directory listing GET is delayed by 1 second for the file switcher
    When the app loads with the editor ready while GitHub files load
    Then the file switcher trigger shows a loading spinner
    And opening the file list shows the hint "Loading files from GitHub…"
    When the GitHub directory listing resolves
    Then the file switcher trigger spinner is gone and the caret is visible
    And the file list shows "loading"
