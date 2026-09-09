import { useMemo } from 'react'
import { useNavigate, useParams } from 'react-router-dom'
import { toast } from 'sonner'
import {
  useDeletePortalLayout,
  useGetPortalLayouts,
  useImportPortalLayout,
} from '@/api/portal-layouts.api'
import { useListPortalThemes } from '@/api/portal-theme.api'
import { downloadPortalLayoutExport, readExportFile } from '@/api/builder-export'
import { RouterParams } from '@/routes/router'
import { NEXT_PORTAL_LAYOUT_URL } from '../portal-urls'
import { countNodes } from '../theme-validation'
import PagePortalLayouts, { type PortalLayoutRow } from '../ui/page-portal-layouts'

export default function PagePortalLayoutsFeature() {
  const { realm_name } = useParams<RouterParams>()
  const navigate = useNavigate()
  const realm = realm_name ?? 'master'

  const { data: layoutsData, isLoading } = useGetPortalLayouts({ realm })
  const { data: themesData } = useListPortalThemes({ realm })
  const { mutate: deleteLayout } = useDeletePortalLayout()
  const { mutate: importLayout } = useImportPortalLayout()

  const rows = useMemo<PortalLayoutRow[]>(() => {
    const themes = themesData?.data ?? []
    return (layoutsData?.data ?? []).map((layout) => ({
      layout,
      nodes: countNodes(layout.tree),
      usedBy: themes.filter((t) => t.layout_id === layout.id).map((t) => t.name),
    }))
  }, [layoutsData, themesData])

  const handleImport = (file: File) => {
    readExportFile(file)
      .then((envelope) => {
        importLayout({ path: { realm_name: realm }, body: envelope as never })
      })
      .catch((error: Error) => toast.error(error.message))
  }

  return (
    <PagePortalLayouts
      rows={rows}
      isLoading={isLoading}
      onCreate={() => navigate(NEXT_PORTAL_LAYOUT_URL(realm, 'new'))}
      onEdit={(layoutId) => navigate(NEXT_PORTAL_LAYOUT_URL(realm, layoutId))}
      onDelete={(layoutId) =>
        deleteLayout({ path: { realm_name: realm, layout_id: layoutId } })
      }
      onExport={(layoutId) =>
        downloadPortalLayoutExport(realm, layoutId).catch(() =>
          toast.error('Could not export this layout')
        )
      }
      onImport={handleImport}
    />
  )
}
