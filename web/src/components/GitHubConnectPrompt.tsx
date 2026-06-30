import { GitHubConnectButton } from './GitHubConnectButton'
import './GitHubConnectPrompt.css'

export function GitHubConnectPrompt() {
  return (
    <section className="github-connect-prompt" aria-label="GitHub sync">
      <p className="github-connect-prompt__text">
        Connect your GitHub account to edit scores stored in your private{' '}
        <code>jianpu-scores</code> repository.
      </p>
      <GitHubConnectButton />
    </section>
  )
}
