import type { Page } from '@playwright/test'
import { expect } from '@playwright/test'
import createClient from 'openapi-fetch'
import type { paths } from '../../../src/generated/live-share-worker/schema'
import {
  CLOUD_WORKER_ORIGIN,
  seedTokenFor,
  workerRouteGlob,
} from '../../cloudFileHelpers'
import { currentSignedInEmail } from './cloud-account.steps'
import { Given, Then, When } from './fixtures'

const worker = createClient<paths>({ baseUrl: CLOUD_WORKER_ORIGIN })
const BATCH_ROUTE = workerRouteGlob('/files/rename-batch')
const DELIVERY_TIMEOUT_MS = 25_000

/** The statuses the worker answered batch renames with, per page. */
const batchAnswers = new WeakMap<Page, number[]>()

Given('I record the batch renames the cloud answers', async ({ page }) => {
  const answers: number[] = []
  batchAnswers.set(page, answers)
  page.on('response', (response) => {
    if (response.url() === `${CLOUD_WORKER_ORIGIN}/files/rename-batch`) {
      answers.push(response.status())
    }
  })
})

Then(
  'the cloud answered the batch renames with {string}',
  async ({ page }, statuses: string) => {
    await expect
      .poll(() => (batchAnswers.get(page) ?? []).join(', '), {
        timeout: DELIVERY_TIMEOUT_MS,
      })
      .toBe(statuses)
  },
)

When('folder moves cannot reach the cloud', async ({ page }) => {
  await page.route(BATCH_ROUTE, (route) => route.abort('connectionrefused'))
})

When('folder moves can reach the cloud again', async ({ page }) => {
  await page.unroute(BATCH_ROUTE)
  await page.evaluate(() => window.dispatchEvent(new Event('online')))
})

Then(
  '{string} does not exist on the server for the signed-in account',
  async ({}, fileName: string) => {
    const identityToken = await seedTokenFor(currentSignedInEmail())
    await expect
      .poll(
        async () => {
          const { data } = await worker.POST('/files/list', {
            body: { identityToken },
          })
          return (
            data?.files.some(
              (file) => file.name === fileName && file.trashedAt === null,
            ) ?? false
          )
        },
        { timeout: DELIVERY_TIMEOUT_MS },
      )
      .toBe(false)
  },
)
