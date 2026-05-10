import { useGetUsers } from '@/api/user.api'
import { RouterParams } from '@/routes/router'
import { IDENTITY_CREATE_URL, IDENTITY_URL } from '@/routes/sub-router/user-management.router'
import { isServiceAccount } from '@/utils'
import { useMemo } from 'react'
import { useNavigate, useParams } from 'react-router'
import PageIdentities from '../ui/page-identities'

export default function PageIdentitiesFeature() {
  const { realm_name } = useParams<RouterParams>()
  const navigate = useNavigate()
  const { data, isLoading } = useGetUsers({ realm: realm_name ?? 'master' })

  // CIAM = customer identities only. Service accounts (M2M) belong in the
  // admin/clients view, not in the customer list.
  const identities = useMemo(
    () => (data?.data ?? []).filter((u) => !isServiceAccount(u)),
    [data],
  )

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
      identities={identities}
      isLoading={isLoading}
      onSelect={handleSelect}
      onCreate={handleCreate}
    />
  )
}
