import { useNavigate, useParams } from 'react-router'
import { useGetUsers, useUserCount, type UsersQuery } from '@/api/user.api'
import { usePagedListing } from '@/components/kit'
import { RouterParams } from '@/routes/router'
import { USERS_URL } from '@/routes/router'
import { Schemas } from '@/api/api.client'
import PageUsersOverview from '../ui/page-users-overview'

import User = Schemas.User

const UNVERIFIED_PREVIEW = 5

const USER_FILTER_KEYS = [
  'username',
  'email',
  'firstname',
  'lastname',
  'enabled',
  'email_verified',
  'service_account',
  'role_id',
] as const

export default function PageUsersOverviewFeature() {
  const { realm_name } = useParams<RouterParams>()
  const navigate = useNavigate()
  const realm = realm_name ?? 'master'

  const listing = usePagedListing(USER_FILTER_KEYS)
  const { data: usersResponse, isLoading } = useGetUsers({
    realm,
    query: listing.apiQuery as UsersQuery,
  })

  const total = useUserCount({ realm })
  const enabled = useUserCount({ realm, filter: { enabled: true } })
  const disabled = useUserCount({ realm, filter: { enabled: false } })
  const verified = useUserCount({ realm, filter: { email_verified: true } })
  const { data: unverifiedResponse } = useGetUsers({
    realm,
    query: { email_verified: false, service_account: false, limit: UNVERIFIED_PREVIEW },
  })

  return (
    <PageUsersOverview
      users={usersResponse?.data ?? []}
      pagination={usersResponse?.metadata}
      listing={listing}
      isLoading={isLoading}
      counts={{
        total: total.count,
        enabled: enabled.count,
        disabled: disabled.count,
        verified: verified.count,
      }}
      unverified={{
        total: unverifiedResponse?.metadata.total ?? 0,
        names: (unverifiedResponse?.data ?? []).map((user) => user.username),
      }}
      userHref={(user: User) => `${USERS_URL(realm)}/${user.id}/overview`}
      onCreate={() => navigate(`${USERS_URL(realm)}/create`)}
    />
  )
}
