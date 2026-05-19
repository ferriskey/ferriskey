import { useNavigate, useParams } from 'react-router-dom'
import { BasicSpinner } from '@/components/ui/spinner'
import {
  useActivatePortalTheme,
  useGetPortalThemeById,
  useUpdatePortalThemeMetadata,
  useUpdatePortalThemePage,
} from '@/api/portal-theme.api'
import { useGetPortalLayouts } from '@/api/portal-layouts.api'
import type { RouterParams } from '@/routes/router'
import { PORTAL_THEMES_URL } from '@/routes/sub-router/portal-theme.router'
import { PortalThemeProvider } from '@/pages/portal-theme/context/portal-theme-context'
import PageThemeBuilder from '../ui/page-theme-builder'

export default function PageThemeBuilderFeature() {
  const { realm_name, theme_id } = useParams<RouterParams & { theme_id: string }>()
  const navigate = useNavigate()
  const realm = realm_name ?? 'master'
  const themeId = theme_id ?? ''

  const { data: themeData, isLoading } = useGetPortalThemeById({ realm, themeId })
  const { data: layoutsData } = useGetPortalLayouts({ realm })
  const { mutate: updateMetadata, isPending: isSavingMetadata } = useUpdatePortalThemeMetadata()
  const { mutate: updatePage, isPending: isSavingPage } = useUpdatePortalThemePage()
  const { mutate: activateTheme, isPending: isActivating } = useActivatePortalTheme()

  if (isLoading || !themeData?.data) {
    return (
      <div className='flex h-[60vh] items-center justify-center text-muted-foreground'>
        <BasicSpinner />
      </div>
    )
  }

  const theme = themeData.data

  const handleBack = () => navigate(PORTAL_THEMES_URL(realm))

  const handleSaveMetadata = (name: string, layoutId: string | null, configJson: object) => {
    updateMetadata({
      path: { realm_name: realm, theme_id: themeId },
      body: {
        name,
        layout_id: layoutId ?? undefined,
        config: configJson,
      },
    })
  }

  const handleSavePage = (
    pageType:
      | 'login'
      | 'register'
      | 'totp'
      | 'forgot_password'
      | 'reset_password'
      | 'magic_link_verify'
      | 'verify_email',
    tree: unknown,
  ) => {
    updatePage({
      path: { realm_name: realm, theme_id: themeId, page_type: pageType },
      body: { tree },
    })
  }

  const handleActivate = () => {
    activateTheme({ path: { realm_name: realm, theme_id: themeId } })
  }

  return (
    <PortalThemeProvider initial={theme.config}>
      <PageThemeBuilder
        theme={theme}
        layouts={layoutsData?.data ?? []}
        isSavingMetadata={isSavingMetadata}
        isSavingPage={isSavingPage}
        isActivating={isActivating}
        realm={realm}
        onBack={handleBack}
        onSaveMetadata={handleSaveMetadata}
        onSavePage={handleSavePage}
        onActivate={handleActivate}
      />
    </PortalThemeProvider>
  )
}
