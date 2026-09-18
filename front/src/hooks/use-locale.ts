import { useCallback, useEffect, useMemo } from 'react'
import { useTranslation } from 'react-i18next'
import { useLocation, useParams } from 'react-router'
import { toast } from 'sonner'
import { apiErrorMessage } from '@/lib/api-error'
import { useGetOwnLocale, useGetRealmLocaleSettings, useUpdateOwnLocale } from '@/api/locale.api'
import {
  FALLBACK_LOCALE,
  SUPPORTED_LOCALES,
  applyLocalePreferences,
  setLocale as applyLocale,
  type SupportedLocale,
} from '@/lib/i18n'
import type { RouterParams } from '@/routes/router'
import { localeStore } from '@/store/locale.store'
import userStore from '@/store/user.store'

export interface UseLocaleResult {
  locale: SupportedLocale
  locales: readonly SupportedLocale[]
  defaultLocale: SupportedLocale
  isSaving: boolean
  setLocale: (locale: SupportedLocale) => Promise<void>
}

let pendingChoice: SupportedLocale | null = null

function realmFromPathname(pathname: string): string | null {
  return pathname.match(/^\/realms\/([^/]+)/)?.[1] ?? null
}

function useRealmName(): string | undefined {
  const { realm_name } = useParams<RouterParams>()
  const { pathname } = useLocation()

  return useMemo(() => realm_name ?? realmFromPathname(pathname) ?? undefined, [realm_name, pathname])
}

export function useLocaleSync(): void {
  const realm = useRealmName()
  const isAuthenticated = userStore((state) => state.isAuthenticated)
  const { data: realmSettings } = useGetRealmLocaleSettings({ realm })
  const { data: accountLocale } = useGetOwnLocale({ realm, enabled: isAuthenticated })

  const realmDefaultLocale = localeStore((state) => state.realmDefaultLocale)
  const realmLocales = localeStore((state) => state.realmLocales)
  const commitLocale = localeStore((state) => state.setLocale)
  const commitRealmLocales = localeStore((state) => state.setRealmLocales)

  useEffect(() => {
    if (!realmSettings) return
    commitRealmLocales(realmSettings)
  }, [realmSettings, commitRealmLocales])

  useEffect(() => {
    let cancelled = false

    const resolve = async () => {
      if (pendingChoice) return

      const resolved = await applyLocalePreferences({
        userLocale: isAuthenticated ? accountLocale : null,
        realmDefaultLocale,
      })
      const allowed =
        isAuthenticated || realmLocales.includes(resolved)
          ? resolved
          : (realmDefaultLocale ?? FALLBACK_LOCALE)

      if (allowed !== resolved) {
        await applyLocale(allowed)
      }

      if (!cancelled) commitLocale(allowed)
    }

    void resolve()

    return () => {
      cancelled = true
    }
  }, [accountLocale, commitLocale, isAuthenticated, realmDefaultLocale, realmLocales])
}

export function useLocale(): UseLocaleResult {
  useTranslation()
  useLocaleSync()

  const realm = useRealmName()
  const isAuthenticated = userStore((state) => state.isAuthenticated)
  const { mutateAsync: saveAccountLocale, isPending } = useUpdateOwnLocale()

  const locale = localeStore((state) => state.locale)
  const realmLocales = localeStore((state) => state.realmLocales)
  const realmDefaultLocale = localeStore((state) => state.realmDefaultLocale)
  const commitLocale = localeStore((state) => state.setLocale)

  const defaultLocale = realmDefaultLocale ?? FALLBACK_LOCALE

  const offeredLocales = isAuthenticated ? SUPPORTED_LOCALES : realmLocales

  const changeLocale = useCallback(
    async (next: SupportedLocale) => {
      const target = offeredLocales.includes(next) ? next : defaultLocale
      pendingChoice = target

      try {
        await applyLocale(target)
        commitLocale(target)

        if (isAuthenticated && realm && realmLocales.includes(target)) {
          await saveAccountLocale({ realm, locale: target }).catch((error: unknown) => {
            toast.error(apiErrorMessage(error))
          })
        }
      } finally {
        pendingChoice = null
      }
    },
    [
      commitLocale,
      defaultLocale,
      isAuthenticated,
      offeredLocales,
      realm,
      realmLocales,
      saveAccountLocale,
    ]
  )

  return useMemo(
    () => ({
      locale,
      locales: offeredLocales,
      defaultLocale,
      isSaving: isPending,
      setLocale: changeLocale,
    }),
    [changeLocale, defaultLocale, isPending, locale, offeredLocales]
  )
}
