import { useCallback, useState } from 'react'
import { buildShareUrl } from '../shareUrl'

interface ShareButtonProps {
  filename: string
  content: string
}

export function ShareButton({ filename, content }: ShareButtonProps) {
  const [copied, setCopied] = useState(false)

  const handleShare = useCallback(async () => {
    const url = buildShareUrl(filename, content)
    try {
      await navigator.clipboard.writeText(url)
      setCopied(true)
      window.setTimeout(() => setCopied(false), 2000)
    } catch {
      window.prompt('Copy this link to share:', url)
    }
  }, [filename, content])

  return (
    <button
      type="button"
      className="file-tab-bar-btn"
      data-testid="share-button"
      aria-label="Copy share link"
      onClick={() => {
        void handleShare()
      }}
    >
      {copied ? 'Link copied' : 'Share'}
    </button>
  )
}
