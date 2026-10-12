import { execFileSync } from 'node:child_process'
import { CNCF_POLICY, checkLicenses, parseLicenseReport } from './licenses.ts'

const output = execFileSync('pnpm', ['licenses', 'list', '--prod', '--json'], {
  encoding: 'utf8',
  maxBuffer: 64 * 1024 * 1024,
})
const report = parseLicenseReport(output)
const findings = checkLicenses(report, CNCF_POLICY)

for (const finding of findings) {
  console.error(
    `error  ${finding.name}@${finding.versions.join(',')}  ${finding.license} is outside the CNCF allowlist`
  )
}

const total = Object.values(report).reduce((count, packages) => count + packages.length, 0)

if (findings.length > 0) {
  console.error(`\n${findings.length} of ${total} production packages fail the license policy`)
  process.exit(1)
}

console.log(`${total} production packages pass the license policy`)
