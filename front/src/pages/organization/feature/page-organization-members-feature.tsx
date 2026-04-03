import { useParams } from 'react-router'
import { RouterParams } from '@/routes/router'
import {
  useGetOrganizationMembers,
  useRemoveOrganizationMember,
} from '@/api/organization.api'
import { useGetUsers } from '@/api/user.api'
import PageOrganizationMembers from '../ui/page-organization-members'
import { useMemo } from 'react'
import { Schemas } from '@/api/api.client'
import User = Schemas.User

export default function PageOrganizationMembersFeature() {
  const { realm_name, organizationId } = useParams<RouterParams & { organizationId: string }>()

  const {
    data: members,
    isLoading: isLoadingMembers,
    isError,
  } = useGetOrganizationMembers({ realm: realm_name, organizationId })

  const { data: usersResponse, isLoading: isLoadingUsers } = useGetUsers({ realm: realm_name })

  const { mutate: removeMember } = useRemoveOrganizationMember()

  const memberUsers = useMemo<User[]>(() => {
    if (!members || !usersResponse) return []
    const userMap = new Map(usersResponse.data.map((u) => [u.id, u]))
    return members
      .map((m) => userMap.get(m.user_id))
      .filter((u): u is User => u !== undefined)
  }, [members, usersResponse])

  const handleRemove = (userId: string) => {
    if (!realm_name || !organizationId) return
    removeMember({
      path: { realm_name, organization_id: organizationId, user_id: userId },
    })
  }

  return (
    <PageOrganizationMembers
      members={memberUsers}
      isLoading={isLoadingMembers || isLoadingUsers}
      isError={isError}
      handleRemove={handleRemove}
    />
  )
}
