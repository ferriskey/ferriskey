import { Laptop, Moon, Sun } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip'
import { useTheme } from '@/components/theme-provider'
import { cn } from '@/lib/utils'

const options = [
  { value: 'light', icon: Sun },
  { value: 'dark', icon: Moon },
  { value: 'system', icon: Laptop },
] as const

function switchWithoutAnimating(apply: () => void) {
  const root = document.documentElement
  root.classList.add('theme-switching')
  apply()
  requestAnimationFrame(() =>
    requestAnimationFrame(() => root.classList.remove('theme-switching'))
  )
}

export function ThemeSwitcher() {
  const { t } = useTranslation()
  const { theme, setTheme } = useTheme()

  return (
    <div
      role='radiogroup'
      aria-label={t('shell.theme.label')}
      className='flex items-center gap-0.5 rounded-md border border-fk-line p-0.5'
    >
      {options.map((option) => {
        const active = theme === option.value
        const label = t(`shell.theme.${option.value}`)
        return (
          <Tooltip key={option.value}>
            <TooltipTrigger asChild>
              <button
                type='button'
                role='radio'
                aria-checked={active}
                aria-label={label}
                onClick={() => switchWithoutAnimating(() => setTheme(option.value))}
                className={cn(
                  'grid size-6 cursor-pointer place-items-center rounded transition-colors',
                  active
                    ? 'bg-fk-primary-soft text-fk-primary-text'
                    : 'text-neutral-400 hover:text-neutral-700 dark:text-neutral-500 dark:hover:text-neutral-300'
                )}
              >
                <option.icon className='size-3.5' strokeWidth={1.75} />
              </button>
            </TooltipTrigger>
            <TooltipContent side='bottom'>{label}</TooltipContent>
          </Tooltip>
        )
      })}
    </div>
  )
}
