import { readdirSync, readFileSync, statSync } from 'node:fs'
import { join } from 'node:path'
import { fileURLToPath } from 'node:url'
import { parse } from 'yaml'
import {
  checkParity,
  flattenCatalog,
  SOURCE_LOCALE,
  translationHasStarted,
  type CatalogSet,
  type ParityFinding,
} from './parity.ts'

const localesRoot = fileURLToPath(new URL('../../locales', import.meta.url))

function readCatalogSet(root: string): CatalogSet {
  const catalogs: CatalogSet = {}

  for (const locale of readdirSync(root).sort()) {
    const localeDir = join(root, locale)
    if (!statSync(localeDir).isDirectory()) continue

    catalogs[locale] = {}

    for (const file of readdirSync(localeDir).sort()) {
      if (!file.endsWith('.yaml')) continue

      const namespace = file.slice(0, -'.yaml'.length)
      const path = join(localeDir, file)

      try {
        catalogs[locale][namespace] = flattenCatalog(parse(readFileSync(path, 'utf8')))
      } catch (error) {
        const reason = error instanceof Error ? error.message : String(error)
        console.error(`error  ${locale}/${file}  is not valid YAML — ${reason}`)
        process.exit(1)
      }
    }
  }

  return catalogs
}

function formatFinding(finding: ParityFinding): string {
  return `${finding.severity.padEnd(7)}${finding.locale}/${finding.namespace}.yaml  ${finding.key}  ${finding.message}`
}

const catalogs = readCatalogSet(localesRoot)
const locales = Object.keys(catalogs)

if (locales.length === 0) {
  console.error(`error  no locale directory found under ${localesRoot}`)
  process.exit(1)
}

const findings = checkParity(catalogs)
const errors = findings.filter((finding) => finding.severity === 'error')
const warnings = findings.filter((finding) => finding.severity === 'warning')

const namespaces = [...new Set(Object.values(catalogs).flatMap((entry) => Object.keys(entry)))].sort()
const untranslated = locales
  .filter((locale) => locale !== SOURCE_LOCALE)
  .flatMap((locale) =>
    namespaces
      .filter((namespace) => !translationHasStarted(catalogs, locale, namespace))
      .map((namespace) => `${locale}/${namespace}.yaml`)
  )

console.log(`i18n parity — source: ${SOURCE_LOCALE}, locales: ${locales.join(', ')}`)
console.log(
  untranslated.length > 0
    ? `not started, reported only: ${untranslated.join(', ')}`
    : 'every catalogue has started — all gaps are enforced'
)

for (const finding of [...errors, ...warnings]) {
  console.log(formatFinding(finding))
}

console.log(`${errors.length} error(s), ${warnings.length} warning(s)`)

if (errors.length > 0) {
  process.exit(1)
}
