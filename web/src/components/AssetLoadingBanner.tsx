import * as Progress from '@radix-ui/react-progress'
import type { AssetStatus } from '../hooks/useAssetLoader'

const indeterminateStyle = `
@keyframes indeterminate-slide {
  0%   { transform: translateX(-100%); }
  100% { transform: translateX(400%); }
}
[data-state="indeterminate"] .progress-indicator {
  width: 30%;
  animation: indeterminate-slide 1.4s ease-in-out infinite;
}
`

interface AssetLoadingBannerProps {
  soundfontStatus: AssetStatus
  soundfontLoadedBytes: number
  soundfontTotalBytes: number
  fontsStatus: AssetStatus
  fontsLoadedBytes: number
  fontsTotalBytes: number
}

interface RowProps {
  label: string
  status: AssetStatus
  loadedBytes: number
  totalBytes: number
}

function AssetRow({ label, status, loadedBytes, totalBytes }: RowProps) {
  const percent =
    status === 'error'
      ? 100
      : totalBytes > 0
        ? Math.round((loadedBytes / totalBytes) * 100)
        : null

  return (
    <div style={{ display: 'flex', alignItems: 'center', gap: '0.75rem' }}>
      <span
        style={{ minWidth: '10rem', fontSize: '0.85rem', color: '#cdd6f4' }}
      >
        {label}
      </span>
      <Progress.Root
        value={percent}
        style={{
          flex: 1,
          height: '8px',
          background: '#313244',
          borderRadius: '4px',
          overflow: 'hidden',
        }}
      >
        <Progress.Indicator
          className="progress-indicator"
          style={{
            height: '100%',
            background: status === 'error' ? '#f38ba8' : '#89b4fa',
            borderRadius: '4px',
            transition: percent !== null ? 'transform 100ms ease' : undefined,
            transform:
              percent !== null ? `translateX(-${100 - percent}%)` : undefined,
          }}
        />
      </Progress.Root>
      <span
        style={{
          fontSize: '0.8rem',
          minWidth: '3rem',
          color: '#a6adc8',
          textAlign: 'right',
        }}
      >
        {status === 'error' ? 'Error' : percent !== null ? `${percent}%` : '…'}
      </span>
    </div>
  )
}

export function AssetLoadingBanner({
  soundfontStatus,
  soundfontLoadedBytes,
  soundfontTotalBytes,
  fontsStatus,
  fontsLoadedBytes,
  fontsTotalBytes,
}: AssetLoadingBannerProps) {
  if (soundfontStatus === 'ready' && fontsStatus === 'ready') return null

  return (
    <>
      <style>{indeterminateStyle}</style>
      <div
        style={{
          position: 'fixed',
          top: 0,
          left: 0,
          right: 0,
          zIndex: 9999,
          background: '#1e1e2e',
          borderBottom: '1px solid #313244',
          padding: '0.4rem 1rem',
          display: 'flex',
          flexDirection: 'column',
          gap: '0.25rem',
        }}
      >
        {soundfontStatus !== 'ready' && (
          <AssetRow
            label="Soundfont (choir audio)"
            status={soundfontStatus}
            loadedBytes={soundfontLoadedBytes}
            totalBytes={soundfontTotalBytes}
          />
        )}
        {fontsStatus !== 'ready' && (
          <AssetRow
            label="Fonts (PDF export)"
            status={fontsStatus}
            loadedBytes={fontsLoadedBytes}
            totalBytes={fontsTotalBytes}
          />
        )}
      </div>
    </>
  )
}
