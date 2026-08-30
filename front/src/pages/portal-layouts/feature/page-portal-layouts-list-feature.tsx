import { useNavigate, useParams } from 'react-router-dom'
import type { RouterParams } from '@/routes/router'
import {
  useDeletePortalLayout,
  useGetPortalLayouts,
  useImportPortalLayout,
} from '@/api/portal-layouts.api'
import { downloadPortalLayoutExport, readExportFile } from '@/api/builder-export'
import { toast } from 'sonner'
import { PORTAL_LAYOUT_BUILDER_URL } from '@/routes/sub-router/portal-layouts.router'
import PagePortalLayoutsList from '../ui/page-portal-layouts-list'

export default function PagePortalLayoutsListFeature() {
  const { realm_name } = useParams<RouterParams>()
  const navigate = useNavigate()
  const realm = realm_name ?? 'master'

  const { data, isLoading } = useGetPortalLayouts({ realm })
  const { mutate: deleteLayout } = useDeletePortalLayout()
  const { mutate: importLayout } = useImportPortalLayout()

  const handleEdit = (layoutId: string) => {
    navigate(PORTAL_LAYOUT_BUILDER_URL(realm_name, layoutId))
  }

  const handleDelete = (layoutId: string) => {
    deleteLayout({ path: { realm_name: realm, layout_id: layoutId } })
  }

  const handleExport = (layoutId: string) => {
    downloadPortalLayoutExport(realm, layoutId).catch(() =>
      toast.error('Could not export this layout'),
    )
  }

  /**
   * The exported file is sent back as-is: the server reads its envelope, and
   * refuses a file belonging to the other builder or written in a format it
   * cannot read.
   */
  const handleImport = (file: File) => {
    readExportFile(file)
      .then((envelope) => {
        importLayout({
          path: { realm_name: realm },
          body: envelope as never,
        })
      })
      .catch((error: Error) => toast.error(error.message))
  }

  const handleCreate = () => {
    navigate(PORTAL_LAYOUT_BUILDER_URL(realm_name, 'new'))
  }

  return (
    <PagePortalLayoutsList
      layouts={data?.data ?? []}
      isLoading={isLoading}
      onEdit={handleEdit}
      onDelete={handleDelete}
      onExport={handleExport}
      onImport={handleImport}
      onCreate={handleCreate}
    />
  )
}
