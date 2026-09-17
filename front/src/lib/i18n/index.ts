import i18next from 'i18next'
import resourcesToBackend from 'i18next-resources-to-backend'
import { initReactI18next } from 'react-i18next'
import { BUNDLED_RESOURCES, loadCatalog } from './catalogs.ts'
import { buildInitOptions } from './config.ts'
import {
  FALLBACK_LOCALE,
  normalizeLocale,
  pickSupportedLocale,
  type SupportedLocale,
} from './locales.ts'
import { resolveLocale, writeStoredLocale, type LocalePreferences } from './resolve-locale.ts'

export { DEFAULT_NAMESPACE, FALLBACK_LOCALE, SUPPORTED_LOCALES, isSupportedLocale } from './locales.ts'
export type { SupportedLocale } from './locales.ts'
export type { LocalePreferences } from './resolve-locale.ts'

function syncDocumentLocale(locale: SupportedLocale): void {
  if (typeof document === 'undefined') return
  document.documentElement.lang = locale
}

const initialLocale = resolveLocale()

export const i18nReady = i18next
  .use(resourcesToBackend(loadCatalog))
  .use(initReactI18next)
  .init(
    buildInitOptions({
      lng: initialLocale,
      resources: BUNDLED_RESOURCES,
      partialBundledLanguages: true,
    })
  )

syncDocumentLocale(initialLocale)

i18next.on('languageChanged', (language: string) => {
  syncDocumentLocale(normalizeLocale(language) ?? FALLBACK_LOCALE)
})

export function getActiveLocale(): SupportedLocale {
  return normalizeLocale(i18next.resolvedLanguage ?? i18next.language) ?? FALLBACK_LOCALE
}

export function translate(key: string, options?: Record<string, unknown>): string {
  return i18next.t(key, options)
}

export async function setLocale(locale: SupportedLocale): Promise<void> {
  writeStoredLocale(locale)
  await i18next.changeLanguage(locale)
}

export async function applyLocalePreferences(
  preferences: LocalePreferences
): Promise<SupportedLocale> {
  const locale = resolveLocale(preferences)

  if (pickSupportedLocale(preferences.userLocale) === locale) {
    writeStoredLocale(locale)
  }

  if (locale !== getActiveLocale()) {
    await i18next.changeLanguage(locale)
  }

  return locale
}

export async function preloadNamespaces(namespaces: string | string[]): Promise<void> {
  await i18next.loadNamespaces(namespaces)
}

export default i18next
