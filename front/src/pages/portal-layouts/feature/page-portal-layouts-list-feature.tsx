import { useNavigate, useParams } from 'react-router-dom'
import type { RouterParams } from '@/routes/router'
import {
  useDeletePortalLayout,
  useGetPortalLayouts,
  useSetDefaultPortalLayout,
} from '@/api/portal-layouts.api'
import { PORTAL_LAYOUT_BUILDER_URL } from '@/routes/sub-router/portal-layouts.router'
import PagePortalLayoutsList from '../ui/page-portal-layouts-list'

export default function PagePortalLayoutsListFeature() {
  const { realm_name } = useParams<RouterParams>()
  const navigate = useNavigate()
  const realm = realm_name ?? 'master'

  const { data, isLoading } = useGetPortalLayouts({ realm })
  const { mutate: deleteLayout } = useDeletePortalLayout()
  const { mutate: setDefaultLayout } = useSetDefaultPortalLayout()

  const handleEdit = (layoutId: string) => {
    navigate(PORTAL_LAYOUT_BUILDER_URL(realm_name, layoutId))
  }

  const handleDelete = (layoutId: string) => {
    deleteLayout({ path: { realm_name: realm, layout_id: layoutId } })
  }

  const handleSetDefault = (layoutId: string) => {
    setDefaultLayout({ path: { realm_name: realm, layout_id: layoutId } })
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
      onSetDefault={handleSetDefault}
      onCreate={handleCreate}
    />
  )
}
