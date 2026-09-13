Feature: GitHub sign-in callback resolves under the GitHub Pages base path

  Regression coverage for a production-only bug: GitHub Pages has no
  server-side SPA fallback, so a real OAuth redirect landing on
  /jianpu-generator/synced-share/github-callback (a path with no file on
  disk) 404'd outright and never even loaded the app's JS -- invisible to
  the rest of the e2e suite, which always serves the app from Vite's dev
  server at the domain root. This exercises the actual built,
  base-pathed production artifact (web/dist, served the way GitHub Pages
  serves it, 404.html fallback included) instead of mocking anything
  away.

  Scenario: Navigating straight to the GitHub callback path under the GitHub Pages base path renders the callback page, not a dead end
    Given the production build with its GitHub-Pages 404 fallback exists
    When a browser navigates directly to the GitHub callback path with a code and state
    Then the response is a GitHub-Pages-style 404 that still renders the callback page
