import test from 'node:test'
import assert from 'node:assert/strict'

import { createRefreshController } from './auth-refresh-controller'

test('refresh controller coalesces concurrent requests', async () => {
  const controller = createRefreshController()
  let calls = 0

  const factory = async () => {
    calls += 1
    await new Promise((resolve) => setTimeout(resolve, 10))
    return 'ok'
  }

  const [first, second] = await Promise.all([
    controller.run('master', factory),
    controller.run('master', factory),
  ])

  assert.equal(first, 'ok')
  assert.equal(second, 'ok')
  assert.equal(calls, 1)
})

test('refresh controller blocks retries during backoff window', async () => {
  const controller = createRefreshController(() => 1_000)
  let calls = 0

  const factory = async () => {
    calls += 1
    throw new Error('refresh failed')
  }

  await assert.rejects(controller.run('master', factory), /refresh failed/)
  await assert.rejects(controller.run('master', factory), /refresh temporarily blocked/)
  assert.equal(calls, 1)
})

test('refresh controller retries after backoff expires', async () => {
  let now = 1_000
  const controller = createRefreshController(() => now)
  let calls = 0

  const factory = async () => {
    calls += 1
    if (calls === 1) {
      throw new Error('refresh failed')
    }

    return 'ok'
  }

  await assert.rejects(controller.run('master', factory), /refresh failed/)

  now = 20_000
  const result = await controller.run('master', factory)

  assert.equal(result, 'ok')
  assert.equal(calls, 2)
})
