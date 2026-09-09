import { useMemo } from 'react'
import { useNavigate, useParams } from 'react-router'
import { useGetUsers } from '@/api/user.api'
import { RouterParams } from '@/routes/router'
import { NEXT_USERS_URL } from '@/next/routes'
import { Schemas } from '@/api/api.client'
import PageUsersOverview from '../ui/page-users-overview'

import User = Schemas.User

export default function PageUsersOverviewFeature() {
  const { realm_name } = useParams<RouterParams>()
  const navigate = useNavigate()
  const realm = realm_name ?? 'master'

  const { data: usersResponse, isLoading } = useGetUsers({ realm })
  const users = useMemo(() => usersResponse?.data ?? [], [usersResponse])

  return (
    <PageUsersOverview
      users={users}
      isLoading={isLoading}
      userHref={(user: User) => `${NEXT_USERS_URL(realm)}/${user.id}/overview`}
      onCreate={() => navigate(`${NEXT_USERS_URL(realm)}/create`)}
    />
  )
}
