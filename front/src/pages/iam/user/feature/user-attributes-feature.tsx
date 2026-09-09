import { useParams } from 'react-router'
import {
  useDeleteUserAttribute,
  useGetUserAttributes,
  useSetUserAttributes,
} from '@/api/user.api'
import { RouterParams } from '@/routes/router'
import UserAttributesTab from '../ui/user-attributes-tab'

export default function UserAttributesFeature() {
  const { realm_name, user_id } = useParams<RouterParams>()
  const realm = realm_name ?? 'master'

  const { data: attributes, isLoading } = useGetUserAttributes({ realm, userId: user_id })
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
    deleteAttribute({ path: { realm_name, user_id, key } })
  }

  return (
    <UserAttributesTab
      attributes={attributes ?? []}
      isLoading={isLoading}
      onUpsert={handleUpsert}
      onDelete={handleDelete}
    />
  )
}
