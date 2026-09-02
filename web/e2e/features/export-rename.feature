Feature: Rename-before-download modal

  Background:
    Given the single-part PDF export source is loaded, as seen in export rename

  Scenario: Confirming with an edited name downloads under that name
    When I export "PDF" and open the rename modal, as seen in export rename
    Then the rename modal shows the input pre-filled with "test.pdf"
    When I clear the rename input and type "renamed.pdf"
    And I click the modal's "Download" button
    Then the downloaded file is named "renamed.pdf", as seen in export rename

  Scenario: Pressing Enter without editing the name downloads under the computed name
    When I export "PDF" and open the rename modal, as seen in export rename
    And I press Enter in the rename input
    Then the downloaded file is named "test.pdf", as seen in export rename

  Scenario: Pressing Enter in the input submits the same as clicking Download
    When I export "PDF" and open the rename modal, as seen in export rename
    When I clear the rename input and type "enter-submit.pdf"
    And I press Enter in the rename input
    Then the downloaded file is named "enter-submit.pdf", as seen in export rename

  Scenario: Cancel closes the modal with no download
    When I export "PDF" and open the rename modal, as seen in export rename
    When I click the modal's Cancel button
    Then the rename modal is closed
    And no download fires, as seen in export rename

  Scenario: Escape closes the modal with no download
    When I export "PDF" and open the rename modal, as seen in export rename
    When I press Escape
    Then the rename modal is closed
    And no download fires, as seen in export rename

  Scenario: An empty or path-separator name is rejected inline
    When I export "PDF" and open the rename modal, as seen in export rename
    When I clear the rename input and type "sub/dir.pdf"
    And I click the modal's "Download" button
    Then the rename modal shows an inline error
    And no download fires, as seen in export rename

  Scenario: WAV preview Download button opens the modal instead of downloading immediately
    Given the export test timeout is extended to 60 seconds, as seen in export rename
    And the single-part export source is loaded, as seen in export rename
    When I open the export menu and choose "WAV", as seen in export rename
    And I click the inline audio player's Download button
    Then the rename modal shows the input pre-filled with "test.wav"
    When I clear the rename input and type "renamed.wav"
    And I click the modal's "Download" button
    Then the downloaded file is named "renamed.wav", as seen in export rename
    And the inline audio player still plays after the rename modal closes
