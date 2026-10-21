-- An email's failed sign-ins so far, if any.
SELECT failed_count,
       locked_until,
       lockout_level
FROM sign_in_lockouts
WHERE email = ?1;
