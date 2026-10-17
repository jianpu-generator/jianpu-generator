-- upsert_folder_share
-- Start (or resume) sharing the folder `?3` of owner `?2`. `?1` is a freshly
-- generated share_id, only used the first time; on conflict with the unique
-- (owner_user_id, folder_prefix) index the existing row keeps its share_id
-- and is just marked live again, so a folder's link never changes.
INSERT INTO shares (share_id, owner_user_id, file_id, folder_prefix, ended_at, created_at)
VALUES (?1, ?2, NULL, ?3, NULL, ?4)
ON CONFLICT (owner_user_id, folder_prefix) DO UPDATE SET ended_at = NULL
RETURNING share_id;
