Feature: Error isolation between measures

  Scenario: Lyric underflow in measure 1 shows error overlay and still renders measure 2
    Given a user file with lyric underflow in measure 1 but valid lyrics in measure 2
    When the app loads
    Then the error overlay rect for the erroneous measure appears in the SVG
    And measure 2's lyrics still appear, confirming best-effort render
