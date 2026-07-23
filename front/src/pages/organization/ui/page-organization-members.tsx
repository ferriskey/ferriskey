import { Fragment, useState } from 'react'
import { AlertTriangle, Shield, Trash2 } from 'lucide-react'

import { Schemas } from '@/api/api.client'
import { OverviewList } from '@/components/ui/overview-list'
import { EntityAvatar } from '@/components/ui/entity-avatar'
import { Button } from '@/components/ui/button'
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip'
import { isServiceAccount } from '@/utils'
import ManageMemberRolesModalFeature from '../feature/modals/manage-member-roles-modal-feature'

import User = Schemas.User

interface PageOrganizationMembersProps {
  realm?: string
  organizationId?: string
  members: User[]
  isLoading: boolean
  isError: boolean
  handleRemove: (user: User) => void
}

function MemberStatusBadge({ enabled }: { enabled: boolean }) {
  if (!enabled) {
    return (
      <span className='inline-flex items-center gap-1.5 px-3 py-1 rounded-md text-xs font-semibold border border-orange-400/50 text-orange-500 bg-orange-50 dark:bg-orange-500/10'>
        <AlertTriangle className='h-3 w-3' />
        INACTIVE
      </span>
    )
  }
  return (
    <span className='inline-flex items-center px-3 py-1 rounded-md text-xs font-semibold border border-emerald-400/50 text-emerald-600 bg-emerald-50 dark:bg-emerald-500/10'>
      ACTIVE
    </span>
  )
}

function MemberTypeBadge({ isSA }: { isSA: boolean }) {
  return isSA ? (
    <span className='inline-flex items-center px-2.5 py-0.5 rounded-md border border-purple-300 text-purple-500 text-xs font-mono bg-purple-50 dark:bg-purple-500/10 dark:border-purple-400/40'>
      service account
    </span>
  ) : (
    <span className='inline-flex items-center px-2.5 py-0.5 rounded-md border border-blue-300 text-blue-500 text-xs font-mono bg-blue-50 dark:bg-blue-500/10 dark:border-blue-400/40'>
      user account
    </span>
  )
}

export default function PageOrganizationMembers({
  realm,
  organizationId,
  members,
  isLoading,
  handleRemove,
}: PageOrganizationMembersProps) {
  const [rolesUser, setRolesUser] = useState<User | null>(null)

  return (
    <Fragment>
      <div className='flex flex-col gap-6'>
        {/* Members list */}
        <OverviewList
          data={members}
          isLoading={isLoading}
          searchKeys={['username', 'email', 'firstname', 'lastname']}
          searchPlaceholder='Search members...'
          title={(n) => `Members (${n})`}
          emptyLabel='This organization has no members yet.'
          renderRow={(user) => {
            const isSA = isServiceAccount(user)
            const displayName = isSA
              ? 'Service Account'
              : [user.firstname, user.lastname].filter(Boolean).join(' ') || user.username
            return (
              <div className='flex items-center justify-between px-8 py-4'>
                <div className='flex items-center gap-4'>
                  <EntityAvatar
                    label={isSA ? 'S' : user.firstname || user.username || 'U'}
                    color={isSA ? '#8B5CF6' : '#F97316'}
                  />
                  <div>
                    <div className='flex items-center gap-2.5'>
                      <span className='text-base font-medium'>{displayName}</span>
                      <MemberTypeBadge isSA={isSA} />
                    </div>
                    <div className='text-sm text-muted-foreground mt-0.5'>
                      {user.email || user.username}
                    </div>
                  </div>
                </div>

                <div className='flex items-center gap-2'>
                  <MemberStatusBadge enabled={user.enabled ?? true} />
                  <Tooltip>
                    <TooltipTrigger asChild>
                      <Button
                        variant='ghost'
                        size='icon'
                        aria-label='Manage roles'
                        onClick={() => setRolesUser(user)}
                      >
                        <Shield className='h-4 w-4' />
                      </Button>
                    </TooltipTrigger>
                    <TooltipContent>Manage roles</TooltipContent>
                  </Tooltip>
                  <Tooltip>
                    <TooltipTrigger asChild>
                      <Button
                        variant='ghost'
                        size='icon'
                        aria-label='Remove member'
                        className='text-destructive hover:text-destructive'
                        onClick={() => handleRemove(user)}
                      >
                        <Trash2 className='h-4 w-4' />
                      </Button>
                    </TooltipTrigger>
                    <TooltipContent>Remove from organization</TooltipContent>
                  </Tooltip>
                </div>
              </div>
            )
          }}
        />
      </div>

      <ManageMemberRolesModalFeature
        realm={realm}
        orgId={organizationId}
        user={rolesUser}
        open={rolesUser !== null}
        onOpenChange={(open) => {
          if (!open) setRolesUser(null)
        }}
      />
    </Fragment>
  )
}
