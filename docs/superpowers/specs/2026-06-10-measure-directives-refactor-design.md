# Measure Directives Refactor Design

**Date:** 2026-06-10
**Status:** Approved

## Problem

Directives (BPM, key, time signature, label) are measure-level metadata — they apply to all tracks equally. But the current data model embeds them inside `GroupedMeasure`, which is a per-track struct. This causes a bug: directive events (e.g. `bpm=80`) are sent only to the first notes track accumulator in the parser, while chord tracks default to `bpm=120`. The combiner reads from the first track regardless of kind, so when a chord track is declared before the notes track, it reads the wrong (default) BPM value.

## Root Cause

Three-layer mismatch:

1. **Parser** sends directive events only to `first_notes_track_index`, not to chord tracks.
2. **Grouper** initialises every `PartGrouper` with `current_bpm: 120`; chord tracks never receive a `BpmChange` event, so their measures always carry `bpm: Some(120)`.
3. **Combiner** uses `find_map` over all tracks to read metadata, returning the first track's measure — which may be a chord track with the wrong BPM.

## Design

Directives are separated from per-track content at every layer. They flow through their own dedicated path from parser to combiner and are never duplicated into track-level data.

### Data types (`ast/grouped.rs`)

Add `MeasureDirectives` — the single source of truth for measure-level metadata:

```rust
pub(crate) struct MeasureDirectives {
    pub(crate) time_signature: Option<TimeSignature>,
    pub(crate) bpm: Option<u32>,
    pub(crate) key: Option<KeyChange>,
    pub(crate) label: Option<String>,
}
```

`GroupedMeasure` loses all directive fields and holds only per-track content:

```rust
pub(crate) struct GroupedMeasure {
    pub(crate) notes: Notes,
}
```

Add `GroupedScore` as the intermediate type flowing from grouper to combiner:

```rust
pub(crate) struct GroupedScore {
    pub(crate) measure_directives: Vec<MeasureDirectives>,  // one per measure
    pub(crate) parts: Vec<GroupedPart>,                     // one per track
}
```

`GroupedPart` is otherwise unchanged.

### Parser (`interleaved_parser.rs`)

Add a dedicated directive accumulator: `directive_events: Vec<Vec<Spanned<ScoreEvent>>>`, one inner `Vec` per measure group. In `process_bar_group`, the events returned by `split_directive` are appended to this accumulator instead of into any track's event list. Per-track accumulators continue to receive only note/chord/extension events. The `time_num`/`time_den` state used for beat validation stays as-is. `build_parse_result` is extended to return the directive event list alongside the parsed tracks.

### Grouper (`grouper.rs`)

Add a `DirectiveGrouper` that processes the per-measure directive event lists into `Vec<MeasureDirectives>`. It tracks what changed across measures using the same changed-flag logic (`bpm_changed`, `time_sig_changed`, etc.) that `PartGrouper` uses today. `PartGrouper` loses all directive tracking fields (`current_bpm`, `current_key`, `current_time_sig`, `bpm_changed`, `key_changed`, `time_sig_changed`) and the directive-handling arms in `process_event`. `group()` produces a `GroupedScore` instead of a bare `Vec<GroupedTrack>`.

### Combiner (`combiner.rs`)

Takes `&GroupedScore` instead of `&[GroupedTrack]`. The `find_map` guessing which track to read metadata from is replaced by a direct index into `measure_directives[measure_idx]`. Building `PartRow`s and distributing lyrics is unchanged.

## Scope

Internal only. The public API (`compile()`, `render_svgs()`, `Score`, `MultiPartMeasure`) is unchanged.

## Testing

- Add a regression test: chord track declared before notes track with `bpm=80` must render `♩=80`.
- Existing BPM, key, and time signature tests continue to pass unchanged.
