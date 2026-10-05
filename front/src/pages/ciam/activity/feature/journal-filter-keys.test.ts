import test from 'node:test'
import assert from 'node:assert/strict'

import { JOURNAL_FILTER_KEYS, MESSAGES_JOURNAL_FILTER_KEYS } from './journal-filter-keys.ts'

test('the messages journal only accepts the filters its tabs and columns expose', () => {
  assert.deepEqual([...MESSAGES_JOURNAL_FILTER_KEYS], ['event_types', 'actor_id'])
})

test('the shared journal searches the origin instead of filtering it in a column', () => {
  assert.deepEqual([...JOURNAL_FILTER_KEYS], ['search', 'event_types', 'actor_id'])
})
