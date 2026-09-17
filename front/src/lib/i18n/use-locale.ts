import { useTranslation } from 'react-i18next'
import { getActiveLocale, setLocale } from './index.ts'
import { SUPPORTED_LOCALES, type SupportedLocale } from './locales.ts'

export interface UseLocaleResult {
  locale: SupportedLocale
  locales: readonly SupportedLocale[]
  setLocale: (locale: SupportedLocale) => Promise<void>
}

export function useLocale(): UseLocaleResult {
  useTranslation()

  return {
    locale: getActiveLocale(),
    locales: SUPPORTED_LOCALES,
    setLocale,
  }
}
