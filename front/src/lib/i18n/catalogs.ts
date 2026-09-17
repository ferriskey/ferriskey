import enCommon from '@/locales/en/common.yaml'
import { DEFAULT_NAMESPACE, FALLBACK_LOCALE } from './locales.ts'

export type Catalog = Record<string, unknown>

const catalogModules = import.meta.glob<Catalog>('/src/locales/*/*.yaml', {
  import: 'default',
})

export const BUNDLED_RESOURCES = {
  [FALLBACK_LOCALE]: { [DEFAULT_NAMESPACE]: enCommon },
}

export async function loadCatalog(locale: string, namespace: string): Promise<Catalog> {
  const loader = catalogModules[`/src/locales/${locale}/${namespace}.yaml`]

  if (!loader) {
    return {}
  }

  return await loader()
}
