export type LicenseReport = Record<string, { name: string; versions: string[] }[]>

export type LicensePolicy = {
  allow: string[]
  exceptions: { name: RegExp; allow: string[] }[]
}

export type LicenseFinding = { name: string; versions: string[]; license: string }

export const CNCF_POLICY: LicensePolicy = {
  allow: [
    'Apache-2.0',
    '0BSD',
    'BSD-2-Clause',
    'BSD-2-Clause-FreeBSD',
    'BSD-3-Clause',
    'MIT',
    'MIT-0',
    'ISC',
    'OpenSSL',
    'OpenSSL-standalone',
    'PSF-2.0',
    'Python-2.0',
    'Python-2.0.1',
    'PostgreSQL',
    'SSLeay-standalone',
    'UPL-1.0',
    'X11',
    'Zlib',
    'Unicode-3.0',
  ],
  exceptions: [
    { name: /^lightningcss(-.+)?$/, allow: ['MPL-2.0'] },
    { name: /^@fontsource-variable\/geist$/, allow: ['OFL-1.1'] },
  ],
}

export function isExpressionAllowed(expression: string, allow: string[]): boolean {
  let flat = expression.trim()
  if (flat.startsWith('(') && flat.endsWith(')')) flat = flat.slice(1, -1).trim()
  if (flat === '' || /[()]/.test(flat)) return false

  return flat
    .split(/\s+OR\s+/)
    .some((alternative) => alternative.split(/\s+AND\s+/).every((id) => allow.includes(id.trim())))
}

export function parseLicenseReport(output: string): LicenseReport {
  const parsed: unknown = JSON.parse(output)
  const isReport =
    typeof parsed === 'object' &&
    parsed !== null &&
    !Array.isArray(parsed) &&
    Object.values(parsed).every(
      (packages) =>
        Array.isArray(packages) &&
        packages.every((pkg) => typeof pkg?.name === 'string' && Array.isArray(pkg?.versions))
    )
  if (!isReport)
    throw new Error(`pnpm licenses returned an unexpected payload: ${output.slice(0, 200)}`)
  return parsed as LicenseReport
}

export function checkLicenses(report: LicenseReport, policy: LicensePolicy): LicenseFinding[] {
  const findings: LicenseFinding[] = []

  for (const [license, packages] of Object.entries(report)) {
    if (isExpressionAllowed(license, policy.allow)) continue

    for (const pkg of packages) {
      const excepted = policy.exceptions.some(
        (exception) =>
          exception.name.test(pkg.name) && isExpressionAllowed(license, exception.allow)
      )
      if (!excepted) findings.push({ name: pkg.name, versions: pkg.versions, license })
    }
  }

  return findings
}
