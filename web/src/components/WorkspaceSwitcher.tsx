import * as Tabs from '@radix-ui/react-tabs'
import { enableGitHubSync } from '../env'
import './WorkspaceSwitcher.css'

export type Workspace = 'local' | 'github'

export interface WorkspaceSwitcherProps {
  value: Workspace
  onValueChange: (workspace: Workspace) => void
}

export function WorkspaceSwitcher({
  value,
  onValueChange,
}: WorkspaceSwitcherProps) {
  if (!enableGitHubSync) {
    return null
  }

  return (
    <Tabs.Root
      className="workspace-switcher"
      value={value}
      onValueChange={(nextValue) => onValueChange(nextValue as Workspace)}
    >
      <Tabs.List className="workspace-switcher__list" aria-label="Workspace">
        <Tabs.Trigger className="workspace-switcher__trigger" value="local">
          Local
        </Tabs.Trigger>
        <Tabs.Trigger className="workspace-switcher__trigger" value="github">
          GitHub
        </Tabs.Trigger>
      </Tabs.List>
    </Tabs.Root>
  )
}
