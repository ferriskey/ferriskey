import i18next, { getActiveLocale, translate, type SupportedLocale } from '@/lib/i18n'

const INTL_LOCALES: Record<SupportedLocale, string> = {
  en: 'en-GB',
  'zh-CN': 'zh-CN',
}

const dateOnlyOptions: Intl.DateTimeFormatOptions = {
  day: 'numeric',
  month: 'short',
  year: 'numeric',
}

const dateTimeOptions: Intl.DateTimeFormatOptions = {
  day: 'numeric',
  month: 'short',
  year: 'numeric',
  hour: '2-digit',
  minute: '2-digit',
  hour12: false,
}

const timestampOptions: Intl.DateTimeFormatOptions = {
  day: 'numeric',
  month: 'short',
  year: 'numeric',
  hour: '2-digit',
  minute: '2-digit',
  second: '2-digit',
  hour12: false,
}

const dayMonthOptions: Intl.DateTimeFormatOptions = {
  day: 'numeric',
  month: 'short',
}

const timeOnlyOptions: Intl.DateTimeFormatOptions = {
  hour: '2-digit',
  minute: '2-digit',
  second: '2-digit',
  hour12: false,
}

const formatters = new Map<string, Intl.DateTimeFormat>()

i18next.on('languageChanged', () => formatters.clear())

const formatterFor = (name: string, options: Intl.DateTimeFormatOptions) => {
  const locale = getActiveLocale()
  const cacheKey = `${locale}:${name}`
  const cached = formatters.get(cacheKey)
  if (cached) return cached

  const created = new Intl.DateTimeFormat(INTL_LOCALES[locale], options)
  formatters.set(cacheKey, created)
  return created
}

const parse = (iso: string) => {
  const date = new Date(iso)
  return Number.isNaN(date.getTime()) ? null : date
}

export const formatDate = (iso: string) => {
  const date = parse(iso)
  return date ? formatterFor('dateOnly', dateOnlyOptions).format(date) : iso
}

export const formatDateTime = (iso: string) => {
  const date = parse(iso)
  return date ? formatterFor('dateTime', dateTimeOptions).format(date) : iso
}

export const formatTimestamp = (iso: string) => {
  const date = parse(iso)
  return date ? formatterFor('timestamp', timestampOptions).format(date) : iso
}

export const formatDayMonth = (iso: string) => {
  const date = parse(iso)
  return date ? formatterFor('dayMonth', dayMonthOptions).format(date) : iso
}

export const formatTime = (iso: string) => {
  const date = parse(iso)
  return date ? formatterFor('timeOnly', timeOnlyOptions).format(date) : iso
}

const MINUTE = 60_000
const HOUR = 60 * MINUTE
const DAY = 24 * HOUR

export const formatRelative = (iso: string, now: number = Date.now()) => {
  const date = parse(iso)
  if (!date) return iso

  const elapsed = now - date.getTime()
  if (elapsed < 0) return formatterFor('dateTime', dateTimeOptions).format(date)
  if (elapsed < MINUTE) return translate('common:relative_time.just_now')
  if (elapsed < HOUR) {
    return translate('common:relative_time.minutes', { count: Math.floor(elapsed / MINUTE) })
  }
  if (elapsed < DAY) {
    return translate('common:relative_time.hours', { count: Math.floor(elapsed / HOUR) })
  }
  if (elapsed < 7 * DAY) {
    return translate('common:relative_time.days', { count: Math.floor(elapsed / DAY) })
  }
  return formatterFor('dateOnly', dateOnlyOptions).format(date)
}
