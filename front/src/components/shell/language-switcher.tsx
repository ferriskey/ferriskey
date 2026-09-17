import { useTranslation } from 'react-i18next'
import { useLocale } from '@/lib/i18n/use-locale'
import { cn } from '@/lib/utils'

export function LanguageSwitcher() {
  const { t } = useTranslation()
  const { locale, locales, setLocale } = useLocale()

  return (
    <div
      role='radiogroup'
      aria-label={t('language.switcher_label')}
      className='flex items-center gap-0.5 rounded-md border border-fk-line p-0.5'
    >
      {locales.map((option) => {
        const active = locale === option
        const name = t(`language.names.${option}`, { lng: option })

        return (
          <button
            key={option}
            type='button'
            role='radio'
            lang={option}
            aria-checked={active}
            onClick={() => void setLocale(option)}
            className={cn(
              'h-6 cursor-pointer rounded px-1.5 text-[11px] font-medium transition-colors',
              active
                ? 'bg-fk-primary-soft text-fk-primary-text'
                : 'text-neutral-400 hover:text-neutral-700 dark:text-neutral-500 dark:hover:text-neutral-300'
            )}
          >
            {name}
          </button>
        )
      })}
    </div>
  )
}
