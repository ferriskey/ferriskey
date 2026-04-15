import {
  useDeleteUserAttribute,
  useGetUserAttributes,
  useSetUserAttributes,
} from '@/api/user.api'
import { useParams } from 'react-router'
import { UserRouterParams } from '@/routes/sub-router/user.router'
import PageUserAttributes from '../ui/page-user-attributes'

export default function PageUserAttributesFeature() {
  const { realm_name, user_id } = useParams<UserRouterParams>()

  const { data: attributes, isLoading } = useGetUserAttributes({
    realm: realm_name,
    userId: user_id,
  })

  const { mutate: setAttributes } = useSetUserAttributes()
  const { mutate: deleteAttribute } = useDeleteUserAttribute()

  const handleUpsert = (key: string, value: string) => {
    if (!realm_name || !user_id) return
    setAttributes({
      body: { attributes: { [key]: value } },
      path: { realm_name, user_id },
    })
  }

  const handleDelete = (key: string) => {
    if (!realm_name || !user_id) return
    deleteAttribute({
      path: { realm_name, user_id, key },
    })
  }

  return (
    <PageUserAttributes
      attributes={attributes ?? []}
      isLoading={isLoading}
      onUpsert={handleUpsert}
      onDelete={handleDelete}
    />
  )
}
