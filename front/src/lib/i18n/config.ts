import type { InitOptions, Resource } from 'i18next'
import { DEFAULT_NAMESPACE, FALLBACK_LOCALE, SUPPORTED_LOCALES } from './locales.ts'

export function humanizeKey(key: string): string {
  const leaf = key.split(':').pop()?.split('.').pop() ?? key
  const words = leaf
    .replace(/[_-]+/g, ' ')
    .replace(/([a-z\d])([A-Z])/g, '$1 $2')
    .toLowerCase()
    .trim()

  if (!words) return key

  return words.charAt(0).toUpperCase() + words.slice(1)
}

export function reportMissingKey(locales: readonly string[], namespace: string, key: string): void {
  console.warn(
    `[i18n] missing translation for "${namespace}:${key}" in ${locales.join(', ')} — falling back to ${FALLBACK_LOCALE}`
  )
}

export interface InitOptionsInput {
  lng: string
  resources?: Resource
  partialBundledLanguages?: boolean
}

export function buildInitOptions({
  lng,
  resources,
  partialBundledLanguages,
}: InitOptionsInput): InitOptions {
  return {
    lng,
    resources,
    partialBundledLanguages,
    fallbackLng: FALLBACK_LOCALE,
    supportedLngs: [...SUPPORTED_LOCALES],
    ns: [DEFAULT_NAMESPACE],
    defaultNS: DEFAULT_NAMESPACE,
    fallbackNS: DEFAULT_NAMESPACE,
    initAsync: false,
    returnEmptyString: false,
    returnNull: false,
    saveMissing: true,
    missingKeyHandler: (locales, namespace, key) => reportMissingKey(locales, namespace, key),
    parseMissingKeyHandler: (key) => humanizeKey(key),
    interpolation: {
      escapeValue: false,
    },
    react: {
      useSuspense: false,
    },
  }
}
