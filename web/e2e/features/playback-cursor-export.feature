Feature: Playback cursor follows export playback

  # WAV and MP3 share the same inline player and the same
  # `usePlaybackCursor` hook (`Preview.tsx`/`usePlaybackCursor.ts`), so both
  # formats are expected to animate the cursor identically — parameterized
  # rather than two near-duplicate feature files.
  #
  # As of this writing the MP3 row is a known regression: `Preview.tsx` only
  # threads `noteTimings` into the cursor hook when `wavUrl` is the inline
  # player's source (`audioNoteTimings = wavUrl ? noteTimings : undefined`),
  # and `handleGenerateMp3` (`worker/audioMessageHandlers.ts`) never computes
  # note timings at all — unlike `handleGenerateAudio`, which pairs the WAV
  # bytes with a `listNoteTimings` call.

  Scenario Outline: Playing the inline <format> export shows the playback cursor on the sounding note
    Given the playback-cursor export test fixture is loaded
    When I open the export menu and choose "<format>"
    Then the inline audio player is visible with a blob src, as seen in playback cursor export
    When I play the inline audio player
    Then the first note shows the playback cursor highlight

    Examples:
      | format |
      | WAV    |
      | MP3    |
