import { useMemo, useState } from 'react'
import { KeyRound, Search, ShieldQuestion, Smartphone, Trash2 } from 'lucide-react'
import { Button } from '@/components/kit/button'
import { ConfirmDeleteAlert } from '@/components/confirm-delete-alert'
import { useConfirmDeleteAlert } from '@/hooks/use-confirm-delete-alert'
import { IconTile, MetricsBand, Pill, Section } from '@/components/kit'
import type { PillTone } from '@/components/kit'
import UserPasswordForm, { type UserPasswordFormProps } from './user-password-form'
import { cn } from '@/lib/utils'
import { tokens } from '@/styles/style-tokens'
import { Schemas } from '@/api/api.client'

import CredentialOverview = Schemas.CredentialOverview
import { formatDateTime } from '@/shared/format-date'

const credentialMeta: Record<
  string,
  { label: string; icon: typeof KeyRound; tone: PillTone }
> = {
  password: { label: 'Password', icon: KeyRound, tone: 'success' },
  otp: { label: 'One-time password', icon: Smartphone, tone: 'info' },
  recovery_code: { label: 'Recovery codes', icon: ShieldQuestion, tone: 'violet' },
}

const metaFor = (type: string) =>
  credentialMeta[type] ?? { label: type, icon: ShieldQuestion, tone: 'neutral' as PillTone }

const formatCreatedAt = (iso: string) =>
  formatDateTime(iso)

export interface UserCredentialsTabProps {
  credentials: CredentialOverview[]
  isLoading: boolean
  onDelete: (credentialId: string) => void
  passwordForm: Omit<UserPasswordFormProps, 'hasPassword'>
}

export default function UserCredentialsTab({
  credentials,
  isLoading,
  onDelete,
  passwordForm,
}: UserCredentialsTabProps) {
  const { confirm, ask, close } = useConfirmDeleteAlert()
  const [query, setQuery] = useState('')

  const rows = useMemo(() => {
    const q = query.trim().toLowerCase()
    if (!q) return credentials
    return credentials.filter((c) =>
      `${c.credential_type} ${c.user_label ?? ''}`.toLowerCase().includes(q)
    )
  }, [credentials, query])

  const total = credentials.length
  const passwords = credentials.filter((c) => c.credential_type === 'password').length
  const totps = credentials.filter((c) => c.credential_type === 'otp').length
  const hasPassword = passwords > 0

  const askDelete = (credential: CredentialOverview) =>
    ask({
      title: 'Delete credential?',
      description: `Are you sure you want to delete this "${credential.credential_type}" credential?`,
      onConfirm: () => {
        onDelete(credential.id)
        close()
      },
    })

  return (
    <>
      <MetricsBand
        metrics={[
          { key: 'total', label: 'Total credentials', value: total, hint: 'registered' },
          {
            key: 'passwords',
            label: 'Passwords',
            value: passwords,
            hint:
              passwords > 0 && total > 0
                ? `${((passwords / total) * 100).toFixed(0)}% of total`
                : 'No password credentials',
          },
          { key: 'otp', label: 'TOTP', value: totps, hint: 'authenticator app' },
          {
            key: 'other',
            label: 'Other',
            value: total - passwords - totps,
            hint: 'recovery codes & more',
          },
        ]}
      />

      <Section
        title='Authentication methods'
        description='Credentials registered for this account.'
        action={
          credentials.length > 0 ? (
            <label className='relative flex h-7 w-48 items-center'>
              <Search className='pointer-events-none absolute left-2 size-3.5 text-neutral-400 dark:text-neutral-500' />
              <input
                type='search'
                value={query}
                onChange={(e) => setQuery(e.target.value)}
                placeholder='Filter credentials…'
                className='h-full w-full rounded-md border border-fk-line pl-7 pr-2 text-xs outline-none placeholder:text-neutral-400 focus:border-fk-primary-border'
              />
            </label>
          ) : undefined
        }
        contained={!isLoading && rows.length > 0}
      >
        {isLoading ? (
          <div className={cn(tokens.surface.panel, 'divide-y divide-fk-line-soft')}>
            {Array.from({ length: 2 }).map((_, i) => (
              <div key={i} className='flex items-center gap-3 px-3 py-3'>
                <div className='size-7 animate-pulse rounded-md bg-neutral-100 dark:bg-fk-raised' />
                <div className='h-4 w-40 animate-pulse rounded bg-neutral-100 dark:bg-fk-raised' />
              </div>
            ))}
          </div>
        ) : rows.length > 0 ? (
          <ul className={tokens.surface.divider}>
            {rows.map((credential) => {
              const meta = metaFor(credential.credential_type)
              return (
                <li key={credential.id} className='flex items-center gap-2.5 py-2.5'>
                  <IconTile tone={meta.tone === 'neutral' ? 'info' : meta.tone} className='size-7'>
                    <meta.icon className='size-3.5' strokeWidth={1.75} />
                  </IconTile>

                  <div className='min-w-0'>
                    <p className='truncate text-sm font-medium text-neutral-900 dark:text-neutral-100'>
                      {credential.user_label || meta.label}
                    </p>
                    <p className='truncate text-xs text-neutral-500 dark:text-neutral-400'>
                      Added {formatCreatedAt(credential.created_at)}
                    </p>
                  </div>

                  <Pill tone={meta.tone} mono>
                    {credential.credential_type}
                  </Pill>

                  <span className='flex-1' />

                  <Button
                    variant='ghost'
                    size='icon'
                    aria-label={`Delete the ${credential.credential_type} credential`}
                    onClick={() => askDelete(credential)}
                    className='size-7 shrink-0 text-neutral-400 dark:text-neutral-500 hover:text-fk-danger'
                  >
                    <Trash2 />
                  </Button>
                </li>
              )
            })}
          </ul>
        ) : (
          <p className='rounded-md border border-dashed border-fk-amber-border bg-fk-amber-soft/40 px-4 py-3 text-sm text-neutral-600 dark:text-neutral-400'>
            {query
              ? `No credential matches “${query}”.`
              : 'No credential registered — this account cannot authenticate.'}
          </p>
        )}
      </Section>

      <Section
        title={hasPassword ? 'Reset the password' : 'Set a password'}
        description={
          hasPassword
            ? 'The current password is replaced immediately; open sessions stay valid.'
            : 'This account has no password: it can only authenticate through its other methods.'
        }
      >
        <UserPasswordForm hasPassword={hasPassword} {...passwordForm} />
      </Section>

      <p className='rounded-md border border-fk-amber-border bg-fk-amber-soft/40 px-4 py-3 text-sm text-neutral-600 dark:text-neutral-400'>
        Deleting a credential is immediate and irreversible: the user loses that authentication
        method without being notified.
      </p>

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
