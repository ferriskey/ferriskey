import { useGetUsers } from '@/api/user.api'
import { RouterParams } from '@/routes/router'
import { IDENTITY_CREATE_URL, IDENTITY_URL } from '@/routes/sub-router/user-management.router'
import { useNavigate, useParams } from 'react-router'
import PageIdentities from '../ui/page-identities'

export default function PageIdentitiesFeature() {
  const { realm_name } = useParams<RouterParams>()
  const navigate = useNavigate()
  const { data, isLoading } = useGetUsers({ realm: realm_name ?? 'master' })

  const handleSelect = (userId: string) => {
    if (!realm_name) return
    navigate(IDENTITY_URL(realm_name, userId))
  }

  const handleCreate = () => {
    if (!realm_name) return
    navigate(IDENTITY_CREATE_URL(realm_name))
  }

  return (
    <PageIdentities
      identities={data?.data ?? []}
      isLoading={isLoading}
      onSelect={handleSelect}
      onCreate={handleCreate}
    />
  )
}
