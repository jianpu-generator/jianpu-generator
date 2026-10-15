# TODO: file tree (folders of unlimited depth)

Design drafts: file tree storage model and before/after UX (Artifact mockups, captured from the real app).
Model is tentative: file name is a slash-separated path (`Hymns/Advent/O Come.jianpu`); a folder is a shared prefix; `fileIds` stay stable.

## Decisions needed first

- [x] Delete folder: bin each file individually (with its path)
- [x] Folder share: read-only for viewers
- [x] Path length limit: same as S3 object keys, 1024 bytes of UTF-8 for the whole path (CJK names use 3 bytes per character, so count bytes, not characters)
- [ ] Copy `Demo/` into the user's own files? (not asked; default is a later follow-up, out of scope here)
- [x] Tree logic lives in Rust/wasm; TypeScript only renders and passes raw UI actions

## Phase 1: model (pure functions, tests in separate files)

- [x] Path rules: no empty segments, no `.`/`..`, a path can't be both file and folder
- [x] Derive folder tree from the flat file map; explicit empty folders (`folders` list)
- [x] Sorting: folders first, then files, alphabetical
- [x] Unique-name logic scoped to a folder
- [x] Rename/move file (keeps `fileId`)
- [x] Rename/move folder as one all-or-nothing batch (collision check)
- [x] Delete folder to bin, bin entries remember their path
- [x] Restore to original path, recreating the folder if gone
- [x] Duplicate stays in the same folder
- [x] Local persistence: optional `folders` field, no version bump
- [x] Update `ARCHITECTURE.md` (key types, glossary: folder, path)

## Phase 2: local UI (matches the real app's look)

- [ ] File list becomes a tree in the existing monospace dropdown; widen it
- [ ] Toolbar button shows the folder path as a muted prefix
- [ ] Active file's ancestors always expanded; expanded state remembered
- [ ] Demo becomes a read-only `Demo/` row, last and separated
- [ ] `···` menu: add "New folder" and "Move to…"; New creates next to the open file
- [ ] Hover `···` on folder rows: New file here, New folder here, Import here, Rename, Move to, Share folder, Delete folder
- [ ] Move to… dialog (folder picker, no moving into itself, clash message)
- [ ] Rename accepts `/` to move
- [ ] Delete folder confirmation with file count
- [ ] Bin shows each file's original location
- [ ] Empty folder row hint
- [ ] Gherkin scenarios (write failing ones first): create, nest, move, rename folder, delete, restore
- [ ] Check phone layout of the real app first, then design the tree there

## Phase 3: cloud

- [ ] Paths as names in the cloud store; uniqueness on full path
- [ ] Batch-rename message in the outbox (no half-delivered folder move)
- [ ] Explicit empty folders (marker entry or table)
- [ ] Collision-retry naming for folders
- [ ] Capture signed-in / Start Sync share states from the real app for mockups

## Phase 4: share a folder

- [ ] Share dialog for folders: static link (snapshot of the subtree) and synced link
- [ ] Static link size guard: warn and offer synced link when too big
- [ ] Synced folder: new files, edits, renames and deletions appear for viewers
- [ ] Shared marker on the folder row, "via" tag on files inside
- [ ] Shared folder menu: Copy share link, Sharing settings; nested folder shows "Shared via"
- [ ] Warn on delete of a shared folder; moving keeps the link working
- [ ] Warn when a file inside already has its own synced link
- [ ] Stop sync confirmation
- [ ] Viewer: same toolbar as today, file picker rooted at the shared folder, read-only
- [ ] Viewer: "Import this folder" (name clash becomes `Hymns 2`) and Discard
- [ ] Viewer: "no longer shared" screen; live updates without reload
- [ ] Update `syntax.md` only if `.jianpu` syntax changes (not expected)
