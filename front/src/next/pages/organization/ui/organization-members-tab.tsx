import { useMemo, useState } from 'react'
import { Search, Shield, Trash2, UserPlus } from 'lucide-react'
import { Button } from '@/components/kit/button'
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip'
import { EntityPicker, IconTile, Pill, Section, StatusDot } from '@/components/kit'
import { cn } from '@/lib/utils'
import { tokens } from '@/styles/style-tokens'
import { isServiceAccount } from '@/utils'
import { Schemas } from '@/api/api.client'
import { memberDisplayName } from '../member-name'

import User = Schemas.User

export interface OrganizationMembersTabProps {
  members: User[]
  availableUsers: User[]
  isLoading: boolean
  onAdd: (userIds: string[]) => void
  onRemove: (user: User) => void
  onManageRoles: (user: User) => void
}

export default function OrganizationMembersTab({
  members,
  availableUsers,
  isLoading,
  onAdd,
  onRemove,
  onManageRoles,
}: OrganizationMembersTabProps) {
  const [staged, setStaged] = useState<string[]>([])
  const [query, setQuery] = useState('')

  const filtered = useMemo(() => {
    const q = query.trim().toLowerCase()
    if (!q) return members
    return members.filter((user) =>
      `${user.username} ${user.email ?? ''} ${user.firstname ?? ''} ${user.lastname ?? ''}`
        .toLowerCase()
        .includes(q)
    )
  }, [members, query])

  return (
    <>
      <Section
        title='Add members'
        description='Only accounts of this realm that are not members yet can be picked.'
        contained={false}
      >
        <div className='space-y-2'>
          <EntityPicker
            items={availableUsers.map((user) => ({
              id: user.id,
              label: memberDisplayName(user),
              sublabel: user.email ?? user.username,
            }))}
            value={staged}
            onChange={setStaged}
            addLabel='Pick an account'
            searchPlaceholder='Search an account…'
            emptyHint='No account selected yet.'
            exhaustedHint='Every account of this realm is already a member.'
          />
          {staged.length > 0 && (
            <Button
              size='sm'
              onClick={() => {
                onAdd(staged)
                setStaged([])
              }}
            >
              <UserPlus /> Add {staged.length} member{staged.length > 1 ? 's' : ''}
            </Button>
          )}
        </div>
      </Section>

      <Section
        title={`Members (${members.length})`}
        description='Accounts attached to this organization, with their organization-scoped roles.'
        contained={false}
        action={
          <label className='relative flex h-8 w-64 items-center'>
            <Search className='pointer-events-none absolute left-2.5 size-3.5 text-neutral-400 dark:text-neutral-500' />
            <input
              type='search'
              value={query}
              onChange={(e) => setQuery(e.target.value)}
              placeholder='Search members…'
              className='h-full w-full rounded-md border border-fk-line bg-white dark:bg-fk-surface pl-8 pr-3 text-sm outline-none placeholder:text-neutral-400 focus:border-fk-primary-border focus:ring-2 focus:ring-fk-primary/15'
            />
          </label>
        }
      >
        <div className={cn(tokens.surface.panel, tokens.surface.divider)}>
          {isLoading ? (
            Array.from({ length: 4 }).map((_, i) => (
              <div key={i} className='flex items-center gap-3 px-3 py-3'>
                <div className='size-9 animate-pulse rounded-md bg-neutral-100 dark:bg-fk-raised' />
                <div className='h-3 w-48 animate-pulse rounded bg-neutral-100 dark:bg-fk-raised' />
              </div>
            ))
          ) : members.length === 0 ? (
            <p className='px-3 py-10 text-center text-sm text-neutral-500 dark:text-neutral-400'>
              This organization has no member yet.
            </p>
          ) : filtered.length === 0 ? (
            <p className='px-3 py-10 text-center text-sm text-neutral-500 dark:text-neutral-400'>
              No member matches this search.
            </p>
          ) : (
            filtered.map((user) => {
              const serviceAccount = isServiceAccount(user)
              return (
                <div key={user.id} className='flex items-center gap-3 px-3 py-2'>
                  <IconTile tone={serviceAccount ? 'violet' : 'info'}>
                    <span className='text-xs font-semibold uppercase'>
                      {(serviceAccount ? 'S' : user.firstname || user.username || 'U').charAt(0)}
                    </span>
                  </IconTile>
                  <div className='min-w-0 flex-1'>
                    <div className='flex items-center gap-2'>
                      <span className='truncate text-[13px] font-medium text-neutral-900 dark:text-neutral-100'>
                        {memberDisplayName(user)}
                      </span>
                      <Pill tone={serviceAccount ? 'violet' : 'info'} mono>
                        {serviceAccount ? 'service account' : 'user account'}
                      </Pill>
                    </div>
                    <p className='truncate text-xs text-neutral-500 dark:text-neutral-400'>
                      {user.email ?? user.username}
                    </p>
                  </div>
                  <span className='inline-flex shrink-0 items-center gap-1.5 text-xs text-neutral-600 dark:text-neutral-400'>
                    <StatusDot on={user.enabled} />
                    {user.enabled ? 'active' : 'inactive'}
                  </span>
                  <Tooltip>
                    <TooltipTrigger asChild>
                      <Button
                        variant='ghost'
                        size='icon'
                        className='size-7 text-neutral-400 dark:text-neutral-500'
                        aria-label={`Manage roles of ${memberDisplayName(user)}`}
                        onClick={() => onManageRoles(user)}
                      >
                        <Shield />
                      </Button>
                    </TooltipTrigger>
                    <TooltipContent>Manage roles</TooltipContent>
                  </Tooltip>
                  <Tooltip>
                    <TooltipTrigger asChild>
                      <Button
                        variant='ghost'
                        size='icon'
                        className='size-7 text-neutral-400 dark:text-neutral-500 hover:text-fk-danger'
                        aria-label={`Remove ${memberDisplayName(user)}`}
                        onClick={() => onRemove(user)}
                      >
                        <Trash2 />
                      </Button>
                    </TooltipTrigger>
                    <TooltipContent>Remove from organization</TooltipContent>
                  </Tooltip>
                </div>
              )
            })
          )}
        </div>
      </Section>
    </>
  )
}
