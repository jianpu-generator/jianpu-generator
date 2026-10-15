Feature: Creating, nesting and moving files and folders from the file actions menu

  Background:
    Given the file tree is seeded with files "Hymns/Advent/O Come.jianpu", "Hymns/Amazing.jianpu" and "Scratch.jianpu" and an empty folder "Empty" with "Scratch.jianpu" active

  Scenario: New creates the file next to the open file
    Given the file tree active file is "Hymns/Advent/O Come.jianpu"
    When the file tree app loads
    And I choose "New" from the file actions menu
    Then the file switcher toolbar button shows the folder prefix "Hymns / Advent /" and the file name "untitled"

  Scenario: New at the top level when the open file is at the top level
    When the file tree app loads
    And I choose "New" from the file actions menu
    Then the file switcher toolbar button shows no folder prefix and the file name "untitled"

  Scenario: New folder is created next to the open file and shown expanded
    Given the file tree active file is "Hymns/Amazing.jianpu"
    When the file tree app loads
    And I choose "New folder" from the file actions menu
    And I open the file tree list
    Then the file tree shows the folder "New folder" with a count of 0
    And the file tree shows the empty folder hint
    And the file switcher toolbar button shows the folder prefix "Hymns /" and the file name "Amazing"

  Scenario: A second New folder gets a unique name
    When the file tree app loads
    And I choose "New folder" from the file actions menu
    And I choose "New folder" from the file actions menu
    And I open the file tree list
    Then the file tree shows the folder "New folder" with a count of 0
    And the file tree shows the folder "New folder 2" with a count of 0

  Scenario: A folder can be created inside a folder
    Given the file tree active file is "Hymns/Advent/O Come.jianpu"
    When the file tree app loads
    And I choose "New folder" from the file actions menu
    And I open the file tree list
    Then the file tree shows the folder "New folder" with a count of 0
    And the file tree shows the folder "New folder" nested 3 levels deep

  Scenario: Move to lists every folder and moves the open file into the chosen one
    When the file tree app loads
    And I choose "Move to…" from the file actions menu
    Then the move dialog offers the folders "Top level", "Empty", "Hymns" and "Advent"
    When I pick the folder "Advent" in the move dialog
    And I confirm the move dialog
    Then the file switcher toolbar button shows the folder prefix "Hymns / Advent /" and the file name "Scratch"

  Scenario: Move to the top level
    Given the file tree active file is "Hymns/Amazing.jianpu"
    When the file tree app loads
    And I choose "Move to…" from the file actions menu
    And I pick the folder "Top level" in the move dialog
    And I confirm the move dialog
    Then the file switcher toolbar button shows no folder prefix and the file name "Amazing"

  Scenario: Moving into a folder that already has a file of that name shows a clash message
    Given the file tree also has the file "Empty/Scratch.jianpu"
    When the file tree app loads
    And I choose "Move to…" from the file actions menu
    And I pick the folder "Empty" in the move dialog
    Then the move dialog says "already exists"
    And the move dialog cannot be confirmed
    When I pick the folder "Hymns" in the move dialog
    Then the move dialog shows no message
    And the move dialog can be confirmed

  Scenario: Demo files cannot be moved or renamed
    Given the file tree active file is "01-pitches.jianpu"
    When the file tree app loads
    And I open the file actions menu
    Then the file action "Move to…" is disabled
    And the file action "Rename" is disabled

  Scenario: Renaming with a slash moves the file and creates the folders
    When the file tree app loads
    And I open the file tree list
    And I rename the active file in the list to "Archive/Old/Scratch"
    Then the file switcher toolbar button shows the folder prefix "Archive / Old /" and the file name "Scratch"

  Scenario: Renaming from the menu shows the whole path and accepts a slash
    Given the file tree active file is "Hymns/Amazing.jianpu"
    When the file tree app loads
    And I choose "Rename" from the file actions menu
    Then the rename dialog holds "Hymns/Amazing"
    When I type "Elsewhere/Amazing" in the rename dialog and confirm
    Then the file switcher toolbar button shows the folder prefix "Elsewhere /" and the file name "Amazing"

  Scenario: Rename explains an empty segment, a clash and a relative segment instead of doing nothing
    Given the file tree also has the file "Empty/Scratch.jianpu"
    When the file tree app loads
    And I choose "Rename" from the file actions menu
    And I type "Hymns//Scratch" in the rename dialog
    Then the rename dialog says "empty segment"
    And the rename dialog cannot be confirmed
    When I type "../Scratch" in the rename dialog
    Then the rename dialog says "not allowed"
    When I type "Empty/Scratch" in the rename dialog
    Then the rename dialog says "already exists"
    When I type "Empty/Other" in the rename dialog
    Then the rename dialog shows no message
    And the rename dialog can be confirmed

  Scenario: The inline rename in the list also reports why a path is rejected
    When the file tree app loads
    And I open the file tree list
    And I type "Hymns//Scratch" in the active file's rename input in the list
    Then the list rename input says "empty segment"
