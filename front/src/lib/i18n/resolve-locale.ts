import {
  FALLBACK_LOCALE,
  LOCALE_STORAGE_KEY,
  UI_LOCALES_QUERY_PARAM,
  pickSupportedLocale,
  type SupportedLocale,
} from './locales.ts'

export interface LocalePreferences {
  userLocale?: string | null
  realmDefaultLocale?: string | null
}

export function readStoredLocale(): SupportedLocale | null {
  try {
    return pickSupportedLocale(window.localStorage.getItem(LOCALE_STORAGE_KEY))
  } catch {
    return null
  }
}

export function writeStoredLocale(locale: SupportedLocale): void {
  try {
    window.localStorage.setItem(LOCALE_STORAGE_KEY, locale)
  } catch {
    return
  }
}

export function readQueryLocale(): SupportedLocale | null {
  try {
    const params = new URLSearchParams(window.location.search)
    return pickSupportedLocale(params.get(UI_LOCALES_QUERY_PARAM))
  } catch {
    return null
  }
}

export function readNavigatorLocale(): SupportedLocale | null {
  try {
    const candidates = navigator.languages?.length ? navigator.languages : [navigator.language]
    for (const candidate of candidates) {
      const locale = pickSupportedLocale(candidate)
      if (locale) return locale
    }
  } catch {
    return null
  }

  return null
}

export function resolveLocale(preferences: LocalePreferences = {}): SupportedLocale {
  const sources: Array<() => SupportedLocale | null> = [
    () => pickSupportedLocale(preferences.userLocale),
    readQueryLocale,
    readStoredLocale,
    readNavigatorLocale,
    () => pickSupportedLocale(preferences.realmDefaultLocale),
  ]

  for (const source of sources) {
    const locale = source()
    if (locale) return locale
  }

  return FALLBACK_LOCALE
}
