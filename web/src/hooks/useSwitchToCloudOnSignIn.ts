import { useEffect, useRef } from 'react'
import type { StoredAccountAuth } from '../storage/accountAuth'
import type { StorageBackendPreference } from './useStorageBackend'

/**
 * Switches `preference` to `'cloud'` when a session appears where there was
 * none, the way signing out switches it back to `'local'` (see `App.tsx`).
 * A session already present when the app loads leaves `preference` alone, so
 * a user who signed in earlier and then picked "This browser" stays local.
 */
export function useSwitchToCloudOnSignIn(
  accountAuth: StoredAccountAuth | null,
  setPreference: (preference: StorageBackendPreference) => void,
): void {
  const signedInRef = useRef(accountAuth !== null)
  useEffect(() => {
    const signedIn = accountAuth !== null
    if (signedIn && !signedInRef.current) setPreference({ backend: 'cloud' })
    signedInRef.current = signedIn
  }, [accountAuth, setPreference])
}
