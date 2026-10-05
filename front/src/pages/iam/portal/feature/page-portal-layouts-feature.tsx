import { useMemo } from 'react'
import { useNavigate, useParams } from 'react-router-dom'
import { useTranslation } from 'react-i18next'
import { toast } from 'sonner'
import {
  PORTAL_LAYOUT_FILTER_KEYS,
  useDeletePortalLayout,
  useGetPortalLayouts,
  useImportPortalLayout,
  usePortalLayoutCount,
  type PortalLayoutsQuery,
} from '@/api/portal-layouts.api'
import { usePagedListing } from '@/components/kit'
import { downloadPortalLayoutExport, readExportFile } from '@/api/builder-export'
import { RouterParams } from '@/routes/router'
import { NEW_LAYOUT_ID, usePortalUrls } from '../use-portal-urls'
import { countNodes } from '../theme-validation'
import PagePortalLayouts, { type PortalLayoutRow } from '../ui/page-portal-layouts'

export default function PagePortalLayoutsFeature() {
  const { t } = useTranslation('portal')
  const portal = usePortalUrls()
  const { realm_name } = useParams<RouterParams>()
  const navigate = useNavigate()
  const realm = realm_name ?? 'master'

  const listing = usePagedListing(PORTAL_LAYOUT_FILTER_KEYS)
  const { data: layoutsData, isLoading } = useGetPortalLayouts({
    realm,
    query: listing.apiQuery as PortalLayoutsQuery,
    keepPrevious: true,
  })
  const total = usePortalLayoutCount({ realm })
  const defaults = usePortalLayoutCount({ realm, filter: { is_default: true } })
  const used = usePortalLayoutCount({ realm, filter: { in_use: true } })
  const free = usePortalLayoutCount({ realm, filter: { in_use: false, is_default: false } })
  const { mutate: deleteLayout } = useDeletePortalLayout()
  const { mutate: importLayout } = useImportPortalLayout()

  const rows = useMemo<PortalLayoutRow[]>(
    () =>
      (layoutsData?.data ?? []).map((layout) => ({
        layout,
        nodes: countNodes(layout.tree),
      })),
    [layoutsData]
  )

  const handleImport = (file: File) => {
    readExportFile(file)
      .then((envelope) => {
        importLayout({ path: { realm_name: realm }, body: envelope as never })
      })
      .catch((error: Error) => toast.error(error.message))
  }

  return (
    <PagePortalLayouts
      realm={realm}
      rows={rows}
      listing={listing}
      pagination={layoutsData?.metadata}
      counts={{
        total: total.count,
        defaults: defaults.count,
        used: used.count,
        free: free.count,
      }}
      isLoading={isLoading}
      onCreate={() => navigate(portal.layout(NEW_LAYOUT_ID))}
      onEdit={(layoutId) => navigate(portal.layout(layoutId))}
      onDelete={(layoutId) =>
        deleteLayout({ path: { realm_name: realm, layout_id: layoutId } })
      }
      onExport={(layoutId) =>
        downloadPortalLayoutExport(realm, layoutId).catch(() =>
          toast.error(t('layouts.toast.export_failed'))
        )
      }
      onImport={handleImport}
    />
  )
}
