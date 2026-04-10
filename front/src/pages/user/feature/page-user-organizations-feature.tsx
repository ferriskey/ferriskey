import { useParams } from 'react-router'
import { useGetOrganizations, useGetUserOrganizations, useRemoveUserFromOrganization } from '@/api/organization.api'
import { UserRouterParams } from '@/routes/sub-router/user.router'
import PageUserOrganizations from '../ui/page-user-organizations'
import { useMemo } from 'react'
import { Schemas } from '@/api/api.client'
import Organization = Schemas.Organization

export default function PageUserOrganizationsFeature() {
  const { realm_name, user_id } = useParams<UserRouterParams>()

  const {
    data: userOrgs,
    isLoading: isLoadingMemberships,
    isError,
  } = useGetUserOrganizations({
    realm: realm_name,
    userId: user_id,
  })

  const { data: allOrgsResponse, isLoading: isLoadingOrgs } = useGetOrganizations({
    realm: realm_name,
  })

  const { mutate: removeFromOrg } = useRemoveUserFromOrganization()

  const assignedOrganizations = useMemo<Organization[]>(() => {
    if (!userOrgs || !allOrgsResponse) return []
    const orgMap = new Map(allOrgsResponse.data.map((org) => [org.id, org]))
    return userOrgs
      .map((member) => orgMap.get(member.organization_id))
      .filter((org): org is Organization => org !== undefined)
  }, [userOrgs, allOrgsResponse])

  const handleRemove = (organizationId: string) => {
    if (!realm_name || !user_id) return
    removeFromOrg({
      path: { realm_name, organization_id: organizationId, user_id },
    })
  }

  return (
    <PageUserOrganizations
      organizations={assignedOrganizations}
      isLoading={isLoadingMemberships || isLoadingOrgs}
      isError={isError}
      handleRemove={handleRemove}
    />
  )
}
