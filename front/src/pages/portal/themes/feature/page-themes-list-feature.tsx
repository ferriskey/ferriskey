import { useLocation, useNavigate, useParams } from 'react-router-dom'
import type { RouterParams } from '@/routes/router'
import {
  useActivatePortalTheme,
  useCreatePortalTheme,
  useDeletePortalTheme,
  useGetActivePortalTheme,
  useListPortalThemes,
  useUpdatePortalThemePage,
} from '@/api/portal-theme.api'
import { DEFAULT_PAGE_TYPES, defaultPageTree } from '@/lib/builder-portal'
import { toast } from 'sonner'
import { themeBuilderUrl } from '@/routes/sub-router/portal-theme.router'
import PageThemesList from '../ui/page-themes-list'
import { defaultTheme } from '@/pages/portal-theme/lib/theme'

export default function PageThemesListFeature() {
  const { realm_name } = useParams<RouterParams>()
  const navigate = useNavigate()
  const { pathname } = useLocation()
  const realm = realm_name ?? 'master'

  const { data: listData, isLoading } = useListPortalThemes({ realm })
  const { data: activeData } = useGetActivePortalTheme({ realm, pageType: 'login' })
  const activeThemeId = activeData?.theme_id ?? null

  const { mutate: createTheme, isPending: isCreating } = useCreatePortalTheme()
  const { mutateAsync: updatePage } = useUpdatePortalThemePage()
  const { mutate: deleteTheme } = useDeletePortalTheme()
  const { mutate: activateTheme } = useActivatePortalTheme()

  /**
   * Fills a new theme's pages with their default composition.
   *
   * A theme created with empty pages cannot be activated — the server requires
   * each flow's blocks to be present — so seeding here is what makes a fresh
   * theme usable without composing twelve pages by hand first.
   */
  const seedDefaultPages = async (themeId: string) => {
    // Sequential rather than concurrent: should the access token expire
    // mid-seed, twelve parallel calls would each start their own refresh with
    // the same refresh token, and reuse detection revokes a rotated family.
    // One at a time, the first refresh covers the calls that follow. Twelve
    // small writes cost little enough that the wait is not worth the risk.
    let failed = 0
    for (const pageType of DEFAULT_PAGE_TYPES) {
      try {
        await updatePage({
          path: { realm_name: realm, theme_id: themeId, page_type: pageType },
          body: { tree: defaultPageTree(pageType) },
        })
      } catch {
        failed += 1
      }
    }

    if (failed > 0) {
      toast.warning(
        `${failed} page${failed > 1 ? 's' : ''} could not be pre-filled — open them before activating this theme.`,
      )
    }
  }

  const handleCreate = (name: string) => {
    createTheme(
      {
        path: { realm_name: realm },
        body: { name, config: defaultTheme },
      },
      {
        onSuccess: async (res) => {
          const newId = res?.data?.id
          if (!newId) return

          await seedDefaultPages(newId)
          navigate(themeBuilderUrl(pathname, realm, newId))
        },
      },
    )
  }

  const handleEdit = (themeId: string) => {
    navigate(themeBuilderUrl(pathname, realm, themeId))
  }

  const handleActivate = (themeId: string) => {
    activateTheme({ path: { realm_name: realm, theme_id: themeId } })
  }

  const handleDelete = (themeId: string) => {
    deleteTheme({ path: { realm_name: realm, theme_id: themeId } })
  }

  return (
    <PageThemesList
      themes={listData?.data ?? []}
      activeThemeId={activeThemeId}
      isLoading={isLoading}
      isCreating={isCreating}
      onCreate={handleCreate}
      onEdit={handleEdit}
      onActivate={handleActivate}
      onDelete={handleDelete}
    />
  )
}
