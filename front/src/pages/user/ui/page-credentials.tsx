import { useMemo } from 'react'
import { OverviewList } from '@/components/ui/overview-list'
import { EntityAvatar } from '@/components/ui/entity-avatar'
import { ConfirmDeleteAlert } from '@/components/confirm-delete-alert'
import StatisticsCard from '../components/statistics-card'
import SetPasswordFeature from '../feature/modals/set-password-feature'
import { Trash2 } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { useConfirmDeleteAlert } from '@/hooks/use-confirm-delete-alert.ts'
import { Schemas } from '@/api/api.client'

import CredentialOverview = Schemas.CredentialOverview

export interface PageCredentialsProps {
  credentials: CredentialOverview[]
  handleDeleteUserCredential: (credentialId: string) => void
}

const CREDENTIAL_COLORS: Record<string, string> = {
  password: '#10B981',
  otp: '#3B82F6',
  recovery_code: '#8B5CF6',
}

function credentialColor(type: string): string {
  return CREDENTIAL_COLORS[type] ?? '#6B7280'
}

function CredentialTypeBadge({ type }: { type: string }) {
  const styles: Record<string, string> = {
    password:
      'border-emerald-400/50 text-emerald-600 bg-emerald-50 dark:bg-emerald-500/10',
    otp: 'border-blue-300 text-blue-500 bg-blue-50 dark:bg-blue-500/10 dark:border-blue-400/40',
    recovery_code:
      'border-purple-300 text-purple-500 bg-purple-50 dark:bg-purple-500/10 dark:border-purple-400/40',
  }
  const cls = styles[type] ?? 'border-border text-muted-foreground bg-muted/50'
  return (
    <span
      className={`inline-flex items-center px-2.5 py-0.5 rounded-md border text-xs font-mono ${cls}`}
    >
      {type}
    </span>
  )
}

export default function PageCredentials({
  credentials,
  handleDeleteUserCredential,
}: PageCredentialsProps) {
  const { confirm, ask, close } = useConfirmDeleteAlert()

  const statistics = useMemo(() => {
    const total = credentials.length
    const passwords = credentials.filter((c) => c.credential_type === 'password').length
    const totps = credentials.filter((c) => c.credential_type === 'otp').length
    const others = total - passwords - totps
    return { total, passwords, totps, others }
  }, [credentials])

  const onRowDelete = (credential: CredentialOverview) => {
    ask({
      title: 'Delete credential?',
      description: `Are you sure you want to delete this "${credential.credential_type}" credential?`,
      onConfirm: () => {
        handleDeleteUserCredential(credential.id)
        close()
      },
    })
  }

  return (
    <div className='flex flex-col gap-6'>
      {/* Statistics Cards */}
      <div>
        <p className='text-xs text-muted-foreground mb-3'>Credential overview</p>
        <div className='grid gap-4 md:grid-cols-2 lg:grid-cols-4'>
          <StatisticsCard
            title='Total credentials'
            value={statistics.total}
            description='All registered credentials'
          />
          <StatisticsCard
            title='Passwords'
            value={statistics.passwords}
            description={
              statistics.passwords > 0 && statistics.total > 0 ? (
                <span className='text-emerald-600 font-medium'>
                  {((statistics.passwords / statistics.total) * 100).toFixed(0)}% of total
                </span>
              ) : (
                'No password credentials'
              )
            }
          />
          <StatisticsCard
            title='TOTP'
            value={statistics.totps}
            description='Authenticator app credentials'
          />
          <StatisticsCard
            title='Other'
            value={statistics.others}
            description='Recovery codes & more'
          />
        </div>
      </div>

      {/* Actions */}
      <div className='flex justify-end'>
        <SetPasswordFeature contentText='Set Password' />
      </div>

      {/* Credential List */}
      <OverviewList
        data={credentials}
        searchKeys={['credential_type', 'user_label']}
        searchPlaceholder='Search credentials...'
        title={(n) => `Credentials (${n})`}
        emptyLabel='No credentials found.'
        renderRow={(credential) => (
          <div className='flex items-center justify-between px-8 py-4 border-b last:border-b-0 hover:bg-muted/40 transition-colors'>
            <div className='flex items-center gap-4'>
              <EntityAvatar
                label={credential.credential_type}
                color={credentialColor(credential.credential_type)}
              />
              <div>
                <div className='flex items-center gap-2.5'>
                  <span className='text-base font-medium capitalize'>
                    {credential.credential_type}
                  </span>
                  <CredentialTypeBadge type={credential.credential_type} />
                </div>
                <div className='text-sm text-muted-foreground mt-0.5'>
                  {credential.user_label
                    ? credential.user_label
                    : new Date(credential.created_at).toLocaleDateString('en-US', {
                        year: 'numeric',
                        month: 'short',
                        day: '2-digit',
                        hour: '2-digit',
                        minute: '2-digit',
                      })}
                </div>
              </div>
            </div>
            <div className='flex items-center gap-2'>
              {credential.credential_type === 'password' && (
                <SetPasswordFeature contentText='Reset password' />
              )}
              <Button
                variant='ghost'
                size='sm'
                className='text-destructive hover:text-destructive hover:bg-destructive/10'
                onClick={() => onRowDelete(credential)}
              >
                <Trash2 className='h-4 w-4' />
              </Button>
            </div>
          </div>
        )}
      />

      <ConfirmDeleteAlert
        title={confirm.title}
        description={confirm.description}
        open={confirm.open}
        onConfirm={confirm.onConfirm}
        onCancel={close}
      />
    </div>
  )
}
