import test from 'node:test'
import assert from 'node:assert/strict'
import { readFileSync, readdirSync, statSync } from 'node:fs'
import { join } from 'node:path'
import { fileURLToPath } from 'node:url'
import { parse } from 'yaml'

const srcRoot = fileURLToPath(new URL('../..', import.meta.url))
const localesRoot = join(srcRoot, 'locales')

const OUT_OF_SCOPE = [
  'lib/i18n',
  'lib/builder-core',
  'lib/builder-mjml',
  'lib/builder-portal',
  'pages/iam/portal',
  'pages/iam/email-template',
  'api/api.client.ts',
  'api/api.tanstack.ts',
]

const PLURAL_SUFFIXES = ['zero', 'one', 'two', 'few', 'many', 'other']

function sourceFiles(dir: string, found: string[] = []): string[] {
  for (const entry of readdirSync(dir)) {
    const path = join(dir, entry)
    if (statSync(path).isDirectory()) {
      sourceFiles(path, found)
      continue
    }
    if (!/\.tsx?$/.test(path) || /\.test\.tsx?$/.test(path)) continue
    const relative = path.slice(srcRoot.length)
    if (OUT_OF_SCOPE.some((prefix) => relative.startsWith(prefix))) continue
    found.push(path)
  }
  return found
}

function flatten(node: unknown, prefix = '', into: Set<string> = new Set()): Set<string> {
  if (node === null || typeof node !== 'object') {
    if (prefix) into.add(prefix)
    return into
  }
  for (const [key, value] of Object.entries(node as Record<string, unknown>)) {
    flatten(value, prefix ? `${prefix}.${key}` : key, into)
  }
  return into
}

function catalogue(namespace: string): Set<string> {
  return flatten(parse(readFileSync(join(localesRoot, 'en', `${namespace}.yaml`), 'utf8')))
}

const catalogues = new Map<string, Set<string>>(
  readdirSync(join(localesRoot, 'en'))
    .filter((file) => file.endsWith('.yaml'))
    .map((file) => {
      const namespace = file.slice(0, -'.yaml'.length)
      return [namespace, catalogue(namespace)]
    })
)

function resolves(namespace: string, key: string): boolean {
  const keys = catalogues.get(namespace)
  if (!keys) return false
  if (keys.has(key)) return true
  return PLURAL_SUFFIXES.some((suffix) => keys.has(`${key}_${suffix}`))
}

function namespaceConstants(files: string[]): Map<string, string[]> {
  const known = new Map<string, string[]>()
  for (const path of files) {
    for (const match of readFileSync(path, 'utf8').matchAll(
      /\b([A-Z][A-Z0-9_]*NAMESPACES?)\s*(?::[^=]+)?=\s*(\[[^\]]*\]|'[^']+')/g
    )) {
      known.set(match[1], [...match[2].matchAll(/'([^']+)'/g)].map((inner) => inner[1]))
    }
  }
  return known
}

function namespacesOf(source: string, constants: Map<string, string[]>): string[] {
  const calls = [...source.matchAll(/useTranslation\(([^)]*)\)/g)].map((match) => match[1])

  const declared = calls.flatMap((call) =>
    [...call.matchAll(/'([^']+)'/g)].map((inner) => inner[1])
  )
  const viaConstant = calls.flatMap((call) =>
    [...call.matchAll(/\b([A-Z][A-Z0-9_]*)\b/g)].flatMap((inner) => constants.get(inner[1]) ?? [])
  )

  return [...new Set([...declared, ...viaConstant, 'common'])]
}

test('every literal translation key used in the code exists in the english catalogue', () => {
  const missing: string[] = []
  const files = sourceFiles(srcRoot)
  const constants = namespaceConstants(files)

  for (const path of files) {
    const source = readFileSync(path, 'utf8')
    const relative = path.slice(srcRoot.length)
    const local = namespacesOf(source, constants)

    const references = [
      ...[...source.matchAll(/\bt\(\s*'([^']+)'/g)].map((m) => m[1]),
      ...[...source.matchAll(/\btranslate\(\s*'([^']+)'/g)].map((m) => m[1]),
      ...[...source.matchAll(/i18nKey=\s*'([^']+)'/g)].map((m) => m[1]),
    ]

    for (const reference of references) {
      const [maybeNamespace, ...rest] = reference.split(':')
      const qualified = rest.length > 0

      if (qualified) {
        if (!resolves(maybeNamespace, rest.join(':'))) {
          missing.push(`${relative}  ${reference}`)
        }
        continue
      }

      const candidates = source.includes('useTranslation(')
        ? local
        : [...catalogues.keys()]

      if (!candidates.some((namespace) => resolves(namespace, reference))) {
        missing.push(`${relative}  ${reference}`)
      }
    }
  }

  assert.deepEqual(missing, [], 'keys used in code but absent from the english catalogue')
})
