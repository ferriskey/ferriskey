import { useMemo } from 'react'
import { useNavigate, useParams } from 'react-router'
import { useGetUsers } from '@/api/user.api'
import { RouterParams } from '@/routes/router'
import { isServiceAccount } from '@/utils'
import { Schemas } from '@/api/api.client'
import { CONSOLE_IDENTITIES_URL, CONSOLE_IDENTITY_URL } from '../urls'
import PageIdentities from '../ui/page-identities'

import User = Schemas.User

export default function PageIdentitiesFeature() {
  const { realm_name } = useParams<RouterParams>()
  const navigate = useNavigate()
  const realm = realm_name ?? 'master'

  const { data: usersResponse, isLoading } = useGetUsers({ realm })

  const identities = useMemo(
    () => (usersResponse?.data ?? []).filter((user) => !isServiceAccount(user)),
    [usersResponse]
  )

  const firstPending = identities.find((user) => user.required_actions.length > 0)

  return (
    <PageIdentities
      identities={identities}
      isLoading={isLoading}
      identityHref={(identity: User) => `${CONSOLE_IDENTITY_URL(realm, identity.id)}/overview`}
      onCreate={() => navigate(`${CONSOLE_IDENTITIES_URL(realm)}/create`)}
      onReviewPending={() => {
        if (!firstPending) return
        navigate(`${CONSOLE_IDENTITY_URL(realm, firstPending.id)}/overview`)
      }}
    />
  )
}
