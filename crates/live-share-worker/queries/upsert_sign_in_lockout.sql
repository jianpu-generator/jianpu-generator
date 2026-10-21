-- Records an email's failed sign-ins after one more wrong password.
INSERT INTO sign_in_lockouts (email, failed_count, locked_until, lockout_level)
VALUES (?1, ?2, ?3, ?4)
ON CONFLICT (email) DO UPDATE SET
    failed_count = excluded.failed_count,
    locked_until = excluded.locked_until,
    lockout_level = excluded.lockout_level;
