import assert from 'node:assert/strict'
import { test } from 'node:test'
import {
  CNCF_POLICY,
  checkLicenses,
  isExpressionAllowed,
  parseLicenseReport,
  type LicensePolicy,
} from './licenses.ts'

const policy: LicensePolicy = {
  allow: ['MIT', 'ISC', 'Apache-2.0', 'BSD-3-Clause'],
  exceptions: [
    { name: /^lightningcss(-.+)?$/, allow: ['MPL-2.0'] },
    { name: /^@fontsource-variable\/geist$/, allow: ['OFL-1.1'] },
  ],
}

test('an allowed license passes', () => {
  assert.deepEqual(checkLicenses({ MIT: [{ name: 'react', versions: ['19.0.0'] }] }, policy), [])
})

test('a license outside the allowlist is reported with the package', () => {
  const findings = checkLicenses({ 'GPL-3.0': [{ name: 'evil', versions: ['1.0.0'] }] }, policy)
  assert.deepEqual(findings, [{ name: 'evil', versions: ['1.0.0'], license: 'GPL-3.0' }])
})

test('AND needs every part allowed, OR needs one alternative', () => {
  assert.equal(isExpressionAllowed('(Apache-2.0 AND BSD-3-Clause)', policy.allow), true)
  assert.equal(isExpressionAllowed('MIT AND GPL-3.0', policy.allow), false)
  assert.equal(isExpressionAllowed('(MIT OR GPL-3.0)', policy.allow), true)
  assert.equal(isExpressionAllowed('GPL-3.0 OR AGPL-3.0', policy.allow), false)
})

test('an exception covers only its package and its license', () => {
  const report = {
    'MPL-2.0': [
      { name: 'lightningcss', versions: ['1.31.1'] },
      { name: 'lightningcss-linux-x64-gnu', versions: ['1.31.1'] },
      { name: 'other-mpl', versions: ['2.0.0'] },
    ],
    'OFL-1.1': [{ name: 'lightningcss', versions: ['1.31.1'] }],
  }
  assert.deepEqual(checkLicenses(report, policy), [
    { name: 'other-mpl', versions: ['2.0.0'], license: 'MPL-2.0' },
    { name: 'lightningcss', versions: ['1.31.1'], license: 'OFL-1.1' },
  ])
})

test('the self-hosted console fonts are the only packages allowed under OFL-1.1', () => {
  const report = {
    'OFL-1.1': [
      { name: '@fontsource-variable/geist', versions: ['5.2.8'] },
      { name: '@fontsource/ibm-plex-sans', versions: ['5.3.0'] },
      { name: '@fontsource/other-font', versions: ['1.0.0'] },
    ],
  }
  assert.deepEqual(checkLicenses(report, CNCF_POLICY), [
    { name: '@fontsource/other-font', versions: ['1.0.0'], license: 'OFL-1.1' },
  ])
})

test('an unknown or nested expression fails closed', () => {
  assert.equal(isExpressionAllowed('Unknown', policy.allow), false)
  assert.equal(isExpressionAllowed('', policy.allow), false)
  assert.equal(isExpressionAllowed('MIT AND (ISC OR GPL-3.0)', policy.allow), false)
})

test('a pnpm error payload is rejected instead of passing', () => {
  assert.throws(() =>
    parseLicenseReport('{"error":{"code":"ERR_PNPM_MISSING_PACKAGE_INDEX_FILE"}}')
  )
  assert.throws(() => parseLicenseReport('[]'))
  assert.deepEqual(parseLicenseReport('{"MIT":[{"name":"react","versions":["19.0.0"]}]}'), {
    MIT: [{ name: 'react', versions: ['19.0.0'] }],
  })
})
