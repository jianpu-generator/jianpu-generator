-- Best-effort cached display name for a share's owner, used only to build
-- the public "Shared by @login" viewer attribution (see `to_public_doc`).
-- A user could in principle have more than one linked identity in the
-- future (TODO §1's provider-agnostic schema); this picks any one login
-- arbitrarily since only GitHub exists today. Returns NULL both when the
-- owner has no identity row (shouldn't happen in practice) and when the
-- cached `login` itself is NULL -- both cases mean "nothing to show".
SELECT login
FROM user_identities
WHERE user_id = ?1
LIMIT 1;
