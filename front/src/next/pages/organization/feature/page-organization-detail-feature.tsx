import { useMemo, useState } from 'react'
import { useNavigate, useParams } from 'react-router'
import {
  useAddUserToOrganization,
  useDeleteOrganization,
  useDeleteOrganizationAttribute,
  useGetOrganization,
  useGetOrganizationAttributes,
  useGetOrganizationMembers,
  useRemoveOrganizationMember,
  useUpdateOrganization,
  useUpsertOrganizationAttribute,
} from '@/api/organization.api'
import { useGetUsers } from '@/api/user.api'
import { RouterParams } from '@/routes/router'
import { useRouteTabs } from '@/components/kit'
import { ConfirmDeleteAlert } from '@/components/confirm-delete-alert'
import { useConfirmDeleteAlert } from '@/hooks/use-confirm-delete-alert'
import { updateOrganizationSchema } from '@/pages/organization/schemas/update-organization.schema'
import PageOrganizationGroupsFeature from '@/pages/organization/feature/page-organization-groups-feature'
import ManageMemberRolesModalFeature from '@/pages/organization/feature/modals/manage-member-roles-modal-feature'
import { Schemas } from '@/api/api.client'
import { NEXT_ORGANIZATIONS_URL } from '@/next/routes'
import { NEXT_ORGANIZATION_URL } from '../urls'
import PageOrganizationDetail from '../ui/page-organization-detail'
import { type OrganizationDraft } from '../ui/organization-settings-tab'
import { memberDisplayName } from '../member-name'

import User = Schemas.User
import { useCrumbLabel } from '@/next/shell/crumb-store'

interface Draft extends OrganizationDraft {
  key: string
}

const EMPTY_DRAFT: Draft = {
  key: '',
  name: '',
  alias: '',
  enabled: true,
  domain: '',
  redirectUrl: '',
  description: '',
}

const ORGANIZATION_TABS = [
  { key: 'settings', label: 'Settings' },
  { key: 'attributes', label: 'Attributes' },
  { key: 'members', label: 'Members' },
  { key: 'groups', label: 'Groups' },
] as const

