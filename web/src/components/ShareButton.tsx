import { Share1Icon } from '@radix-ui/react-icons'
import { useCallback, useState } from 'react'
import { buildShareUrl } from '../shareUrl'

interface ShareButtonProps {
  filename: string
  content: string
  className?: string
}

export function ShareButton({
  filename,
  content,
  className = 'file-tab-bar-btn',
}: ShareButtonProps) {
  const [copied, setCopied] = useState(false)

  const handleShare = useCallback(async () => {
    const url = await buildShareUrl(filename, content)
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
      className={className}
      data-testid="share-button"
      aria-label="Copy share link"
      onClick={() => {
        void handleShare()
      }}
    >
      {copied ? (
        'Link copied'
      ) : (
        <>
          <Share1Icon aria-hidden="true" />
          Share
        </>
      )}
    </button>
  )
}
