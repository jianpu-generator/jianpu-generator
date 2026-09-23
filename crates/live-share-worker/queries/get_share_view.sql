-- The anonymous viewer read for `GET /shares/:share_id`: follows the share
-- to its `files` row, so a viewer always sees exactly what the owner's
-- autosave last wrote -- there is no separate share-side copy to drift.
-- `trashed_at` is returned so `crate::share::to_public_doc` can treat a
-- binned file as an ended share. `owner_login` is the best-effort cached
-- display name for the "Shared by @login" attribution; LEFT JOIN + LIMIT 1
-- because an owner could in principle have zero or several identities.
SELECT s.ended_at,
       f.trashed_at,
       f.name AS filename,
       f.content,
       f.revision,
       (SELECT ui.login
        FROM user_identities ui
        WHERE ui.user_id = f.owner_user_id
        LIMIT 1) AS owner_login
FROM shares s
JOIN files f ON f.id = s.file_id
WHERE s.share_id = ?1;
