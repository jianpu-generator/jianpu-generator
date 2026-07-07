# TODO: e2e coverage for the Edit Parts modal

`web/e2e/edit-parts-modal.spec.ts` covers the Kind/mode select, the Soundfont
select (including follow-target propagation), and the Octave select. The
table (`web/src/components/EditPartsModal.tsx`) has two remaining
interactive columns with no e2e coverage:

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
- [ ] **Follow target select** (the second `RadixSelect` shown when
      `declaration.mode === 'follow'`, via `handleFollowTargetChange`) —
      with more than one preceding part available, change the follow target
      and assert the select updates and the editor/localStorage source
      reflects the new `follow[<target>]` abbreviation. The existing
      `FOLLOW_SOURCE` fixture only has one preceding part (`M`), so this
      needs a fixture with at least two parts before the `follow` part to
      exercise an actual target change.

For each: follow the existing pattern in `edit-parts-modal.spec.ts` —
assert against both `getEditorSource` and `getStoredSource` where the
change should persist to the `.jianpu` source line.
