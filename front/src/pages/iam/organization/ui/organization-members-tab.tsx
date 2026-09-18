import { useMemo, useState } from 'react'
import { Search, Shield, Trash2, UserPlus } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { Button } from '@/components/kit/button'
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip'
import { EntityPicker, IconTile, Pill, Section, StatusDot } from '@/components/kit'
import { cn } from '@/lib/utils'
import { tokens } from '@/styles/style-tokens'
import { isServiceAccount } from '@/utils'
import { Schemas } from '@/api/api.client'
import { memberDisplayName } from '../member-name'

import User = Schemas.User

const SERVICE_ACCOUNT_INITIAL = 'S'
const FALLBACK_INITIAL = 'U'

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
  const { t } = useTranslation('organization')
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
        title={t('detail.members.add.title')}
        description={t('detail.members.add.description')}
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
            addLabel={t('detail.members.add.pick')}
            searchPlaceholder={t('detail.members.add.search_placeholder')}
            emptyHint={t('detail.members.add.empty_hint')}
            exhaustedHint={t('detail.members.add.exhausted_hint')}
          />
          {staged.length > 0 && (
            <Button
              size='sm'
              onClick={() => {
                onAdd(staged)
                setStaged([])
              }}
            >
              <UserPlus /> {t('detail.members.add.submit', { count: staged.length })}
            </Button>
          )}
        </div>
      </Section>

      <Section
        title={t('detail.members.title', { total: members.length })}
        description={t('detail.members.description')}
        contained={false}
        action={
          <label className='relative flex h-8 w-64 items-center'>
            <Search className='pointer-events-none absolute left-2.5 size-3.5 text-neutral-400 dark:text-neutral-500' />
            <input
              type='search'
              value={query}
              onChange={(e) => setQuery(e.target.value)}
              placeholder={t('detail.members.search_placeholder')}
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
              {t('detail.members.empty')}
            </p>
          ) : filtered.length === 0 ? (
            <p className='px-3 py-10 text-center text-sm text-neutral-500 dark:text-neutral-400'>
              {t('detail.members.no_match')}
            </p>
          ) : (
            filtered.map((user) => {
              const serviceAccount = isServiceAccount(user)
              return (
                <div key={user.id} className='flex items-center gap-3 px-3 py-2'>
                  <IconTile tone={serviceAccount ? 'violet' : 'info'}>
                    <span className='text-xs font-semibold uppercase'>
                      {(serviceAccount
                        ? SERVICE_ACCOUNT_INITIAL
                        : user.firstname || user.username || FALLBACK_INITIAL
                      ).charAt(0)}
                    </span>
                  </IconTile>
                  <div className='min-w-0 flex-1'>
                    <div className='flex items-center gap-2'>
                      <span className='truncate text-[13px] font-medium text-neutral-900 dark:text-neutral-100'>
                        {memberDisplayName(user)}
                      </span>
                      <Pill tone={serviceAccount ? 'violet' : 'info'} mono>
                        {serviceAccount
                          ? t('member.kind.service_account')
                          : t('member.kind.user_account')}
                      </Pill>
                    </div>
                    <p className='truncate text-xs text-neutral-500 dark:text-neutral-400'>
                      {user.email ?? user.username}
                    </p>
                  </div>
                  <span className='inline-flex shrink-0 items-center gap-1.5 text-xs text-neutral-600 dark:text-neutral-400'>
                    <StatusDot on={user.enabled} />
                    {user.enabled ? t('member.status.active') : t('member.status.inactive')}
                  </span>
                  <Tooltip>
                    <TooltipTrigger asChild>
                      <Button
                        variant='ghost'
                        size='icon'
                        className='size-7 text-neutral-400 dark:text-neutral-500'
                        aria-label={t('detail.members.manage_roles_for', {
                          name: memberDisplayName(user),
                        })}
                        onClick={() => onManageRoles(user)}
                      >
                        <Shield />
                      </Button>
                    </TooltipTrigger>
                    <TooltipContent>{t('detail.members.manage_roles')}</TooltipContent>
                  </Tooltip>
                  <Tooltip>
                    <TooltipTrigger asChild>
                      <Button
                        variant='ghost'
                        size='icon'
                        className='size-7 text-neutral-400 dark:text-neutral-500 hover:text-fk-danger'
                        aria-label={t('detail.members.remove_member', {
                          name: memberDisplayName(user),
                        })}
                        onClick={() => onRemove(user)}
                      >
                        <Trash2 />
                      </Button>
                    </TooltipTrigger>
                    <TooltipContent>{t('detail.members.remove')}</TooltipContent>
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
