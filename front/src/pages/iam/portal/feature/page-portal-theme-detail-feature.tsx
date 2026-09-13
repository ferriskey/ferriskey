import { useMemo, useState } from 'react'
import { useNavigate, useParams } from 'react-router-dom'
import { toast } from 'sonner'
import {
  describePortalPageError,
  useActivatePortalTheme,
  useDeletePortalTheme,
  useGetActivePortalTheme,
  useGetPortalPageRequirements,
  useGetPortalThemeById,
  useUpdatePortalThemeMetadata,
} from '@/api/portal-theme.api'
import { useGetPortalLayouts } from '@/api/portal-layouts.api'
import { useRouteTabs } from '@/components/kit'
import {
  PortalThemeProvider,
  usePortalThemeContext,
} from '@/pages/iam/portal/theme-builder/context/portal-theme-context'
import type { Schemas } from '@/api/api.client'
import { RouterParams } from '@/routes/router'
import { usePortalUrls } from '../use-portal-urls'
import { requirementsByPage, statusesForTheme } from '../theme-validation'
import PagePortalThemeDetail from '../ui/page-portal-theme-detail'
import { NO_LAYOUT } from '../ui/theme-layout-tab'
import { useCrumbLabel } from '@/components/shell/crumb-store'

const THEME_TABS = [
  { key: 'theme', label: 'Theme' },
  { key: 'layout', label: 'Layout' },
  { key: 'pages', label: 'Pages' },
] as const

export default function PagePortalThemeDetailFeature() {
  const portal = usePortalUrls()
  const { realm_name, theme_id } = useParams<RouterParams & { theme_id: string }>()
  const navigate = useNavigate()
  const realm = realm_name ?? 'master'
  const themeId = theme_id ?? ''

  const { data: themeData, isLoading } = useGetPortalThemeById({ realm, themeId })
  const { data: layoutsData } = useGetPortalLayouts({ realm })
  const { data: requirementsData } = useGetPortalPageRequirements({ realm })
  const { data: activeData } = useGetActivePortalTheme({ realm, pageType: 'login' })

  const { value: tab, tabs } = useRouteTabs(portal.theme(themeId), THEME_TABS)

  const theme = themeData?.data

  useCrumbLabel(themeId, theme?.name)
  const statuses = useMemo(
    () => statusesForTheme(theme, requirementsByPage(requirementsData?.data)),
    [theme, requirementsData]
  )

  if (isLoading || !theme) {
    return (
      <PagePortalThemeDetail
        isLoading={isLoading}
        layouts={[]}
        isActive={false}
        isActivating={false}
        isSaving={false}
        tab={tab}
        tabs={tabs}
        name=''
        layoutId={NO_LAYOUT}
        savedLayoutId={NO_LAYOUT}
        statuses={statuses}
        dirtyCount={0}
        pageHref={() => ''}
        onNameChange={() => undefined}
        onLayoutChange={() => undefined}
        onActivate={() => undefined}
        onBack={() => navigate(portal.themes())}
        onDiscard={() => undefined}
        onSave={() => undefined}
        onDelete={() => undefined}
      />
    )
  }

  return (
    <PortalThemeProvider key={theme.id} initial={theme.config}>
      <ThemeDetailInner
        realm={realm}
        theme={theme}
        layouts={layoutsData?.data ?? []}
        isActive={activeData?.theme_id === theme.id}
        statuses={statuses}
        tab={tab}
        tabs={tabs}
      />
    </PortalThemeProvider>
  )
}

interface InnerProps {
  realm: string
  theme: Schemas.PortalTheme
  layouts: Schemas.PortalLayout[]
  isActive: boolean
  statuses: ReturnType<typeof statusesForTheme>
  tab: string
  tabs: ReturnType<typeof useRouteTabs>['tabs']
}

function ThemeDetailInner({
  realm,
  theme,
  layouts,
  isActive,
  statuses,
  tab,
  tabs,
}: InnerProps) {
  const portal = usePortalUrls()
  const navigate = useNavigate()
  const { theme: config, isDirty, discard, markSaved } = usePortalThemeContext()

  const savedLayoutId = theme.layout_id ?? NO_LAYOUT
  const [name, setName] = useState(theme.name)
  const [layoutId, setLayoutId] = useState(savedLayoutId)

  const { mutate: updateMetadata, isPending: isSaving } = useUpdatePortalThemeMetadata()
  const { mutate: activateTheme, isPending: isActivating } = useActivatePortalTheme()
  const { mutate: deleteTheme } = useDeletePortalTheme()

  const nameError = name.trim().length === 0 ? 'A theme needs a name.' : undefined

  const dirtyCount =
    (name !== theme.name ? 1 : 0) + (layoutId !== savedLayoutId ? 1 : 0) + (isDirty ? 1 : 0)

  const reset = () => {
    setName(theme.name)
    setLayoutId(savedLayoutId)
    discard()
  }

  const save = () => {
    if (nameError) return
    updateMetadata(
      {
        path: { realm_name: realm, theme_id: theme.id },
        body: {
          name: name.trim(),
          layout_id: layoutId === NO_LAYOUT ? null : layoutId,
          config,
        },
      },
      {
        onSuccess: () => {
          markSaved(config)
          toast.success('Portal theme saved')
        },
        onError: (error) => {
          toast.error('Failed to save portal theme', {
            description: describePortalPageError(error) ?? 'Unknown error',
          })
        },
      }
    )
  }

  const handleDelete = () => {
    deleteTheme(
      { path: { realm_name: realm, theme_id: theme.id } },
      { onSuccess: () => navigate(portal.themes()) }
    )
  }


  return (
    <PagePortalThemeDetail
      theme={theme}
      layouts={layouts}
      isLoading={false}
      isActive={isActive}
      isActivating={isActivating}
      isSaving={isSaving}
      tab={tab}
      tabs={tabs}
      name={name}
      nameError={dirtyCount > 0 ? nameError : undefined}
      layoutId={layoutId}
      savedLayoutId={savedLayoutId}
      statuses={statuses}
      dirtyCount={dirtyCount}
      pageHref={(pageType) => portal.themePage(theme.id, pageType)}
      onNameChange={setName}
      onLayoutChange={setLayoutId}
      onActivate={() => activateTheme({ path: { realm_name: realm, theme_id: theme.id } })}
      onBack={() => navigate(portal.themes())}
      onDiscard={reset}
      onSave={save}
      onDelete={handleDelete}
    />
  )
}
