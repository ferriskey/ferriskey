import { cn } from '@/lib/utils'
import type { RouterParams } from '@/routes/router'
import { PORTAL_LAYOUTS_URL } from '@/routes/sub-router/portal-layouts.router'
import { PORTAL_THEMES_URL } from '@/routes/sub-router/portal-theme.router'
import { LayoutTemplate, Palette } from 'lucide-react'
import { Outlet, Route, Routes, useLocation, useNavigate, useParams } from 'react-router-dom'
import PagePortalBuilderDemo from '../portal-builder-demo/page-portal-builder-demo'
import PagePortalLayouts from '../portal-layouts/page-portal-layouts'
import PagePortalThemeBuilderFeature from '../portal-theme/feature/page-portal-theme-builder-feature'
import PageThemeBuilderFeature from './themes/feature/page-theme-builder-feature'
import PageThemesListFeature from './themes/feature/page-themes-list-feature'

export default function PagePortal() {
  return (
    <Routes>
      {/* Theme builder takes the full viewport — outside the sub-nav shell.
          Every tab is addressable: /themes/:id (default), /:tab, /pages/:type. */}
      <Route path='/themes/:theme_id' element={<PageThemeBuilderFeature />} />
      <Route path='/themes/:theme_id/:section' element={<PageThemeBuilderFeature />} />
      <Route
        path='/themes/:theme_id/pages/:page_type'
        element={<PageThemeBuilderFeature />}
      />

      {/* Legacy single-theme editor + sandbox demo (kept until cleanup PR). */}
      <Route path='/theme' element={<PagePortalThemeBuilderFeature />} />
      <Route path='/builder-demo' element={<PagePortalBuilderDemo />} />

      {/* Shell with the Themes | Layouts sub-nav. */}
      <Route element={<PortalShell />}>
        <Route index element={<PageThemesListFeature />} />
        <Route path='/themes' element={<PageThemesListFeature />} />
        <Route path='/layouts/*' element={<PagePortalLayouts />} />
      </Route>
    </Routes>
  )
}

function PortalShell() {
  const { realm_name } = useParams<RouterParams>()
  const location = useLocation()
  const navigate = useNavigate()
  const realm = realm_name ?? 'master'

  const tabs = [
    {
      id: 'themes' as const,
      label: 'Themes',
      Icon: Palette,
      to: PORTAL_THEMES_URL(realm),
      isActive: location.pathname === PORTAL_THEMES_URL(realm),
    },
    {
      id: 'layouts' as const,
      label: 'Layouts',
      Icon: LayoutTemplate,
      to: PORTAL_LAYOUTS_URL(realm),
      isActive: location.pathname.startsWith(PORTAL_LAYOUTS_URL(realm)),
    },
  ]

  return (
    // `min-h-0 flex-1` instead of `h-full`: this div is a flex child of the
    // SidebarInset, which already hosts the TopBar (h-14) above us. `h-full`
    // would resolve to 100% of the inset and push total content past the
    // viewport bottom, leaving a 56px scrollable strip on the page.
    <div className='flex min-h-0 flex-1 flex-col'>
      {/* Match the RealmSwitcher row's rendered height — the
          `SidebarMenuButton size="lg"` ends up ~44px once `py-3` is added
          to the `text-sm` content's 20px line box. */}
      <div className='flex h-10 shrink-0 items-center gap-2 border-b border-border px-6 text-sm'>
        <h1 className='mr-4 font-semibold'>Portail</h1>
        <nav className='flex items-center gap-1'>
          {tabs.map(({ id, label, Icon, to, isActive }) => (
            <button
              key={id}
              type='button'
              onClick={() => navigate(to)}
              className={cn(
                'flex items-center gap-2 rounded-md px-3 py-1.5 text-sm transition-colors',
                isActive
                  ? 'bg-sidebar-primary/15 text-sidebar-primary'
                  : 'text-muted-foreground hover:bg-muted'
              )}
            >
              <Icon className='h-4 w-4' />
              <span>{label}</span>
            </button>
          ))}
        </nav>
      </div>
      {/* `min-h-0` lets this flex child shrink below content size; pair
          with `overflow-hidden` so the route (e.g. the builder, which fills
          its parent with `h-full`) can't push the page into scroll. */}
      <div className='min-h-0 flex-1 overflow-hidden'>
        <Outlet />
      </div>
    </div>
  )
}
