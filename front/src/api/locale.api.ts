import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { SUPPORTED_LOCALES, normalizeLocale, type SupportedLocale } from '@/lib/i18n/locales'

export const OWN_LOCALE_PATH = '/realms/{realm_name}/users/@me/locale'

export interface RealmLocaleSettings {
  defaultLocale: SupportedLocale | null
  supportedLocales: readonly SupportedLocale[]
}

export interface RealmLocaleQuery {
  realm?: string
}

export interface OwnLocaleQuery {
  realm?: string
  enabled?: boolean
}

export interface UpdateOwnLocaleVariables {
  realm: string
  locale: SupportedLocale | null
}

interface LocaleRequestClient {
  request: (
    method: 'put',
    path: string,
    params: { path: { realm_name: string }; body: { locale: string | null } }
  ) => Promise<unknown>
}

export function readRealmLocaleSettings(payload: unknown): RealmLocaleSettings {
  const settings = payload as
    | { default_locale?: unknown; supported_locales?: unknown }
    | null
    | undefined

  const rawSupported = settings?.supported_locales
  const supported = (Array.isArray(rawSupported) ? rawSupported : [])
    .map((entry) => normalizeLocale(entry))
    .filter((entry): entry is SupportedLocale => entry !== null)

  return {
    defaultLocale: normalizeLocale(settings?.default_locale),
    supportedLocales: SUPPORTED_LOCALES.filter((locale) => supported.includes(locale)),
  }
}

export function readOwnLocale(payload: unknown): SupportedLocale | null {
  const profile = payload as { data?: { locale?: unknown } | null } | null | undefined
  return normalizeLocale(profile?.data?.locale)
}

export const useGetRealmLocaleSettings = ({ realm }: RealmLocaleQuery) => {
  return useQuery({
    ...window.tanstackApi.get('/realms/{name}/login-settings', {
      path: {
        name: realm!,
      },
    }).queryOptions,
    enabled: !!realm,
    select: readRealmLocaleSettings,
  })
}

export const useGetOwnLocale = ({ realm, enabled }: OwnLocaleQuery) => {
  return useQuery({
    ...window.tanstackApi.get('/realms/{realm_name}/users/me', {
      path: {
        realm_name: realm!,
      },
    }).queryOptions,
    enabled: !!realm && enabled !== false,
    select: readOwnLocale,
  })
}

export const useUpdateOwnLocale = () => {
  const queryClient = useQueryClient()

  return useMutation({
    mutationKey: [{ method: 'put', path: OWN_LOCALE_PATH }],
    mutationFn: async ({ realm, locale }: UpdateOwnLocaleVariables) => {
      const client = window.api as unknown as LocaleRequestClient

      return client.request('put', OWN_LOCALE_PATH, {
        path: { realm_name: realm },
        body: { locale },
      })
    },
    onSuccess: async (_response, variables) => {
      const keys = window.tanstackApi.get('/realms/{realm_name}/users/me', {
        path: {
          realm_name: variables.realm,
        },
      }).queryKey

      await queryClient.invalidateQueries({ queryKey: keys })
    },
  })
}
