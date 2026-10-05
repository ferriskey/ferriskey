import { useMemo, useState } from 'react'
import { useParams } from 'react-router'
import { useTranslation } from 'react-i18next'
import { toast } from 'sonner'
import {
  useAddUserToOrganization,
  useGetUserOrganizations,
  useOrganizationSearch,
  useOrganizationsByIds,
  useRemoveUserFromOrganization,
} from '@/api/organization.api'
import { Schemas } from '@/api/api.client'
import { RouterParams } from '@/routes/router'
import { assignOrganizationSchema } from '@/pages/iam/user/schemas/assign-organization.schema'
import UserOrganizationsTab, { type UserMembership } from '../ui/user-organizations-tab'

export default function UserOrganizationsFeature() {
  const { realm_name, user_id } = useParams<RouterParams>()
  const { t } = useTranslation('user')
  const realm = realm_name ?? 'master'

  const {
    data: userOrgs,
    isLoading: isLoadingMemberships,
    isError,
  } = useGetUserOrganizations({ realm, userId: user_id })
  const { mutateAsync: addToOrganization } = useAddUserToOrganization()
  const { mutate: removeFromOrganization } = useRemoveUserFromOrganization()

  const [selectedOrganizationIds, setSelectedOrganizationIds] = useState<string[]>([])

  const memberOrganizationIds = useMemo(
    () => (userOrgs ?? []).map((member) => member.organization_id),
    [userOrgs]
  )
  const joined = useOrganizationsByIds({ realm, ids: memberOrganizationIds })
  const selected = useOrganizationsByIds({ realm, ids: selectedOrganizationIds })
  const search = useOrganizationSearch({
    realm,
    filter: { without_member: user_id },
    enabled: !!user_id,
  })

  const memberships = useMemo<UserMembership[]>(() => {
    if (!userOrgs) return []
    const organizations = new Map(joined.organizations.map((org) => [org.id, org]))
    return userOrgs.flatMap((member) => {
      const organization = organizations.get(member.organization_id)
      return organization ? [{ organization, joinedAt: member.created_at }] : []
    })
  }, [userOrgs, joined.organizations])

  const availableOrganizations = useMemo(() => {
    const byId = new Map<string, Schemas.Organization>()
    for (const organization of [...selected.organizations, ...search.organizations]) {
      byId.set(organization.id, organization)
    }
    return [...byId.values()]
  }, [selected.organizations, search.organizations])

  const handleAssign = async () => {
    if (!user_id || !realm_name) {
      toast.error(t('detail.organizations.toast.missing_context'))
      return
    }
    if (
      !assignOrganizationSchema.safeParse({ organizationIds: selectedOrganizationIds }).success
    )
      return

    const attempted = selectedOrganizationIds
    const outcomes = await Promise.allSettled(
      attempted.map((organizationId) =>
        addToOrganization({
          path: { realm_name, organization_id: organizationId },
          body: { user_id },
        })
      )
    )

    const failed = attempted.filter((_, index) => outcomes[index].status === 'rejected')
    setSelectedOrganizationIds(failed)

    const assignedCount = attempted.length - failed.length
    if (assignedCount > 0) {
      toast.success(t('detail.organizations.toast.assigned', { count: assignedCount }))
    }
    if (failed.length > 0) {
      toast.error(t('detail.organizations.toast.assign_failed', { count: failed.length }))
    }
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
      onSearchOrganizations={search.setSearch}
      isSearchingOrganizations={search.isLoading}
      isLoading={isLoadingMemberships || joined.isLoading}
      isError={isError || joined.isError}
      selectedOrganizationIds={selectedOrganizationIds}
      onSelectedOrganizationIdsChange={setSelectedOrganizationIds}
      onAssign={handleAssign}
      onRemove={handleRemove}
    />
  )
}
