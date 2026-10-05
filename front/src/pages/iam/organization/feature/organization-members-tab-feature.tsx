import { useMemo, useState } from 'react'
import {
  ORGANIZATION_MEMBER_FILTER_KEYS,
  useAddUserToOrganization,
  useGetOrganizationMembers,
  type OrganizationMembersQuery,
} from '@/api/organization.api'
import { useUserSearch, useUsersByIds } from '@/api/user.api'
import { usePagedListing } from '@/components/kit'
import { Schemas } from '@/api/api.client'
import OrganizationMembersTab, { type OrganizationMemberRow } from '../ui/organization-members-tab'

import User = Schemas.User

export interface OrganizationMembersTabFeatureProps {
  realm: string
  organizationId: string
  onRemove: (user: User) => void
  onManageRoles: (user: User) => void
}

export default function OrganizationMembersTabFeature({
  realm,
  organizationId,
  onRemove,
  onManageRoles,
}: OrganizationMembersTabFeatureProps) {
  const listing = usePagedListing(ORGANIZATION_MEMBER_FILTER_KEYS)
  const { data, isLoading } = useGetOrganizationMembers({
    realm,
    organizationId,
    query: listing.apiQuery as OrganizationMembersQuery,
    keepPrevious: true,
  })

  const memberships = useMemo(() => data?.data ?? [], [data])
  const memberIds = useMemo(() => memberships.map((member) => member.user_id), [memberships])
  const { users, isLoading: isLoadingUsers } = useUsersByIds({ realm, ids: memberIds })
  const rows = useMemo<OrganizationMemberRow[]>(() => {
    const byId = new Map(users.map((user) => [user.id, user]))
    return memberships.flatMap((member) => {
      const user = byId.get(member.user_id)
      return user ? [{ id: member.id, joinedAt: member.created_at, user }] : []
    })
  }, [memberships, users])
  const [settledRows, setSettledRows] = useState<OrganizationMemberRow[] | null>(null)
  if (!isLoading && !isLoadingUsers && settledRows !== rows) setSettledRows(rows)

  const notInOrganization = useMemo(
    () => ({ not_in_organization: organizationId }),
    [organizationId]
  )
  const userSearch = useUserSearch({ realm, filter: notInOrganization })
  const { mutate: addMember } = useAddUserToOrganization()

  return (
    <OrganizationMembersTab
      rows={isLoadingUsers && settledRows ? settledRows : rows}
      listing={listing}
      pagination={data?.metadata}
      isLoading={isLoading || (isLoadingUsers && settledRows === null)}
      availableUsers={userSearch.users}
      onSearchUsers={userSearch.setSearch}
      isSearchingUsers={userSearch.isLoading}
      onAdd={(userIds) => {
        for (const userId of userIds) {
          addMember({
            path: { realm_name: realm, organization_id: organizationId },
            body: { user_id: userId },
          })
        }
      }}
      onRemove={onRemove}
      onManageRoles={onManageRoles}
    />
  )
}
