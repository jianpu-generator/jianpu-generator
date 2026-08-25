Feature: GitHub OAuth device-flow connect and disconnect

  Scenario: Connect via device flow shows the verification code and switches to the github backend
    Given the GitHub device-flow OAuth endpoints are mocked with user code "ABCD-1234"
    And the mocked GitHub user and repo exist for the mocked owner
    And the GitHub Contents API is mocked with no seeded files
    When the app loads for the OAuth connect flow
    And I open the storage settings modal for OAuth
    And I select the "GitHub repository" storage option
    And I click the GitHub OAuth "Connect GitHub" button
    Then the device verification UI shows the code "ABCD-1234"
    And the app shows connected as the mocked owner
    And the stored storage-backend preference is set to github for the mocked owner

  Scenario: Disconnect reverts to the local backend and stops saving to github
    Given GitHub auth is seeded for the mocked owner
    And the mocked GitHub user exists for disconnect
    And the GitHub Contents API is mocked with a seeded file "scores/song.jianpu" and PUT forbidden
    When the app loads the GitHub-backed file list for disconnect
    And I select the "song.jianpu" tab before disconnecting
    And I open the storage settings modal for OAuth
    Then the app shows connected as the mocked owner
    When I click the GitHub OAuth "Disconnect" button
    Then the stored github-auth is cleared
    And the app no longer shows as connected
    And the "This browser" storage option is checked
    When I close the storage settings modal and attempt to edit and force-save
    Then no PUT to GitHub occurs after disconnecting
