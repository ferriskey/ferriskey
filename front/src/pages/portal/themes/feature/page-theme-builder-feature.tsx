import { useNavigate, useParams } from 'react-router-dom'
import { toast } from 'sonner'
import { BasicSpinner } from '@/components/ui/spinner'
import {
  describePortalPageError,
  labelForPageType,
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
import type { Schemas } from '@/api/api.client'

type PageType = Schemas.PortalPageType

export default function PageThemeBuilderFeature() {
  const { realm_name, theme_id } = useParams<RouterParams & { theme_id: string }>()
  const navigate = useNavigate()
  const realm = realm_name ?? 'master'
  const themeId = theme_id ?? ''

  const { data: themeData, isLoading } = useGetPortalThemeById({ realm, themeId })
  const { data: layoutsData } = useGetPortalLayouts({ realm })
  const { mutateAsync: updateMetadataAsync, isPending: isSavingMetadata } =
    useUpdatePortalThemeMetadata()
  const { mutateAsync: updatePageAsync, isPending: isSavingPage } = useUpdatePortalThemePage()
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

  // The save button fires metadata + N pages in parallel. We only want one
  // toast at the end: success if every call passed, error otherwise — so a
  // 422 on a page tree can't be hidden behind a green metadata toast.
  const handleSaveTheme = async (
    name: string,
    layoutId: string | null,
    configJson: object,
    pages: { pageType: PageType; tree: unknown }[],
  ) => {
    const results = await Promise.allSettled([
      updateMetadataAsync({
        path: { realm_name: realm, theme_id: themeId },
        body: {
          name,
          layout_id: layoutId ?? undefined,
          config: configJson,
        },
      }),
      ...pages.map(({ pageType, tree }) =>
        updatePageAsync({
          path: { realm_name: realm, theme_id: themeId, page_type: pageType },
          body: { tree },
        }).catch((error: unknown) => {
          // Re-throw so Promise.allSettled records the rejection, but tag
          // the page_type onto the error so the toast can name it.
          throw { pageType, error }
        }),
      ),
    ])

    const failures = results.filter(
      (r): r is PromiseRejectedResult => r.status === 'rejected',
    )

    if (failures.length === 0) {
      toast.success('Portal theme saved')
      return
    }

    const descriptions = failures.map((f) => {
      const reason = f.reason as { pageType?: PageType; error?: unknown } | unknown
      if (reason && typeof reason === 'object' && 'pageType' in reason) {
        const { pageType, error } = reason as { pageType: PageType; error: unknown }
        const detail = describePortalPageError(error) ?? 'Unknown error'
        return `${labelForPageType(pageType)} page — ${detail}`
      }
      const detail = describePortalPageError(reason) ?? 'Unknown error'
      return `Theme metadata — ${detail}`
    })

    toast.error('Failed to save portal theme', { description: descriptions.join('\n') })
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
        onSaveTheme={handleSaveTheme}
        onActivate={handleActivate}
      />
    </PortalThemeProvider>
  )
}
