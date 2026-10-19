# TLA+ specs

Model-checked designs of the cloud sync path. Each folder holds one model:
`<Module>.tla` and one or more configs (`<Module>.cfg`, or
`<Module>.<variant>.cfg` for other bounds).

```sh
mise install   # Java (temurin-21)
just tla       # downloads the pinned tla2tools.jar once, then runs every config
```

CI runs `just tla` in the `tla` job. A change to the code a spec stands for
must update the spec in the same commit (see `CLAUDE.md`).

## `outbox-tabs/OutboxTabs`

Several tabs of one account share the outbox stored in IndexedDB. They deliver
one file's content saves to the server over a network that loses, delays and
duplicates messages.

### Abstractions

- **One file lane, `SaveContent` only.** Lane ordering, batches,
  rename/trash/create messages and the multi-file scheduler are out of scope
  (planned for `OutboxCore`, Phase 1 of `PLAN-formal-verification.md`).
- **Content is a number.** Edit `n` saves content `n`, so "newer" is `>`.
- **One step per store transaction.** `transact` runs under the cross-tab
  store lock and IndexedDB applies its writes in one transaction, so the
  read-change-write is atomic. Every other `await` in the looper is a step
  boundary.
- **The codec is the identity.** `encode_queue` / `decode_stored_queue`
  round-trip exactly (covered by `crates/cloud-outbox/tests/codec.rs`).
- **The network is a set.** The server may handle any request ever sent, any
  number of times, in any order. A lost request or response is a `Timeout`,
  which the looper treats as `Transient`.
- **Merge keeps the head's content.** With one user and one device, "theirs"
  is only ever an older write of ours. Multi-device merging is Phase 3.

### Variables

| Spec | Code |
|---|---|
| `store` | the `lane:<fileId>` / `rev:<fileId>` records in IndexedDB (`outboxStore.ts`) |
| `server` | the file's `files` row: `content`, `revision` |
| `net` | requests sent by `sendMessage.ts` |
| `shown` | each looper's in-memory `queue`, used only for display |
| `lock`, `leading` | `runAsLeader` in `outboxLock.ts` (`navigator.locks`) |
| `drain`, `drainAgain` | `drainOnce` / `drainRequested` in `outboxLooper.ts` |
| `request`, `response` | the send `drainOnce` is awaiting |
| `inbox` | an unread `BroadcastChannel` message (`outboxChanges.ts`) |

### Actions

| Spec | Code |
|---|---|
| `Enqueue`, `Eligible`, `BeginSendQueue`, `OnResult`, `Resolve` | `crates/cloud-outbox`: `enqueue`, `is_eligible`, `begin_send`, `on_result`, `resolve` |
| `Recover` | the recovery step in `decode_queue` (codec.rs), run only by a tab taking over as leader |
| `Mutate` | `mutate` (enqueue / resolve): one `transact`, then announce and `requestDrain` |
| `Begin` | `drainOnce`: `transact(begin_send)`, then `sendMessage` |
| `ServerHandle` | `files.rs` content CAS (`update_file_content.sql`) + `classifyRevisionConflict` |
| `Receive`, `Timeout` | `drainOnce`: `transact(on_result)` (with `recordSent` on `Ok`) |
| `Wake` | the scheduler's wake-up for a `Waiting` lane |
| `Hear` | `adoptExternalChange` |
| `Acquire`, `BecomeLeader` | the leader task in `start()`: `transact(identity, { recover: true })`, then drain |
| `Close`, `Reopen` | closing a tab (frees the lock, requests stay on the wire); `start()` |

### Properties

- `NoLostEdit`: the latest content is on the server, still queued in the
  store, or the user discarded it.
- `ServerContentMonotone`: the server never goes back to older content.
- `SingleDrainer`: at most one open tab runs the drain loop.

### Bounds

| Config | Tabs | Edits | Closes | Resolves | Distinct states | Time |
|---|---|---|---|---|---|---|
| `OutboxTabs.cfg` | 2 | 3 | 1 | 1 | ~119k | ~1 s |
| `OutboxTabs.three-tabs.cfg` | 3 | 3 | 2 | 2 | ~5.9M | ~30 s |

### Bug it found

Before store transactions, each looper kept its own in-memory queue and wrote
the store as a per-key diff against what it last persisted. The leader, while
awaiting `sendMessage`, held a stale copy of the lane. When the response
arrived, it applied `on_result` to that copy and wrote it back. This
overwrote a save a follower tab had queued in the meantime, so the user's
latest edit was silently lost.

TLC found it in a few steps, including with only one tab editing. The
vitest "keeps a save queued from a follower tab while the leader is sending
the previous one" (`outboxLooperTabs.test.ts`) reproduces it.

The fix (`outboxTransaction.ts`) makes every change to the stored queue read
the store, decode it as stored (`decode_stored_queue`, no recovery), apply the
change, and write back only the records that differ. All of this happens
under the store lock `jianpu-outbox-store:<account>`. Recovery, which clears
`in_flight` and turns `Waiting` into `Draining`, now happens only when a tab
takes over as leader.

As a check that the model is sensitive to this bug: making `Receive` apply
`OnResult` to `shown[t]` instead of `store` brings the counterexample back
within seven states.
