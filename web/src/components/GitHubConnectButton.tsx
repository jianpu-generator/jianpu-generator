import './GitHubConnectButton.css'

export interface GitHubConnectButtonProps {
  className?: string
}

export function GitHubConnectButton({ className }: GitHubConnectButtonProps) {
  return (
    <a
      className={['github-connect-button', className].filter(Boolean).join(' ')}
      href="/api/github/login"
    >
      Connect with GitHub
    </a>
  )
}
