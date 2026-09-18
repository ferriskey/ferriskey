import type { ReactNode } from 'react'
import { useTranslation } from 'react-i18next'
import { ChevronRight, Menu } from 'lucide-react'
import { Link } from 'react-router-dom'
import { AccountMenu } from './account-menu'
import { LanguageSwitcher } from './language-switcher'
import { ThemeSwitcher } from './theme-switcher'
import { RealmPicker } from './realm-picker'

const BRAND_NAME = 'FerrisKey'

export function TopBar({
  realm,
  homeHref,
  realmHrefFor,
  children,
  onOpenNav,
}: {
  realm: string
  homeHref: string
  realmHrefFor: (realmName: string) => string
  children?: ReactNode
  onOpenNav?: () => void
}) {
  const { t } = useTranslation()

  return (
    <header className='flex h-11 shrink-0 items-center gap-2 border-b border-fk-line bg-white px-3 dark:bg-fk-surface'>
      {onOpenNav && (
        <button
          type='button'
          onClick={onOpenNav}
          aria-label={t('shell.open_navigation')}
          className='grid size-7 shrink-0 cursor-pointer place-items-center rounded-md text-neutral-500 transition-colors hover:bg-neutral-100 hover:text-neutral-900 dark:text-neutral-400 dark:hover:bg-fk-raised dark:hover:text-neutral-100'
        >
          <Menu className='size-4' />
        </button>
      )}
      <Link to={homeHref} className='flex shrink-0 items-center gap-2'>
        <img src='/logo_ferriskey.png' alt={BRAND_NAME} className='size-6 rounded' />
        <span className='hidden text-[13px] font-semibold tracking-tight sm:inline'>
          {BRAND_NAME}
        </span>
      </Link>

      <ChevronRight className='hidden size-3.5 shrink-0 text-neutral-300 sm:block dark:text-neutral-600' />

      <RealmPicker realm={realm} hrefFor={realmHrefFor} />

      {children}

      <div className='ml-auto flex items-center gap-2'>
        <LanguageSwitcher />
        <ThemeSwitcher />
        <AccountMenu />
      </div>
    </header>
  )
}
