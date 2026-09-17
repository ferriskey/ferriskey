import { create } from 'zustand'
import { createJSONStorage, devtools, persist } from 'zustand/middleware'
import type { RealmLocaleSettings } from '@/api/locale.api'
import {
  SUPPORTED_LOCALES,
  getActiveLocale,
  isSupportedLocale,
  setLocale as applyLocale,
  type SupportedLocale,
} from '@/lib/i18n'
import { readQueryLocale, readStoredLocale } from '@/lib/i18n/resolve-locale'

export const LOCALE_STORE_KEY = 'ferriskey-locale'

export interface LocaleState {
  locale: SupportedLocale
  realmDefaultLocale: SupportedLocale | null
  realmLocales: readonly SupportedLocale[]
  setLocale: (locale: SupportedLocale) => void
  setRealmLocales: (settings: RealmLocaleSettings) => void
}

function mirroredLocale(persisted: unknown): SupportedLocale | null {
  const value = (persisted as { locale?: unknown } | null | undefined)?.locale
  return isSupportedLocale(value) ? value : null
}

function bootLocale(persisted: unknown): SupportedLocale {
  const active = getActiveLocale()
  const mirrored = mirroredLocale(persisted)

  if (!mirrored || mirrored === active) return active
  if (readQueryLocale() || readStoredLocale()) return active

  return mirrored
}

export const localeStore = create<LocaleState>()(
  devtools(
    persist(
      (set) => ({
        locale: getActiveLocale(),
        realmDefaultLocale: null,
        realmLocales: SUPPORTED_LOCALES,
        setLocale: (locale: SupportedLocale) => set({ locale }),
        setRealmLocales: (settings: RealmLocaleSettings) =>
          set({
            realmDefaultLocale: settings.defaultLocale,
            realmLocales:
              settings.supportedLocales.length > 0 ? settings.supportedLocales : SUPPORTED_LOCALES,
          }),
      }),
      {
        name: LOCALE_STORE_KEY,
        storage: createJSONStorage(() => localStorage),
        partialize: (state) => ({ locale: state.locale }),
        merge: (persisted, current) => ({ ...current, locale: bootLocale(persisted) }),
        onRehydrateStorage: () => (state) => {
          if (!state || state.locale === getActiveLocale()) return
          void applyLocale(state.locale)
        },
      }
    )
  )
)
