import { useLocation, useParams } from 'react-router'
import { useTranslation } from 'react-i18next'
import { ACCOUNT_URL, RouterParams } from '@/routes/router'

export function useAccountTabs() {
  const { t } = useTranslation('account')
  const { realm_name } = useParams<RouterParams>()
  const { pathname } = useLocation()

  const base = ACCOUNT_URL(realm_name)

  const tabs = [
    { key: 'overview', label: t('tabs.overview'), href: base },
    { key: 'security', label: t('tabs.security'), href: `${base}/security` },
    { key: 'sessions', label: t('tabs.sessions'), href: `${base}/sessions` },
  ]

  const tab = tabs.slice(1).find(({ href }) => pathname.endsWith(href.slice(base.length)))

  return { tabs, tab: tab?.key ?? 'overview' }
}
