import { useLocation, useNavigate, useParams } from 'react-router-dom'
import { OverviewHeader } from '@/components/ui/overview-header'
import type { RouterParams } from '@/routes/router'
import { PORTAL_LAYOUTS_URL } from '@/routes/sub-router/portal-layouts.router'
import { CONSOLE_THEMES_URL, PORTAL_THEMES_URL } from '@/routes/sub-router/portal-theme.router'

type PortalTab = 'themes' | 'layouts'

interface Props {
  activeTab: PortalTab
  primaryAction?: {
    label: string
    onClick: () => void
  }
}

export function PortalOverviewHeader({ activeTab, primaryAction }: Props) {
  const { realm_name } = useParams<RouterParams>()
  const navigate = useNavigate()
  const { pathname } = useLocation()
  const realm = realm_name ?? 'master'
  // In the console (CIAM) only themes are exposed (layouts stay admin-only),
  // so we drop the Layouts tab there. The admin portal keeps both tabs.
  const isConsole = pathname.includes('/console/')

  const tabs = isConsole
    ? [
        {
          key: 'themes',
          label: 'Themes',
          onClick: () => navigate(CONSOLE_THEMES_URL(realm)),
          active: activeTab === 'themes',
        },
      ]
    : [
        {
          key: 'themes',
          label: 'Themes',
          onClick: () => navigate(PORTAL_THEMES_URL(realm)),
          active: activeTab === 'themes',
        },
        {
          key: 'layouts',
          label: 'Layouts',
          onClick: () => navigate(PORTAL_LAYOUTS_URL(realm)),
          active: activeTab === 'layouts',
        },
      ]

  return (
    <OverviewHeader
      title='Portal Customization'
      description="Customize your realm's authentication portal — manage themes and layouts rendered to end users."
      primaryAction={primaryAction}
      tabs={tabs}
    />
  )
}
