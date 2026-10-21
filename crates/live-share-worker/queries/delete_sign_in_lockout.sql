-- A successful sign-in clears the email's failed sign-ins.
DELETE FROM sign_in_lockouts
WHERE email = ?1;
