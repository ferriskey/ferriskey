export const SUPPORTED_LOCALES = ['en', 'zh-CN'] as const

export type SupportedLocale = (typeof SUPPORTED_LOCALES)[number]

export const FALLBACK_LOCALE: SupportedLocale = 'en'

export const DEFAULT_NAMESPACE = 'common'

export const LOCALE_STORAGE_KEY = 'ferriskey-ui-locale'

export const UI_LOCALES_QUERY_PARAM = 'ui_locales'

const LOCALE_ALIASES: Record<string, SupportedLocale> = {
  zh: 'zh-CN',
  cmn: 'zh-CN',
  'zh-cn': 'zh-CN',
  'zh-sg': 'zh-CN',
  'zh-my': 'zh-CN',
  'zh-hans': 'zh-CN',
  'zh-hans-cn': 'zh-CN',
  'zh-hans-sg': 'zh-CN',
}

export function isSupportedLocale(value: unknown): value is SupportedLocale {
  return SUPPORTED_LOCALES.some((locale) => locale === value)
}

export function normalizeLocale(value: unknown): SupportedLocale | null {
  if (typeof value !== 'string') return null

  const tag = value.trim().toLowerCase().replace(/_/g, '-')
  if (!tag) return null

  const exact = SUPPORTED_LOCALES.find((locale) => locale.toLowerCase() === tag)
  if (exact) return exact

  const alias = LOCALE_ALIASES[tag]
  if (alias) return alias

  const primary = tag.split('-')[0]
  const byPrimary = SUPPORTED_LOCALES.find(
    (locale) => locale.toLowerCase().split('-')[0] === primary
  )
  if (byPrimary) return byPrimary

  return LOCALE_ALIASES[primary] ?? null
}

export function pickSupportedLocale(value: unknown): SupportedLocale | null {
  if (typeof value !== 'string') return null

  for (const candidate of value.split(/[\s,]+/)) {
    const locale = normalizeLocale(candidate.split(';')[0])
    if (locale) return locale
  }

  return null
}
