# Formal verification of the cloud sync path (TLA+)

Goal: model-check the parts of the app where a bug silently loses or
overwrites a user's score — the cloud outbox, multi-tab leadership, and
multi-device conflict handling — and keep those models tied to the code so
they don't rot.

Non-goal: verifying the parser / layout / renderer. Their "correct" is mostly
visual; fuzzing and property tests cover them better (see the appendix).

## Status

- **Phase 0: done.** `just tla` (Java via `mise.toml`) and the `tla` CI job.
- **Phase 2: done**, as `specs/outbox-tabs/OutboxTabs`. It covers one lane
  and `SaveContent` only; see `specs/README.md`. The suspected race below was
  real and is fixed by store transactions (`outboxTransaction.ts`).
- Phases 1, 3 and 4: not started.

## Why here

The sync path is a set of concurrent actors that tests exercise only in the
interleavings someone thought of:

| Actor | Code | State |
|---|---|---|
| Outbox state machine | `crates/cloud-outbox/src/{enqueue,scheduling,transitions,codec}.rs` | `Queue { lanes, revisions }` |
| Looper (per tab) | `web/src/storage/outbox/outboxLooper.ts` | in-memory `queue`, `persisted` diff map, `isLeader` |
| Leader lock | `web/src/storage/outbox/outboxLock.ts` (`navigator.locks`) | one leader, handover on close/abort |
| Change channel | `web/src/storage/outbox/outboxChanges.ts` (`BroadcastChannel`) | async, unordered w.r.t. store writes |
| Conflict flow | `web/src/storage/outbox/conflictFlow.ts`, `baseSnapshots.ts`, `cloud-outbox/src/merge.rs` | base snapshot, `MergedAndSave` |
| Server | `crates/live-share-worker/src/files.rs`, `queries/*.sql` | `files` rows: content CAS on `revision`; rename/trash/restore unconditional; idempotent create; all-or-nothing batch rename |
| Network | `sendMessage.ts` | requests/responses can be lost, delayed, duplicated |

The Rust core is pure functions over `Queue`, which maps almost one-to-one
onto TLA+ actions. That makes this a good fit.

### Suspected race to check first (confirmed and fixed)

A follower tab's `mutate` does load → change → `persist` → `announce`
without holding the leader lock. Meanwhile the leader may be awaiting
`sendMessage` for the **same lane**. When the response arrives, the leader
runs `outboxOnResult` on its stale in-memory queue and `persistNow` puts its
version of `lane:<fileId>`, because that record differs from what the leader
last persisted. That can overwrite the message the follower just appended.
The later `adoptExternalChange` → `load()` then reads the already-clobbered
record.

`outboxLooperTabs.test.ts` only covers the case where the leader is idle. The
model should find this in seconds. If it's real, it's a lost edit with no
error.

Related interleavings the model should cover:
- **Leader handover while a request is in flight.** `runAsLeader` stops
  racing on abort and releases the lock. `decode_queue` clears `in_flight`, so
  the new leader resends while the old request is still out. Does
  idempotency (CAS / `idempotent_create_match` / `AlreadyApplied`) make this
  harmless for every message kind?
- **Lost `Ok` response on `SaveContent`.** The retry gets a `Conflict` against
  our own write, which goes to `NeedsMerge`, which merges mine == theirs.
  Does it settle cleanly without user prompts?
- **Revisions regressing.** `MergedAndSave` / `record_revision` set
  `revisions[file]` from a server value fetched earlier. Can the recorded
  revision go backwards?
- **Crash between `recordSent` and `on_result`** (the code comment claims a
  resend is harmless).

## Phase 0: setup and a cheap confirmation (½ day)

- Add Java + TLA+ tools via `mise.toml` (`java`, and download
  `tla2tools.jar` in a `just tla` recipe). The editor is the VS Code TLA+
  extension.
- Directory layout: `specs/` at the repo root with one folder per model, plus
  `specs/README.md` that maps each spec variable and action to the code it
  abstracts.
- Before modelling, write a failing vitest for the suspected race (leader
  mid-send on `file-a`, follower enqueues to `file-a`, response arrives),
  following the bug-fix rule: failing scenario first. This settles whether
  it's real today. The model then guards the fix and finds the variants.

