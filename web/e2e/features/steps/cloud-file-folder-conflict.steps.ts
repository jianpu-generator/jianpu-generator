import { expect } from '@playwright/test'
import {
  type CloudWriteOutcome,
  seedCloudFile,
  tryCreateCloudFile,
  tryRenameCloudFile,
} from '../../cloudFileHelpers'
import { currentSignedInLogin } from './cloud-account.steps'
import { Then, When } from './fixtures'

let lastOutcome: CloudWriteOutcome | undefined

When('I try to create the cloud file {string}', async ({}, name: string) => {
  lastOutcome = await tryCreateCloudFile(currentSignedInLogin(), name)
})

When(
  'I try to rename the cloud file {string} to {string}',
  async ({}, from: string, to: string) => {
    // Seeding is idempotent per name, so this resolves the file's real id.
    const { id } = await seedCloudFile(currentSignedInLogin(), from)
    lastOutcome = await tryRenameCloudFile(currentSignedInLogin(), id, to)
  },
)

Then('the cloud answers {string}', async ({}, code: string) => {
  expect(lastOutcome).toBe(code)
})
