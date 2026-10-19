----------------------------- MODULE OutboxTabs -----------------------------
(***************************************************************************)
(* Several browser tabs of one account sharing the cloud outbox stored in  *)
(* IndexedDB, delivering one file's content saves to the server over an   *)
(* unreliable network.                                                     *)
(*                                                                         *)
(* Each `await` in web/src/storage/outbox/outboxLooper.ts is a step        *)
(* boundary, so other tabs can act in between. Every change to the stored  *)
(* queue is one `transact`: read, change, write under the cross-tab store  *)
(* lock, so it is a single step here. The Rust core (crates/cloud-outbox)  *)
(* is abstracted to one file lane holding `SaveContent` messages.          *)
(* specs/README.md maps every variable and action to the code it stands    *)
(* for.                                                                    *)
(***************************************************************************)
EXTENDS Naturals, Sequences, TLC

CONSTANTS
    Tabs,         \* the open tabs
    UserTabs,     \* the tabs the user edits and resolves in
    MaxEdits,     \* content edits the user makes, across all tabs
    MaxCloses,    \* tab closes (each one may reopen as a fresh tab)
    MaxResolves   \* resolutions the user picks (retry, discard, merge)

NoId == 0                          \* `in_flight: None`; message ids start at 1
NoLane == [present |-> FALSE]      \* no `lane:<fileId>` record / no lane
NoRequest == [id |-> NoId, content |-> 0, expected |-> 0]
NoResult == [kind |-> "None", rev |-> 0]
NoTab == "NoTab"

VARIABLES
    store,       \* the IndexedDB records the codec owns: [lane, rev]
    server,      \* the file's `files` row: [content, rev]
    net,         \* every request ever sent; the server may (re)handle any
    alive,       \* tab -> is the tab open
    shown,       \* tab -> the queue it last read, which its UI shows
    lock,        \* holder of the leader lock, or NoTab
    leading,     \* tab -> "no" | "taking" (holds the lock) | "yes"
    drain,       \* tab -> "off" | "begin" | "sending" (drainOnce's await points)
    drainAgain,  \* tab -> `drainRequested`
    request,     \* tab -> the request its drain awaits
    response,    \* tab -> the response received for it, if any
    inbox,       \* tab -> an unread `BroadcastChannel` announcement
    edits,       \* content edits made so far; edit n saves content n
    discarded,   \* the user discarded the lane since the latest edit
    closes,
    resolves,
    nextId       \* the next message id (`crypto.randomUUID()`)

vars == <<store, server, net, alive, shown, lock, leading, drain, drainAgain,
          request, response, inbox, edits, discarded, closes, resolves,
          nextId>>

-----------------------------------------------------------------------------
(* The Rust core, crates/cloud-outbox. A queue is [lane, rev]: one file's  *)
(* lane and its recorded revision (0 when none is recorded). The codec     *)
(* (`encode_queue` / `decode_stored_queue`) is the identity here.          *)

\* enqueue.rs `enqueue`: a save replaces the lane's last save unless that
\* one is in flight.
Enqueue(q, msg) ==
    IF ~q.lane.present
    THEN [q EXCEPT !.lane = [present |-> TRUE, msgs |-> <<msg>>,
                             inFlight |-> NoId, status |-> "Draining"]]
    ELSE LET l == q.lane
             coalesce == Len(l.msgs) > 0 /\ l.inFlight # l.msgs[Len(l.msgs)].id
             kept == IF coalesce THEN SubSeq(l.msgs, 1, Len(l.msgs) - 1)
                                 ELSE l.msgs
         IN [q EXCEPT !.lane = [l EXCEPT
                 !.msgs = Append(kept, msg),
                 !.status = IF @ = "Idle" THEN "Draining" ELSE @]]

\* scheduling.rs `is_eligible`; `due` stands for `retry_at_ms <= now_ms`.
Eligible(q, due) ==
    /\ q.lane.present
    /\ q.lane.msgs # <<>>
    /\ q.lane.inFlight = NoId
    /\ \/ q.lane.status \in {"Idle", "Draining"}
       \/ q.lane.status = "Waiting" /\ due

\* scheduling.rs `begin_send`, for an eligible lane.
BeginSendQueue(q) ==
    [q EXCEPT !.lane.inFlight = Head(q.lane.msgs).id,
              !.lane.status = "Draining"]

RequestOf(q) ==
    [id |-> Head(q.lane.msgs).id,
     content |-> Head(q.lane.msgs).content,
     expected |-> q.rev]

\* transitions.rs `on_result`: a result for a message that is not the
\* lane's in-flight head is ignored.
OnResult(q, id, result) ==
    IF ~(/\ q.lane.present
         /\ q.lane.inFlight = id
         /\ q.lane.msgs # <<>>
         /\ Head(q.lane.msgs).id = id)
    THEN q
    ELSE CASE result.kind = "Ok" ->
                LET rest == Tail(q.lane.msgs)
                IN [lane |-> [present |-> TRUE, msgs |-> rest, inFlight |-> NoId,
                              status |-> IF rest = <<>> THEN "Idle" ELSE "Draining"],
                    rev |-> result.rev]
           [] result.kind = "Transient" ->
                [q EXCEPT !.lane.inFlight = NoId, !.lane.status = "Waiting"]
           [] result.kind = "Conflict" ->
                [q EXCEPT !.lane.inFlight = NoId, !.lane.status = "NeedsMerge"]

\* transitions.rs `resolve`. A merge keeps the head's content: with one user,
\* "theirs" is only ever an older write of ours.
Resolve(q, change) ==
    CASE change.kind = "Retry" ->
            IF q.lane.present /\ q.lane.status \in {"Waiting", "NeedsMerge"}
            THEN [q EXCEPT !.lane.status = "Draining", !.lane.inFlight = NoId]
            ELSE q
      [] change.kind = "Discard" ->
            [lane |-> NoLane, rev |-> 0]
      [] change.kind = "Merge" ->
            IF q.lane.present /\ q.lane.status = "NeedsMerge" /\ q.lane.msgs # <<>>
            THEN [lane |-> [q.lane EXCEPT !.status = "Draining", !.inFlight = NoId],
                  rev |-> change.rev]
            ELSE q

\* codec.rs `decode_queue`'s recovery: the previous leader's interrupted work
\* resumes.
Recover(q) ==
    [q EXCEPT
        !.lane = IF ~@.present THEN @
                 ELSE [@ EXCEPT !.inFlight = NoId,
                                !.status = IF @ = "Waiting" THEN "Draining" ELSE @]]

-----------------------------------------------------------------------------
(* The looper, web/src/storage/outbox/outboxLooper.ts.                     *)

\* `requestDrain`.
DrainAfterRequest(t) ==
    IF leading[t] = "yes" /\ drain[t] = "off" THEN [drain EXCEPT ![t] = "begin"]
    ELSE drain
DrainAgainAfterRequest(t) ==
    IF leading[t] = "yes" /\ drain[t] # "off" THEN [drainAgain EXCEPT ![t] = TRUE]
    ELSE drainAgain

\* `changes.announce()`: every other open tab hears it, later.
Announce(t) == [u \in Tabs |-> IF u # t /\ alive[u] THEN TRUE ELSE inbox[u]]

\* `transact(change)`: the stored queue is replaced by `next`, which was
\* computed from the stored queue itself.
Transact(t, next) ==
    /\ store' = next
    /\ shown' = [shown EXCEPT ![t] = next]

-----------------------------------------------------------------------------

Results ==
    [kind : {"Ok", "Conflict"}, rev : Nat] \cup {[kind |-> "Transient", rev |-> 0]}

TypeOK ==
    /\ alive \in [Tabs -> BOOLEAN]
    /\ lock \in Tabs \cup {NoTab}
    /\ leading \in [Tabs -> {"no", "taking", "yes"}]
    /\ drain \in [Tabs -> {"off", "begin", "sending"}]
    /\ drainAgain \in [Tabs -> BOOLEAN]
    /\ response \in [Tabs -> Results \cup {NoResult}]
    /\ inbox \in [Tabs -> BOOLEAN]
    /\ edits \in 0..MaxEdits

Init ==
    /\ store = [lane |-> NoLane, rev |-> 0]
    /\ server = [content |-> 0, rev |-> 0]
    /\ net = {}
    /\ alive = [t \in Tabs |-> TRUE]
    /\ shown = [t \in Tabs |-> store]
    /\ lock = NoTab
    /\ leading = [t \in Tabs |-> "no"]
    /\ drain = [t \in Tabs |-> "off"]
    /\ drainAgain = [t \in Tabs |-> FALSE]
    /\ request = [t \in Tabs |-> NoRequest]
    /\ response = [t \in Tabs |-> NoResult]
    /\ inbox = [t \in Tabs |-> FALSE]
    /\ edits = 0
    /\ discarded = FALSE
    /\ closes = 0
    /\ resolves = 0
    /\ nextId = 1

-----------------------------------------------------------------------------
(* User actions go through `mutate`: one `transact`, then announce and     *)
(* request a drain. The user picks a resolution from what the tab shows.   *)

UserChanges(t) ==
    LET lane == shown[t].lane
    IN  (IF edits < MaxEdits THEN {[kind |-> "Edit", rev |-> 0]} ELSE {})
        \cup
        (IF resolves < MaxResolves /\ lane.present
         THEN {[kind |-> "Discard", rev |-> 0]}
              \cup (IF lane.status \in {"Waiting", "NeedsMerge"}
                    THEN {[kind |-> "Retry", rev |-> 0]} ELSE {})
              \cup (IF lane.status = "NeedsMerge"
                    \* conflictFlow fetches the server copy before resolving.
                    THEN {[kind |-> "Merge", rev |-> server.rev]} ELSE {})
         ELSE {})

Mutate(t) ==
    /\ t \in UserTabs /\ alive[t]
    /\ \E change \in UserChanges(t) :
        /\ IF change.kind = "Edit"
           THEN /\ Transact(t, Enqueue(store, [id |-> nextId, content |-> edits + 1]))
                /\ edits' = edits + 1
                /\ nextId' = nextId + 1
                /\ discarded' = FALSE
                /\ UNCHANGED resolves
           ELSE /\ Transact(t, Resolve(store, change))
                /\ resolves' = resolves + 1
                /\ discarded' = (discarded \/ change.kind = "Discard")
                /\ UNCHANGED <<edits, nextId>>
    /\ inbox' = Announce(t)
    /\ drain' = DrainAfterRequest(t)
    /\ drainAgain' = DrainAgainAfterRequest(t)
    /\ UNCHANGED <<server, net, alive, lock, leading, request, response, closes>>

-----------------------------------------------------------------------------
(* The leader's drain loop, `drainOnce`.                                   *)

\* `transact(begin_send)`, then `sendMessage` puts the request on the wire.
\* With nothing to send the loop ends, unless a drain was requested meanwhile.
Begin(t) ==
    /\ alive[t] /\ drain[t] = "begin"
    /\ \E due \in BOOLEAN :
        IF Eligible(store, due)
        THEN /\ Transact(t, BeginSendQueue(store))
             /\ request' = [request EXCEPT ![t] = RequestOf(store)]
             /\ response' = [response EXCEPT ![t] = NoResult]
             /\ net' = net \cup {RequestOf(store)}
             /\ drain' = [drain EXCEPT ![t] = "sending"]
             /\ UNCHANGED drainAgain
        ELSE /\ shown' = [shown EXCEPT ![t] = store]
             /\ drain' = [drain EXCEPT ![t] = IF drainAgain[t] THEN "begin" ELSE "off"]
             /\ drainAgain' = [drainAgain EXCEPT ![t] = FALSE]
             /\ UNCHANGED <<store, request, response, net>>
    /\ UNCHANGED <<server, alive, lock, leading, inbox, edits, discarded,
                   closes, resolves, nextId>>

\* The server handles any request ever sent, any number of times (a delayed
\* or duplicated delivery). Content saves are a CAS on the revision
\* (update_file_content.sql). On a revision conflict, sendMessage.ts
\* `classifyRevisionConflict` turns "the server already holds our text" into
\* Ok. Every tab awaiting this very request gets the response.
ServerHandle ==
    \E r \in net :
        LET applied == server.rev = r.expected
            result == CASE applied -> [kind |-> "Ok", rev |-> server.rev + 1]
                        [] server.content = r.content -> [kind |-> "Ok", rev |-> server.rev]
                        [] OTHER -> [kind |-> "Conflict", rev |-> server.rev]
        IN /\ server' = IF applied THEN [content |-> r.content, rev |-> server.rev + 1]
                        ELSE server
           /\ response' = [t \in Tabs |->
                 IF alive[t] /\ drain[t] = "sending" /\ request[t] = r
                    /\ response[t] = NoResult
                 THEN result ELSE response[t]]
    /\ UNCHANGED <<store, net, alive, shown, lock, leading, drain, drainAgain,
                   request, inbox, edits, discarded, closes, resolves, nextId>>

\* `transact(on_result)`.
ApplyResult(t, result) ==
    /\ Transact(t, OnResult(store, request[t].id, result))
    /\ drain' = [drain EXCEPT ![t] = "begin"]
    /\ request' = [request EXCEPT ![t] = NoRequest]
    /\ response' = [response EXCEPT ![t] = NoResult]

Receive(t) ==
    /\ alive[t] /\ drain[t] = "sending" /\ response[t] # NoResult
    /\ ApplyResult(t, response[t])
    /\ UNCHANGED <<server, net, alive, lock, leading, drainAgain, inbox, edits,
                   discarded, closes, resolves, nextId>>

\* The request or its response was lost: the send times out (Transient).
Timeout(t) ==
    /\ alive[t] /\ drain[t] = "sending"
    /\ ApplyResult(t, [kind |-> "Transient", rev |-> 0])
    /\ UNCHANGED <<server, net, alive, lock, leading, drainAgain, inbox, edits,
                   discarded, closes, resolves, nextId>>

\* The scheduler's wake-up for a `Waiting` lane.
Wake(t) ==
    /\ alive[t] /\ leading[t] = "yes"
    /\ store.lane.present /\ store.lane.status = "Waiting"
    /\ drain' = DrainAfterRequest(t)
    /\ drainAgain' = DrainAgainAfterRequest(t)
    /\ UNCHANGED <<store, server, net, alive, shown, lock, leading, request,
                   response, inbox, edits, discarded, closes, resolves, nextId>>

-----------------------------------------------------------------------------
(* Cross-tab coordination.                                                 *)

\* A `BroadcastChannel` announcement arrives: the tab re-reads the stored
\* queue to show it, and the leader drains it (`adoptExternalChange`).
Hear(t) ==
    /\ alive[t] /\ inbox[t]
    /\ inbox' = [inbox EXCEPT ![t] = FALSE]
    /\ shown' = [shown EXCEPT ![t] = store]
    /\ drain' = DrainAfterRequest(t)
    /\ drainAgain' = DrainAgainAfterRequest(t)
    /\ UNCHANGED <<store, server, net, alive, lock, leading, request, response,
                   edits, discarded, closes, resolves, nextId>>

\* outboxLock.ts: the browser grants the free leader lock to a waiting tab ...
Acquire(t) ==
    /\ alive[t] /\ lock = NoTab /\ leading[t] = "no"
    /\ lock' = t
    /\ leading' = [leading EXCEPT ![t] = "taking"]
    /\ UNCHANGED <<store, server, net, alive, shown, drain, drainAgain, request,
                   response, inbox, edits, discarded, closes, resolves, nextId>>

\* ... whose task recovers the previous leader's interrupted work, then
\* leads and drains.
BecomeLeader(t) ==
    /\ alive[t] /\ leading[t] = "taking"
    /\ Transact(t, Recover(store))
    /\ leading' = [leading EXCEPT ![t] = "yes"]
    /\ drain' = [drain EXCEPT ![t] = "begin"]
    /\ UNCHANGED <<server, net, alive, lock, drainAgain, request, response,
                   inbox, edits, discarded, closes, resolves, nextId>>

\* Closing a tab ends all of its work at once and frees the lock. Requests
\* it sent stay on the wire.
Close(t) ==
    /\ alive[t] /\ closes < MaxCloses
    /\ closes' = closes + 1
    /\ alive' = [alive EXCEPT ![t] = FALSE]
    /\ lock' = IF lock = t THEN NoTab ELSE lock
    /\ leading' = [leading EXCEPT ![t] = "no"]
    /\ drain' = [drain EXCEPT ![t] = "off"]
    /\ drainAgain' = [drainAgain EXCEPT ![t] = FALSE]
    /\ request' = [request EXCEPT ![t] = NoRequest]
    /\ response' = [response EXCEPT ![t] = NoResult]
    /\ inbox' = [inbox EXCEPT ![t] = FALSE]
    /\ UNCHANGED <<store, server, net, shown, edits, discarded, resolves, nextId>>

\* Reopening: `start()` reads the stored queue to show it.
Reopen(t) ==
    /\ ~alive[t]
    /\ alive' = [alive EXCEPT ![t] = TRUE]
    /\ shown' = [shown EXCEPT ![t] = store]
    /\ UNCHANGED <<store, server, net, lock, leading, drain, drainAgain,
                   request, response, inbox, edits, discarded, closes,
                   resolves, nextId>>

-----------------------------------------------------------------------------

Next ==
    \/ ServerHandle
    \/ \E t \in Tabs :
        \/ Mutate(t)
        \/ Begin(t) \/ Receive(t) \/ Timeout(t) \/ Wake(t)
        \/ Hear(t) \/ Acquire(t) \/ BecomeLeader(t) \/ Close(t) \/ Reopen(t)

Spec == Init /\ [][Next]_vars

Symmetry == Permutations(Tabs)

-----------------------------------------------------------------------------
(* Properties.                                                             *)

\* The user's latest content is on the server, still queued in the store, or
\* the user discarded it. It is never just gone.
NoLostEdit ==
    \/ edits = 0
    \/ discarded
    \/ server.content = edits
    \/ /\ store.lane.present
       /\ \E i \in DOMAIN store.lane.msgs : store.lane.msgs[i].content = edits

\* The server never goes back to older content.
ServerContentMonotone == [][server'.content >= server.content]_server

\* At most one open tab runs the drain loop.
SingleDrainer ==
    \A t, u \in Tabs : (alive[t] /\ alive[u] /\ drain[t] # "off" /\ drain[u] # "off") => t = u

=============================================================================