## Phase 1: `OutboxCore`, one tab, one device, unreliable network (2–3 days)

State:
- `lanes`, `revisions`: the `Queue`, abstracted. Lane = status, `inFlight`,
  sequence of messages. Message kinds: `Create`, `Save(c)`, `Rename(n)`,
  `Trash`, `Restore`, `RenameBatch(moves)`.
- `server`: per file `[content, name, revision, trashed]`, with name
  uniqueness.
- `net`: a bag of in-flight requests and responses.
- `durable`: what `encode_queue` last wrote, so a `Crash` action can restart
  via `decode_queue` semantics (clear `inFlight`, recover `Waiting`).

Actions: `UserEdit(f, c)` (enqueue with `SaveContent` coalescing),
`UserRename`, `UserTrash`, `UserRestore`, `UserFolderMove`, `BeginSend`
(oldest eligible head, respecting `is_blocked`), `ServerHandle`,
`DropRequest`, `DropResponse`, `DuplicateRequest`, `DeliverResponse`
(`on_result`, ignoring stale ids), `RetryTimerFires`, `UserResolve(Retry |
Discard)`, `SignIn`, `Crash`.

Bounds: 2 files, 3 content values, 2 names, at most 4 user actions, message
ids from a small set. Time is abstracted: `Waiting` becomes eligible
nondeterministically.

Properties:
- **NoLostEdit (safety):** for every file, the user's latest content is
  either on the server, still queued durably, or was explicitly discarded.
  It's never just gone.
- **LaneOrder:** the server applies a lane's messages in enqueue order. A
  `RenameBatch` never runs before a pending `Create` of its files, and never
  runs before an earlier structural message of a file it moves.
- **BatchAtomic:** a batch rename applies all of its moves or none.
- **DuplicateHarmless:** delivering any request twice leaves the server
  state the same as delivering it once.
- **RevisionMonotone:** `revisions[f]` never decreases.
- **NoBlockedCycle:** `is_blocked` never makes every non-empty lane wait at
  once. This checks the "complementary comparisons" claim in
  `scheduling.rs`.
- **EventuallyQuiet (liveness, weak fairness on network/timer/user
  resolution):** every message is eventually delivered or its lane sits in a
  state the user can see (`NeedsSignIn`, `NeedsMerge`, `Failed`). Never stuck
  with `inFlight` set and nothing pending, and never `Waiting` with no wake.

## Phase 2: `OutboxTabs`, N tabs sharing one store (1–2 days)

Extends Phase 1 (or `INSTANCE`s it):
- `store`: key → value, with per-key writes. Whether `store.apply` is atomic
  across keys is an open question below; model both.
- Per tab: `memQueue`, `persisted`, `isLeader`, and a program counter for the
  `await` points in `drainOnce` / `mutate` / `adoptExternalChange`. Each
  `await` is a step boundary, which is what exposes the race.
- `lock`: holder plus waiters. `TabClose` / `Abort` releases it immediately,
  even mid-send.
- `channel`: announcements delivered asynchronously, possibly after later
  store writes.

Properties: the same **NoLostEdit** (now over edits from any tab), plus
**SingleDrainer** (at most one tab runs `drainOnce` at a time). Also record,
rather than assert, how many concurrent in-flight requests the same lane can
have during handover. Phase 1's **DuplicateHarmless** must cover that number.

Expected outcome: a counterexample trace for the suspected race. Fix it in
code (for example, have followers send mutations to the leader instead of
writing the store, or do every store mutation under a short `navigator.locks`
critical section), mirror the fix in the spec, and re-check.

## Phase 3: `SyncDevices`, two devices, one server, merge (2–3 days)

- Two devices, each with an abstracted Phase-1 outbox plus a base snapshot
  per file.
- Merge is a nondeterministic oracle: `Clean(m)` or `Conflicted`, with the
  constraint that `mine == theirs` gives `Clean`.
- Conflict flow: `NeedsMerge`, fetch server copy, merge, then
  `MergedAndSave`, or wait for a user choice (keep mine / keep theirs /
  edited merge / "Overwrite").
