import { useState } from 'react'
import { cloudflareAppUrl, enableGitHubSync } from '../env'
import './GitHubSyncDisabledBanner.css'

const DISMISS_STORAGE_KEY = 'jianpu:github-sync-banner:dismissed'

function readDismissed(): boolean {
  try {
    return localStorage.getItem(DISMISS_STORAGE_KEY) === 'true'
  } catch {
    return false
  }
}

export function GitHubSyncDisabledBanner() {
  const [dismissed, setDismissed] = useState(readDismissed)

  if (enableGitHubSync || dismissed) {
    return null
  }

  const dismiss = () => {
    try {
      localStorage.setItem(DISMISS_STORAGE_KEY, 'true')
    } catch {
      // Ignore storage failures; banner still hides for this session.
    }
    setDismissed(true)
  }

  return (
    <div className="github-sync-banner" role="status">
      <p className="github-sync-banner__text">
        GitHub sync requires the Cloudflare deployment.{' '}
        <a
          className="github-sync-banner__link"
          href={cloudflareAppUrl}
          target="_blank"
          rel="noopener noreferrer"
        >
          Open the Cloudflare app
        </a>
      </p>
      <button
        type="button"
        className="github-sync-banner__dismiss"
        onClick={dismiss}
        aria-label="Dismiss GitHub sync notice"
      >
        Dismiss
      </button>
    </div>
  )
}
