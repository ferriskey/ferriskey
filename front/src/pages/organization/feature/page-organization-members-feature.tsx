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
import { ConfirmDeleteAlert } from '@/components/confirm-delete-alert'
import { useConfirmDeleteAlert } from '@/hooks/use-confirm-delete-alert'
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
  const { confirm, ask, close } = useConfirmDeleteAlert()

  const memberUsers = useMemo<User[]>(() => {
    if (!members || !usersResponse) return []
    const userMap = new Map(usersResponse.data.map((u) => [u.id, u]))
    return members
      .map((m) => userMap.get(m.user_id))
      .filter((u): u is User => u !== undefined)
  }, [members, usersResponse])

  const handleRemove = (user: User) => {
    if (!realm_name || !organizationId) return
    const displayName =
      [user.firstname, user.lastname].filter(Boolean).join(' ') || user.username
    ask({
      title: 'Remove member?',
      description: `Remove "${displayName}" from this organization? Their organization-scoped roles will be revoked. This does not delete the user.`,
      onConfirm: () => {
        removeMember({
          path: { realm_name, organization_id: organizationId, user_id: user.id },
        })
        close()
      },
    })
  }

  return (
    <>
      <PageOrganizationMembers
        realm={realm_name}
        organizationId={organizationId}
        members={memberUsers}
        isLoading={isLoadingMembers || isLoadingUsers}
        isError={isError}
        handleRemove={handleRemove}
      />
      <ConfirmDeleteAlert
        title={confirm.title}
        description={confirm.description}
        open={confirm.open}
        onConfirm={confirm.onConfirm}
        onCancel={close}
      />
    </>
  )
}