- Pull path: how a device learns of remote changes. Model whatever the app
  actually does; it's an open question below.

Properties:
- **NoSilentOverwrite:** a server content write from device A replaces
  content written by device B only if A's base already included B's write,
  or a user explicitly chose to overwrite.
- **Convergence (liveness):** once users stop editing and the network is
  fair, both devices' local copies equal the server copy.
- **StructuralRaces:** rename/trash/restore are unconditional
  server-side. Enumerate the outcomes of concurrent rename versus trash
  across devices and confirm each one is acceptable. This is documentation as
  much as checking.

## Phase 4: keep the spec honest (2 days)

A spec that drifts from the code is worse than none. Two mechanisms:

1. **Property tests on the real crate.** Turn the Phase-1 invariants into
   `quickcheck` properties in `crates/cloud-outbox/tests/`. Generate random
   action sequences (enqueue / begin_send / on_result with random
   `SendResult` / resolve / encode-decode round-trip) against the real
   functions, plus a tiny in-test fake server. `quickcheck` is already a
   dependency. This catches code changes that break an invariant even when
   the spec isn't updated.
2. **Trace replay.** Export TLC behaviours (`-dumpTrace json`), then replay
   them in a Rust test that maps each spec action onto the matching
   `cloud-outbox` call. Assert the resulting `Queue` abstracts to the spec's
   state. This checks that the spec describes the code, not an idealised
   version of it.

## Phase 5: CI (½ day)

- A `tla` job in `.github/workflows/ci.yml`, path-filtered to
  `specs/**`, `crates/cloud-outbox/**`, `web/src/storage/outbox/**`,
  `crates/live-share-worker/src/files.rs` and
  `crates/live-share-worker/queries/**`. It runs TLC on the small configs
  with `-workers auto` and a time cap of about 10 minutes.
- Larger bounds (3 tabs, 3 files) run in a nightly scheduled job.
- PR checklist line (in `AGENTS.md`/`CLAUDE.md`): changes to the outbox,
  looper or file-write queries must update `specs/` in the same commit, like
  the existing `syntax.md` / `ARCHITECTURE.md` rules.

## Later / optional

- **Share lifecycle** (`share.rs`, `folder_share.rs`): an ended or trashed
  share never exposes content, and a folder-share prefix rewrite stays
  consistent across concurrent renames. Most of this is single-request logic,
  so property tests are likely enough. Model it only if concurrent folder
  moves plus share rewrites turn out to be subtle.
- **OAuth / identity flow** (`oauth.rs`, `verification.rs`): mainly a
  security question. A small model (or ProVerif-style reasoning) of the
  state/nonce handling is possible but lower priority.
- **Quint** instead of raw TLA+ if the syntax becomes a barrier. It uses the
  same semantics and has TypeScript-like syntax and a simulator.

## Open questions (answer before or during Phase 2/3)

1. Is `OutboxStore.apply({puts, deletes})` one IndexedDB transaction, so it's
   atomic across keys?
2. How does a device pick up another device's edits: polling, on focus, only
   on conflict? Convergence depends on this.
3. What exactly does "Overwrite" do: send with `expected_revision =
   current_revision`?
4. Does `onAcknowledged` record the base snapshot, and can it run
   before or after another tab's mutation in a way that leaves a stale base?

## Estimate

About 8–11 focused days end to end. Phase 0 plus Phase 2 alone (about 2–3
days) is the highest-value slice, because it targets the most likely real bug.

## Appendix: other static analysis, re-checked against the repo

- Clippy already denies `panic`, `unwrap_used`, `expect_used` and
  `indexing_slicing` in production code (workspace `[lints]`), so the
  anti-panic lints are done. `arithmetic_side_effects` is **not** enabled.
  It's worth turning on for `midi/`, `grid_layout/` and `layout/`.
- `cargo-fuzz` on parse → desugar → group → layout (no panic; format
  idempotent; part-stream `apply(extract(x)) == x`): not set up yet.
- `quickcheck` is declared but unused in `src/`. Round-trip properties are the
  cheapest next step.
- Kani for duration / tuplet / column-width arithmetic: optional.
- Mutation testing on PR diffs: see the earlier discussion.
