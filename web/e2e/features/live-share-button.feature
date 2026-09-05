Feature: Live share button

  Scenario: A viewer opening the live link sees the current score immediately, before any owner edit
    Given clipboard permissions are granted
    And the file store is seeded with the live score
    When the owner loads the app and clicks "Go Live"
    Then a live-link-copied toast is shown
    When a viewer opens the copied live link in a new page
    Then the viewer's preview contains "Live Score"

  Scenario: The copied live link carries the filename as a human-readable suffix, and a viewer opening it still sees the score
    Given clipboard permissions are granted
    And the file store is seeded with the live score
    When the owner loads the app and clicks "Go Live"
    Then a live-link-copied toast is shown
    And the copied live link contains the filename as a human-readable suffix
    When a viewer opens the copied live link in a new page
    Then the viewer's preview contains "Live Score"

  Scenario: A viewer opening the live link does not get a ?file= param populated in the URL
    Given clipboard permissions are granted
    And the file store is seeded with the live score
    When the owner loads the app and clicks "Go Live"
    Then a live-link-copied toast is shown
    When a viewer opens the copied live link in a new page
    Then the viewer's page URL has no query string

  Scenario: Go live button copies a #live= link and shows a toast, then a dropdown offers copy/stop
    Given clipboard permissions are granted
    And local storage is cleared
    When the owner loads the app and clicks "Go Live"
    Then a live-link-copied toast is shown
    And the copied live link matches the live URL hash format
    And the go-live button now reads "Live"
    When the owner clicks the go-live button again
    Then the copy-live-link and stop-live buttons are visible
    When the owner clicks the copy-live-link button
    Then a live-link-copied toast is shown
    And the copied link is unchanged from before
    When the owner clicks the go-live button and then the stop-live button
    Then the stop-live button disappears
    And the go-live button reads "Go Live"

  Scenario: Stopping live ends the link for viewers, and going live again on the same link revives it
    Given clipboard permissions are granted
    And the file store is seeded with the live score
    When the owner loads the app and clicks "Go Live"
    Then a live-link-copied toast is shown
    When a viewer opens the copied live link in a new page
    Then the viewer sees the preview page
    When the owner clicks the go-live button and then the stop-live button
    Then the viewer sees "This live session has ended."
    And the viewer's preview no longer contains "Live Score"
    When a late viewer opens the copied live link in a new page
    Then the late viewer sees "This live session has ended."
    And the late viewer's preview no longer contains "Live Score"
    When the owner clicks "Go Live" again
    Then a live-link-copied toast is shown
    And the revived live link is identical to the original link
    When the late viewer reloads the page
    Then the late viewer's preview contains "Live Score"

  Scenario: A viewer importing the live score clears the #live= hash and focuses the imported file
    Given clipboard permissions are granted
    And the file store is seeded with the live score
    When the owner loads the app and clicks "Go Live"
    Then a live-link-copied toast is shown
    When a separate browser context opens the copied live link as a viewer
    And the viewer clicks "Import to my scores"
    Then the viewer's shared preview banner is gone
    And the viewer's page URL has no hash
    And the viewer's file switcher shows the live filename

  Scenario: Dragging across measures in a live viewer highlights them, even without a mounted editor to round-trip the selection through
    Given clipboard permissions are granted
    And the file store is seeded with a multi-measure live drag score
    When the owner loads the app and clicks "Go Live"
    Then a live-link-copied toast is shown
    When a viewer opens the copied live link in a new page and waits for measures to render
    Then the viewer's parts toolbar is visible and no Monaco editor is mounted
    When the viewer drags from measure 0 to measure 2
    Then the viewer's measure highlight is visible
    And the viewer's play-measure button reads "Measures 1-3"

  Scenario: Tapping a single note in a live viewer only highlights that note, not its whole measure
    Given clipboard permissions are granted
    And the file store is seeded with a multi-measure live drag score
    When the owner loads the app and clicks "Go Live"
    Then a live-link-copied toast is shown
    When a viewer opens the copied live link in a new page and waits for measures to render
    Then the viewer's parts toolbar is visible and no Monaco editor is mounted
    When the viewer taps the first note
    Then the viewer's tapped note is highlighted
    And the viewer's measure highlight is not shown

  Scenario: Re-going-live on the same file reproduces the same link, so it never needs re-sharing
    Given clipboard permissions are granted
    And local storage is cleared
    When the owner loads the app and clicks "Go Live"
    Then a live-link-copied toast is shown
    When the owner clicks the go-live button and then the stop-live button
    Then the stop-live button disappears
    When the owner clicks "Go Live" again
    Then a live-link-copied toast is shown
    And the revived live link is identical to the original link
