export const PLURAL_CATEGORIES = ['zero', 'one', 'two', 'few', 'many', 'other'] as const

export type PluralCategory = (typeof PLURAL_CATEGORIES)[number]

const PLURAL_SUFFIX_PATTERN = new RegExp(`^(.*)_(${PLURAL_CATEGORIES.join('|')})$`)

export type CatalogNode = string | number | boolean | null | { [key: string]: CatalogNode }

export type FlatCatalog = Record<string, string>

export type LocaleCatalogs = Record<string, FlatCatalog>

export type CatalogSet = Record<string, LocaleCatalogs>

export interface ParityFinding {
  severity: 'error' | 'warning'
  locale: string
  namespace: string
  key: string
  message: string
}

export const SOURCE_LOCALE = 'en'

export interface ParityOptions {
  sourceLocale?: string
  pluralCategoriesFor?: (locale: string) => readonly string[]
}

export function flattenCatalog(node: CatalogNode, prefix = ''): FlatCatalog {
  if (node === null || node === undefined) return {}

  if (typeof node !== 'object') {
    return prefix ? { [prefix]: String(node) } : {}
  }

  const flat: FlatCatalog = {}

  for (const [key, value] of Object.entries(node)) {
    const path = prefix ? `${prefix}.${key}` : key
    Object.assign(flat, flattenCatalog(value, path))
  }

  return flat
}

export function splitPluralKey(key: string): { base: string; category: PluralCategory } | null {
  const match = PLURAL_SUFFIX_PATTERN.exec(key)
  if (!match) return null

  return { base: match[1], category: match[2] as PluralCategory }
}

function intlPluralCategories(locale: string): readonly string[] {
  return new Intl.PluralRules(locale).resolvedOptions().pluralCategories
}

export function translationHasStarted(
  catalogs: CatalogSet,
  locale: string,
  namespace: string
): boolean {
  return Object.keys(catalogs[locale]?.[namespace] ?? {}).length > 0
}

function collectNamespaces(catalogs: CatalogSet): string[] {
  const namespaces = new Set<string>()

  for (const localeCatalogs of Object.values(catalogs)) {
    for (const namespace of Object.keys(localeCatalogs)) {
      namespaces.add(namespace)
    }
  }

  return [...namespaces].sort()
}

export function checkParity(catalogs: CatalogSet, options: ParityOptions = {}): ParityFinding[] {
  const sourceLocale = options.sourceLocale ?? SOURCE_LOCALE
  const pluralCategoriesFor = options.pluralCategoriesFor ?? intlPluralCategories
  const locales = Object.keys(catalogs).sort()
  const findings: ParityFinding[] = []

  for (const namespace of collectNamespaces(catalogs)) {
    const pluralBases = new Set<string>()

    for (const locale of locales) {
      for (const key of Object.keys(catalogs[locale]?.[namespace] ?? {})) {
        const plural = splitPluralKey(key)
        if (plural?.category === 'other') {
          pluralBases.add(plural.base)
        }
      }
    }

    const baseKeysByLocale = new Map<string, Map<string, Set<string>>>()

    for (const locale of locales) {
      const baseKeys = new Map<string, Set<string>>()

      for (const key of Object.keys(catalogs[locale]?.[namespace] ?? {})) {
        const plural = splitPluralKey(key)

        if (plural && pluralBases.has(plural.base)) {
          const categories = baseKeys.get(plural.base) ?? new Set<string>()
          categories.add(plural.category)
          baseKeys.set(plural.base, categories)
          continue
        }

        baseKeys.set(key, new Set<string>())
      }

      baseKeysByLocale.set(locale, baseKeys)
    }

    const allBaseKeys = new Set<string>()
    for (const baseKeys of baseKeysByLocale.values()) {
      for (const base of baseKeys.keys()) {
        allBaseKeys.add(base)
      }
    }

    for (const base of [...allBaseKeys].sort()) {
      for (const locale of locales) {
        const baseKeys = baseKeysByLocale.get(locale)
        const started = locale === sourceLocale || translationHasStarted(catalogs, locale, namespace)

        if (!baseKeys?.has(base)) {
          findings.push({
            severity: started ? 'error' : 'warning',
            locale,
            namespace,
            key: base,
            message: started
              ? 'key is missing from this catalogue'
              : 'catalogue is empty — translation has not started',
          })
          continue
        }

        if (!pluralBases.has(base)) continue

        const expected = new Set(pluralCategoriesFor(locale))
        const actual = baseKeys.get(base) ?? new Set<string>()

        for (const category of [...expected].sort()) {
          if (!actual.has(category)) {
            findings.push({
              severity: 'error',
              locale,
              namespace,
              key: `${base}_${category}`,
              message: `plural form is missing — ${locale} declares the category "${category}"`,
            })
          }
        }

        for (const category of [...actual].sort()) {
          if (!expected.has(category)) {
            findings.push({
              severity: 'error',
              locale,
              namespace,
              key: `${base}_${category}`,
              message: `plural form is not declared by ${locale} — allowed categories are ${[...expected].sort().join(', ')}`,
            })
          }
        }
      }
    }
  }

  return findings
}
