import { FALLBACK_LOCALE } from './locales.ts'

export type Catalog = Record<string, unknown>

const CATALOG_ROOT = '/src/locales'
const FALLBACK_ROOT = `${CATALOG_ROOT}/en/`
const CATALOG_SUFFIX = '.yaml'

const catalogModules = import.meta.glob<Catalog>('/src/locales/*/*.yaml', {
  import: 'default',
})

const fallbackModules = import.meta.glob<Catalog>('/src/locales/en/*.yaml', {
  import: 'default',
  eager: true,
})

export const BUNDLED_RESOURCES = {
  [FALLBACK_LOCALE]: Object.fromEntries(
    Object.entries(fallbackModules).map(([path, catalog]) => [
      path.slice(FALLBACK_ROOT.length, -CATALOG_SUFFIX.length),
      catalog,
    ])
  ),
}

export async function loadCatalog(locale: string, namespace: string): Promise<Catalog> {
  const loader = catalogModules[`${CATALOG_ROOT}/${locale}/${namespace}${CATALOG_SUFFIX}`]

  if (!loader) {
    return {}
  }

  return await loader()
}
