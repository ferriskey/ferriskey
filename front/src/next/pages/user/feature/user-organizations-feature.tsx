import { useMemo, useState } from 'react'
import { useParams } from 'react-router'
import { toast } from 'sonner'
import {
  useAddUserToOrganization,
  useGetOrganizations,
  useGetUserOrganizations,
  useRemoveUserFromOrganization,
} from '@/api/organization.api'
import { RouterParams } from '@/routes/router'
import { assignOrganizationSchema } from '@/pages/user/schemas/assign-organization.schema'
import UserOrganizationsTab, { type UserMembership } from '../ui/user-organizations-tab'

export default function UserOrganizationsFeature() {
  const { realm_name, user_id } = useParams<RouterParams>()
  const realm = realm_name ?? 'master'

  const {
    data: userOrgs,
    isLoading: isLoadingMemberships,
    isError,
  } = useGetUserOrganizations({ realm, userId: user_id })
  const { data: allOrgsResponse, isLoading: isLoadingOrgs } = useGetOrganizations({ realm })
  const { mutate: addToOrganization } = useAddUserToOrganization()
  const { mutate: removeFromOrganization } = useRemoveUserFromOrganization()

  const [selectedOrganizationIds, setSelectedOrganizationIds] = useState<string[]>([])

  const memberships = useMemo<UserMembership[]>(() => {
    if (!userOrgs || !allOrgsResponse) return []
    const organizations = new Map(allOrgsResponse.data.map((org) => [org.id, org]))
    return userOrgs.flatMap((member) => {
      const organization = organizations.get(member.organization_id)
      return organization ? [{ organization, joinedAt: member.created_at }] : []
    })
  }, [userOrgs, allOrgsResponse])

  const availableOrganizations = useMemo(() => {
    const assigned = new Set(memberships.map((m) => m.organization.id))
    return (allOrgsResponse?.data ?? []).filter((org) => !assigned.has(org.id))
  }, [memberships, allOrgsResponse])

  const handleAssign = () => {
    if (!user_id || !realm_name) {
      toast.error('User or realm not found')
      return
    }
    if (
      !assignOrganizationSchema.safeParse({ organizationIds: selectedOrganizationIds }).success
    )
      return

    for (const organizationId of selectedOrganizationIds) {
      addToOrganization({
        path: { realm_name, organization_id: organizationId },
        body: { user_id },
      })
    }
    setSelectedOrganizationIds([])
    toast.success('Organization(s) assigned successfully')
  }

  const handleRemove = (organizationId: string) => {
    if (!user_id || !realm_name) return
    removeFromOrganization({
      path: { realm_name, organization_id: organizationId, user_id },
    })
  }

  return (
    <UserOrganizationsTab
      memberships={memberships}
      availableOrganizations={availableOrganizations}
      isLoading={isLoadingMemberships || isLoadingOrgs}
      isError={isError}
      selectedOrganizationIds={selectedOrganizationIds}
      onSelectedOrganizationIdsChange={setSelectedOrganizationIds}
      onAssign={handleAssign}
      onRemove={handleRemove}
    />
  )
}
