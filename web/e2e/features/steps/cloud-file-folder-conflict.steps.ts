import { expect } from '@playwright/test'
import {
  type CloudWriteOutcome,
  seedCloudFile,
  tryCreateCloudFile,
  tryRenameCloudFile,
} from '../../cloudFileHelpers'
import { currentSignedInEmail } from './cloud-account.steps'
import { Then, When } from './fixtures'

let lastOutcome: CloudWriteOutcome | undefined

When('I try to create the cloud file {string}', async ({}, name: string) => {
  lastOutcome = await tryCreateCloudFile(currentSignedInEmail(), name)
})

When(
  'I try to rename the cloud file {string} to {string}',
  async ({}, from: string, to: string) => {
    // Seeding is idempotent per name, so this resolves the file's real id.
    const { id } = await seedCloudFile(currentSignedInEmail(), from)
    lastOutcome = await tryRenameCloudFile(currentSignedInEmail(), id, to)
  },
)

Then('the cloud answers {string}', async ({}, code: string) => {
  expect(lastOutcome).toBe(code)
})
