import { useGetUsers } from '@/api/user.api'
import { RouterParams } from '@/routes/router'
import { USER_OVERVIEW_URL, USER_URL } from '@/routes/sub-router/user.router'
import { useNavigate, useParams } from 'react-router'
import PageIdentities from '../ui/page-identities'

export default function PageIdentitiesFeature() {
  const { realm_name } = useParams<RouterParams>()
  const navigate = useNavigate()
  const { data, isLoading } = useGetUsers({ realm: realm_name ?? 'master' })

  const handleSelect = (userId: string) => {
    if (!realm_name) return
    navigate(`${USER_URL(realm_name, userId)}${USER_OVERVIEW_URL}`)
  }

  const handleCreate = () => {
    if (!realm_name) return
    navigate(`/realms/${realm_name}/users/create`)
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
