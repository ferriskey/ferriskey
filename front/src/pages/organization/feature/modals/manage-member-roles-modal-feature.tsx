import { useMemo } from 'react'
import { toast } from 'sonner'

import { Schemas } from '@/api/api.client'
import {
  Dialog,
  DialogBody,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import MultipleSelector, { Option } from '@/components/ui/multiselect'
import { useGetRoles } from '@/api/role.api'
import {
  useAssignOrganizationMemberRole,
  useOrganizationMemberRoles,
  useRevokeOrganizationMemberRole,
} from '@/api/organization-member.api'

import User = Schemas.User

interface ManageMemberRolesModalFeatureProps {
  realm?: string
  orgId?: string
  user: User | null
  open: boolean
  onOpenChange: (open: boolean) => void
}

export default function ManageMemberRolesModalFeature({
  realm,
  orgId,
  user,
  open,
  onOpenChange,
}: ManageMemberRolesModalFeatureProps) {
  const userId = user?.id
  const { data: assigned } = useOrganizationMemberRoles(realm, orgId, userId)
  const rolesResp = useGetRoles({ realm })
  const rolesData = (rolesResp.data as { data?: Array<{ id: string; name: string }> } | undefined)
    ?.data
  const assignRole = useAssignOrganizationMemberRole(realm, orgId, userId)
  const revokeRole = useRevokeOrganizationMemberRole(realm, orgId, userId)

  const fail = (e: unknown) =>
    toast.error(e instanceof Error ? e.message : 'Failed to update member roles')

  const options: Option[] = useMemo(
    () => (rolesData ?? []).map((r) => ({ value: r.id, label: r.name })),
    [rolesData]
  )
  const value: Option[] = useMemo(
    () => (assigned ?? []).map((r) => ({ value: r.id, label: r.name })),
    [assigned]
  )

  const onChange = (next: Option[]) => {
    const before = new Set(value.map((o) => o.value))
    const after = new Set(next.map((o) => o.value))
    next
      .filter((o) => !before.has(o.value))
      .forEach((o) => assignRole.mutate(o.value, { onError: fail }))
    value
      .filter((o) => !after.has(o.value))
      .forEach((o) => revokeRole.mutate(o.value, { onError: fail }))
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Manage roles{user ? ` — ${user.username}` : ''}</DialogTitle>
          <DialogDescription>
            Assign realm or client roles scoped to this member within the organization. These
            roles are surfaced in the member's token under the organization context.
          </DialogDescription>
        </DialogHeader>
        <DialogBody>
          <MultipleSelector
            value={value}
            options={options}
            onChange={onChange}
            placeholder='Search and assign roles…'
            hidePlaceholderWhenSelected
            emptyIndicator={
              <p className='text-center text-sm text-muted-foreground'>No roles found.</p>
            }
          />
        </DialogBody>
      </DialogContent>
    </Dialog>
  )
}
