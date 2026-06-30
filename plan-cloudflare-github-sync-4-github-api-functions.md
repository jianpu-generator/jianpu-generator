# Step 4: GitHub repo API Pages Functions

## Goal
Proxy GitHub Contents API for `jianpu-scores` repo using session cookie from step 3. Last-write-wins via SHA.

## Depends on
- Step 2: `web/src/github/manifest.ts` path helpers and manifest shape
- Step 3: session auth middleware

## Routes

| Route | Method | Purpose |
|-------|--------|---------|
| `/api/github/store` | GET | Load manifest + all score/bin file contents → JSON matching client needs |
| `/api/github/files/[...path]` | GET | Read one file under scores/ or bin/ |
| `/api/github/files/[...path]` | PUT | Write file body JSON `{ content, sha? }`; fetch SHA if missing; last-write-wins |
| `/api/github/manifest` | PATCH | Update manifest (active, fileIds, bin list ops from client) |

## Behavior
- All routes require valid session; 401 if not connected
- `GET /store`: read `.jianpu/manifest.json`, list/read `scores/*` and `bin/*`
- Initial empty repo: return empty manifest + empty files (client creates tabs)
- Use GitHub REST API with user's token from session
- Reuse repo name `jianpu-scores` from session

## Verification
Manual curl with session cookie after OAuth, or unit tests for pure GitHub API helpers if extracted.

## Out of scope
- Frontend hooks (step 5)
- Autosave debounce (step 6)
