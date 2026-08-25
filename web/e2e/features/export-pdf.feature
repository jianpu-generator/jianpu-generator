Feature: Export PDF

  Scenario: Export > PDF produces a non-empty downloaded file
    Given the single-part PDF export source is loaded
    When I export "PDF" and capture the download, as seen in export pdf
    Then the downloaded PDF file is larger than 1000 bytes
    And the downloaded file is named "test.pdf", as seen in export pdf

  Scenario: Export Parts > PDF (ZIP) produces a non-empty downloaded zip for a multi-part score
    Given the multi-part PDF export source is loaded
    When I export "PDF (ZIP)" and capture the download, as seen in export pdf
    Then the downloaded PDF file is larger than 1000 bytes
    And the downloaded file is named "test.zip", as seen in export pdf

  Scenario: Export > PDF filename includes only the enabled parts when a part is hidden
    Given the multi-part PDF export source is loaded
    And I hide the "H" part via its eye toggle, as seen in export pdf
    When I export "PDF" and capture the download, as seen in export pdf
    Then the downloaded file is named "test (Melody).pdf", as seen in export pdf

  Scenario: Rapid double-click on Export > PDF only triggers a single export
    Given the single-part PDF export source is loaded
    When I open the export menu and rapid double-click the "PDF" item, capturing the download
    Then the downloaded file is named "test.pdf", as seen in export pdf
    And only a single PDF download ever fires
