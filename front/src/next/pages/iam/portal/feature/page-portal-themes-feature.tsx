import { useMemo } from 'react'
import { useNavigate, useParams } from 'react-router-dom'
import { toast } from 'sonner'
import {
  useActivatePortalTheme,
  useCreatePortalTheme,
  useDeletePortalTheme,
  useGetActivePortalTheme,
  useGetPortalPageRequirements,
  useImportPortalTheme,
  useListPortalThemes,
  useUpdatePortalThemePage,
} from '@/api/portal-theme.api'
import { useGetPortalLayouts } from '@/api/portal-layouts.api'
import { downloadPortalThemeExport, readExportFile } from '@/api/builder-export'
import { DEFAULT_PAGE_TYPES, defaultPageTree } from '@/lib/builder-portal'
import { defaultTheme } from '@/pages/portal-theme/lib/theme'
import { RouterParams } from '@/routes/router'
import { usePortalUrls } from '../use-portal-urls'
import { failingPages, requirementsByPage, statusesForTheme } from '../theme-validation'
import PagePortalThemes, { type PortalThemeRow } from '../ui/page-portal-themes'

export default function PagePortalThemesFeature() {
  const portal = usePortalUrls()
  const { realm_name } = useParams<RouterParams>()
  const navigate = useNavigate()
  const realm = realm_name ?? 'master'

  const { data: listData, isLoading } = useListPortalThemes({ realm })
  const { data: activeData } = useGetActivePortalTheme({ realm, pageType: 'login' })
  const { data: layoutsData } = useGetPortalLayouts({ realm })
  const { data: requirementsData } = useGetPortalPageRequirements({ realm })

  const { mutate: createTheme, isPending: isCreating } = useCreatePortalTheme()
  const { mutateAsync: updatePage } = useUpdatePortalThemePage()
  const { mutate: importTheme } = useImportPortalTheme()
  const { mutate: deleteTheme } = useDeletePortalTheme()
  const { mutate: activateTheme } = useActivatePortalTheme()

  const activeThemeId = activeData?.theme_id ?? null

  const rows = useMemo<PortalThemeRow[]>(() => {
    const requirements = requirementsByPage(requirementsData?.data)
    const layouts = layoutsData?.data ?? []
    return (listData?.data ?? []).map((theme) => ({
      theme,
      isActive: activeThemeId === theme.id,
      layoutName: layouts.find((l) => l.id === theme.layout_id)?.name,
      failures: failingPages(statusesForTheme(theme, requirements)),
    }))
  }, [listData, layoutsData, requirementsData, activeThemeId])

  const seedDefaultPages = async (themeId: string) => {
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
        `${failed} page${failed > 1 ? 's' : ''} could not be pre-filled — open them before activating this theme.`
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
          navigate(`${portal.theme(newId)}/theme`)
        },
      }
    )
  }

  const handleImport = (file: File) => {
    readExportFile(file)
      .then((envelope) => {
        importTheme({ path: { realm_name: realm }, body: envelope as never })
      })
      .catch((error: Error) => toast.error(error.message))
  }

  return (
    <PagePortalThemes
      rows={rows}
      isLoading={isLoading}
      isCreating={isCreating}
      themeHref={(themeId) => `${portal.theme(themeId)}/theme`}
      onCreate={handleCreate}
      onActivate={(themeId) =>
        activateTheme({ path: { realm_name: realm, theme_id: themeId } })
      }
      onDelete={(themeId) => deleteTheme({ path: { realm_name: realm, theme_id: themeId } })}
      onExport={(themeId) =>
        downloadPortalThemeExport(realm, themeId).catch(() =>
          toast.error('Could not export this theme')
        )
      }
      onImport={handleImport}
    />
  )
}
