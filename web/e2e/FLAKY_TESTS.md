# Known flaky e2e tests

Observed while landing the mobile-workspace change (unrelated to that
change — confirmed by running these specs against an unmodified `master`
checkout, where they also failed intermittently).

## `conflict-resolution-github.spec.ts`

Both tests (`overwriting mine re-pushes the in-memory edit...` and
`discarding mine reloads the remote content...`) intermittently fail at
the `setUpConflictingEdit` helper's assertion:

```
expect(page.getByTestId('save-status-badge')).toHaveText('Save failed')
```

Instead the badge still reads `Unsaved (autosaving in Ns)`, meaning the
`Meta+s` force-save in the helper didn't land before the assertion's
5s timeout. Passes reliably when run alone or with few workers; fails
more often under the full suite's 9-worker parallel run, suggesting the
force-save timing is sensitive to CPU contention rather than a real bug
in the app.

## `live-share-button.spec.ts`

`stopping live ends the link for viewers, and going live again on the
same link revives it` intermittently fails waiting for
`getByText('This live session has ended.')` to become visible — observed
once during the same full-suite runs above, not reproduced in isolation.
