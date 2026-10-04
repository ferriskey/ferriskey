import { useNavigate, useParams } from 'react-router'
import { useGetUsers, useUserCount, type UsersFilter, type UsersQuery } from '@/api/user.api'
import { usePagedListing } from '@/components/kit'
import { RouterParams } from '@/routes/router'
import { Schemas } from '@/api/api.client'
import { CONSOLE_IDENTITIES_URL, CONSOLE_IDENTITY_URL } from '../urls'
import PageIdentities from '../ui/page-identities'

import User = Schemas.User

const IDENTITIES: UsersFilter = { service_account: false }

const IDENTITY_FILTER_KEYS = ['username', 'email', 'enabled', 'email_verified'] as const

export default function PageIdentitiesFeature() {
  const { realm_name } = useParams<RouterParams>()
  const navigate = useNavigate()
  const realm = realm_name ?? 'master'

  const listing = usePagedListing(IDENTITY_FILTER_KEYS)
  const { data: usersResponse, isLoading } = useGetUsers({
    realm,
    query: { ...(listing.apiQuery as UsersQuery), ...IDENTITIES },
  })

  const total = useUserCount({ realm, filter: IDENTITIES })
  const verified = useUserCount({ realm, filter: { ...IDENTITIES, email_verified: true } })
  const disabled = useUserCount({ realm, filter: { ...IDENTITIES, enabled: false } })
  const active = useUserCount({ realm, filter: { ...IDENTITIES, enabled: true } })

  const identities = usersResponse?.data ?? []
  const firstPending = identities.find((user) => user.required_actions.length > 0)

  return (
    <PageIdentities
      identities={identities}
      pagination={usersResponse?.metadata}
      listing={listing}
      isLoading={isLoading}
      counts={{
        total: total.count,
        verified: verified.count,
        disabled: disabled.count,
        active: active.count,
      }}
      identityHref={(identity: User) => `${CONSOLE_IDENTITY_URL(realm, identity.id)}/overview`}
      onCreate={() => navigate(`${CONSOLE_IDENTITIES_URL(realm)}/create`)}
      onReviewPending={() => {
        if (!firstPending) return
        navigate(`${CONSOLE_IDENTITY_URL(realm, firstPending.id)}/overview`)
      }}
    />
  )
}