export default function PageOrganizationDetailFeature() {
  const { realm_name, organizationId } = useParams<RouterParams & { organizationId: string }>()
  const navigate = useNavigate()
  const realm = realm_name ?? 'master'

  const { data: organization, isLoading } = useGetOrganization({ realm, organizationId })
  const { mutate: updateOrganization } = useUpdateOrganization()
  const { mutateAsync: deleteOrganization } = useDeleteOrganization()

  const { data: attributes, isLoading: isLoadingAttributes } = useGetOrganizationAttributes({
    realm,
    organizationId,
  })
  const { mutate: upsertAttribute } = useUpsertOrganizationAttribute()
  const { mutate: deleteAttribute } = useDeleteOrganizationAttribute()

  const { data: members, isLoading: isLoadingMembers } = useGetOrganizationMembers({
    realm,
    organizationId,
  })
  const { data: usersResponse, isLoading: isLoadingUsers } = useGetUsers({ realm })
  const { mutate: addMember } = useAddUserToOrganization()
  const { mutate: removeMember } = useRemoveOrganizationMember()
  const { confirm, ask, close } = useConfirmDeleteAlert()

  const { value: tab, tabs } = useRouteTabs(
    NEXT_ORGANIZATION_URL(realm, organizationId),
    ORGANIZATION_TABS
  )

  const [rolesUser, setRolesUser] = useState<User | null>(null)
  const [draft, setDraft] = useState<Draft>(EMPTY_DRAFT)

  const organizationKey = organization?.id ?? ''
  const pristine: Draft = organization
    ? {
        key: organizationKey,
        name: organization.name,
        alias: organization.alias,
        enabled: organization.enabled,
        domain: organization.domain ?? '',
        redirectUrl: organization.redirect_url ?? '',
        description: organization.description ?? '',
      }
    : EMPTY_DRAFT

  if (organization && draft.key !== organizationKey) setDraft(pristine)

  const current = draft.key === organizationKey ? draft : pristine

  const parsed = updateOrganizationSchema.safeParse({
    name: current.name,
    alias: current.alias,
    domain: current.domain,
    redirectUrl: current.redirectUrl,
    description: current.description,
    enabled: current.enabled,
  })

  const isDirty = Boolean(
    organization &&
      (current.name !== pristine.name ||
        current.alias !== pristine.alias ||
        current.enabled !== pristine.enabled ||
        current.domain !== pristine.domain ||
        current.redirectUrl !== pristine.redirectUrl ||
        current.description !== pristine.description)
  )

  const errors =
    parsed.success || !isDirty
      ? {}
      : {
          name: parsed.error.issues.find((i) => i.path[0] === 'name')?.message,
          alias: parsed.error.issues.find((i) => i.path[0] === 'alias')?.message,
        }

  const memberUsers = useMemo<User[]>(() => {
    if (!members || !usersResponse) return []
    const userMap = new Map(usersResponse.data.map((u) => [u.id, u]))
    return members
      .map((m) => userMap.get(m.user_id))
      .filter((u): u is User => u !== undefined)
  }, [members, usersResponse])

  const availableUsers = useMemo<User[]>(() => {
    if (!members || !usersResponse) return []
    const memberIds = new Set(members.map((m) => m.user_id))
    return usersResponse.data.filter((u) => !memberIds.has(u.id))
  }, [members, usersResponse])

  const handleSave = () => {
    if (!organization || !parsed.success) return
    updateOrganization({
      body: {
        name: current.name,
        alias: current.alias,
        domain: current.domain || null,
        redirect_url: current.redirectUrl || null,
        description: current.description || null,
        enabled: current.enabled,
      },
      path: { realm_name: realm, organization_id: organization.id },
    })
  }

  const handleDelete = async () => {
    if (!organization) return
    try {
      await deleteOrganization({
        path: { realm_name: realm, organization_id: organization.id },
      })
      navigate(NEXT_ORGANIZATIONS_URL(realm))
    } catch {
      setDraft(pristine)
    }
  }

  const handleRemoveMember = (user: User) => {
    if (!organizationId) return
    ask({
      title: 'Remove member?',
      description: `Remove "${memberDisplayName(user)}" from this organization? Their organization-scoped roles will be revoked. This does not delete the user.`,
      onConfirm: () => {
        removeMember({
          path: { realm_name: realm, organization_id: organizationId, user_id: user.id },
        })
        close()
      },
    })
  }


  useCrumbLabel(organizationId, organization?.name)

  return (
    <>
      <PageOrganizationDetail
        organization={organization}
        isLoading={isLoading}
        tab={tab}
        tabs={tabs}
        draft={current}
        errors={errors}
        isDirty={isDirty}
        onDraftChange={(patch) => setDraft({ ...current, ...patch, key: organizationKey })}
        onDiscard={() => setDraft(pristine)}
        onSave={handleSave}
        onDelete={handleDelete}
        attributes={attributes ?? []}
        isLoadingAttributes={isLoadingAttributes}
        onUpsertAttribute={(key, value) => {
          if (!organizationId) return
          upsertAttribute({
            body: { value },
            path: { realm_name: realm, organization_id: organizationId, key },
          })
        }}
        onDeleteAttribute={(key) => {
          if (!organizationId) return
          deleteAttribute({
            path: { realm_name: realm, organization_id: organizationId, key },
          })
        }}
        members={memberUsers}
        availableUsers={availableUsers}
        isLoadingMembers={isLoadingMembers || isLoadingUsers}
        onAddMembers={(userIds) => {
          if (!organizationId) return
          for (const userId of userIds) {
            addMember({
              path: { realm_name: realm, organization_id: organizationId },
              body: { user_id: userId },
            })
          }
        }}
        onRemoveMember={handleRemoveMember}
        onManageRoles={setRolesUser}
        groups={<PageOrganizationGroupsFeature />}
        onBack={() => navigate(NEXT_ORGANIZATIONS_URL(realm))}
      />

      <ManageMemberRolesModalFeature
        realm={realm}
        orgId={organizationId}
        user={rolesUser}
        open={rolesUser !== null}
        onOpenChange={(open) => {
          if (!open) setRolesUser(null)
        }}
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
