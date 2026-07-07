# TODO: e2e coverage for the Edit Parts modal

`web/e2e/edit-parts-modal.spec.ts` covers the Kind/mode select, the Soundfont
select (including follow-target propagation), the Octave select, the Volume
slider, and the Follow target select. All interactive columns in the table
(`web/src/components/EditPartsModal.tsx`) now have e2e coverage.

- [x] **Volume** (`Slider.Root`/`Slider.Thumb` in `PartRow`, via
      `handleVolumeChange` → `onPartDeclarationChange`) — drag or set the
      slider for a part and assert the displayed percentage updates, plus
      the editor/localStorage source reflects the new volume (declaration
      value of `100` maps to `null`/omitted, per `handleVolumeChange`).
      See `web/e2e/edit-parts-modal.spec.ts`'s "volume slider changes the
      MIDI volume for a part" test. Added `data-testid`s
      (`volume-slider-<abbr>`, `volume-value-<abbr>`) to
      `EditPartsModal.tsx` to make the slider addressable; the test focuses
      the Radix slider thumb and presses `Home` to deterministically hit
      the min value (1%) rather than relying on drag pixel math.
- [x] **Follow target select** (the second `RadixSelect` shown when
      `declaration.mode === 'follow'`, via `handleFollowTargetChange`) —
      with more than one preceding part available, change the follow target
      and assert the select updates and the editor/localStorage source
      reflects the new `follow[<target>]` abbreviation. See
      `web/e2e/edit-parts-modal.spec.ts`'s "follow target select changes the
      followed part" test, using the new `MULTI_FOLLOW_SOURCE` fixture
      (`M`, `H`, `C = follow[M]`) so there are two preceding parts to choose
      between. Added a `testId` prop pass-through
      (`follow-target-select-<abbr>`) to the follow-target `RadixSelect` in
      `EditPartsModal.tsx` to make it addressable.

Nothing left to add here — this file can be deleted.
