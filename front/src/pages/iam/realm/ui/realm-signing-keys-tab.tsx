import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { Check, Copy, KeyRound, RefreshCw } from 'lucide-react'
import {
  AlertDialog,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from '@/components/ui/alert-dialog'
import { Button, ConfirmDestructiveDialog, EmptyState, Pill, Section } from '@/components/kit'
import type { PillTone } from '@/components/kit'
import type { Schemas } from '@/api/api.client'
import { cn } from '@/lib/utils'
import { tokens } from '@/styles/style-tokens'
import { formatTimestamp } from '@/utils/format-date'
import { REALM_NAMESPACE } from '../realm-namespace'

type SigningKey = Schemas.SigningKey
type SigningKeyStatus = Schemas.SigningKeyStatus

const SHORT_KID_LENGTH = 8

const STATUS_ORDER: SigningKeyStatus[] = ['active', 'passive', 'retired', 'revoked']

const STATUS_TONES: Record<SigningKeyStatus, PillTone> = {
  active: 'success',
  passive: 'amber',
  retired: 'neutral',
  revoked: 'danger',
}

const REVOCABLE: SigningKeyStatus[] = ['active', 'passive']

const COPIED_RESET_MS = 2000

const shortKid = (kid: string) => kid.slice(0, SHORT_KID_LENGTH)

export interface RealmSigningKeysTabProps {
  keys: SigningKey[]
  isLoading: boolean
  isError: boolean
  isRotating: boolean
  isRevoking: boolean
  onRotate: () => void
  onRevoke: (kid: string) => void
}

function CopyKid({ kid, label }: { kid: string; label: string }) {
  const [copied, setCopied] = useState(false)

  const copy = async () => {
    await navigator.clipboard.writeText(kid)
    setCopied(true)
    setTimeout(() => setCopied(false), COPIED_RESET_MS)
  }

  return (
    <Button
      type='button'
      variant='ghost'
      size='icon-xs'
      aria-label={label}
      onClick={() => void copy()}
    >
      {copied ? <Check className='text-fk-success' /> : <Copy />}
    </Button>
  )
}

export default function RealmSigningKeysTab({
  keys,
  isLoading,
  isError,
  isRotating,
  isRevoking,
  onRotate,
  onRevoke,
}: RealmSigningKeysTabProps) {
  const { t } = useTranslation(REALM_NAMESPACE)
  const [rotateOpen, setRotateOpen] = useState(false)
  const [revokeTarget, setRevokeTarget] = useState<SigningKey | null>(null)

  const dash = t('keys.table.none')

  const confirmRotate = () => {
    setRotateOpen(false)
    onRotate()
  }

  const confirmRevoke = () => {
    if (!revokeTarget) return
    const { kid } = revokeTarget
    setRevokeTarget(null)
    onRevoke(kid)
  }

  const body = () => {
    if (isLoading) {
      return <div className='h-40 animate-pulse rounded bg-neutral-100 dark:bg-fk-raised' />
    }

    if (isError) {
      return (
        <div className={cn(tokens.surface.panel, 'grid place-items-center px-6 py-12')}>
          <p className='text-sm font-medium text-neutral-700 dark:text-neutral-300'>
            {t('keys.error.title')}
          </p>
          <p className='mt-1 max-w-sm text-center text-sm text-neutral-500 dark:text-neutral-400'>
            {t('keys.error.description')}
          </p>
        </div>
      )
    }

    if (keys.length === 0) {
      return <EmptyState icon={KeyRound} label={t('keys.empty.title')} hint={t('keys.empty.hint')} />
    }

    return (
      <div className={cn(tokens.surface.panel, 'overflow-x-auto')}>
        <table className={cn('w-full', tokens.table.text)}>
          <thead>
            <tr
              className={cn(
                'border-b border-fk-line bg-neutral-50/60 text-left dark:bg-fk-surface/60',
                tokens.table.headerText
              )}
            >
              <th className={cn(tokens.table.headerPadding, 'font-medium')}>
                {t('keys.table.kid')}
              </th>
              <th className={cn(tokens.table.headerPadding, 'font-medium')}>
                {t('keys.table.algorithm')}
              </th>
              <th className={cn(tokens.table.headerPadding, 'font-medium')}>
                {t('keys.table.status')}
              </th>
              <th className={cn(tokens.table.headerPadding, 'font-medium')}>
                {t('keys.table.created_at')}
              </th>
              <th className={cn(tokens.table.headerPadding, 'font-medium')}>
                {t('keys.table.retire_at')}
              </th>
              <th className={cn(tokens.table.headerPadding, 'font-medium')}>
                {t('keys.table.revoked_at')}
              </th>
              <th className={cn(tokens.table.headerPadding, 'text-right font-medium')}>
                <span className='sr-only'>{t('keys.table.actions')}</span>
              </th>
            </tr>
          </thead>
          <tbody className={tokens.surface.divider}>
            {keys.map((key) => (
              <tr key={key.kid}>
                <td className={tokens.table.cellPadding}>
                  <span className='inline-flex items-center gap-1'>
                    <span
                      title={key.kid}
                      className='font-mono-ui text-neutral-900 dark:text-neutral-100'
                    >
                      {shortKid(key.kid)}
                    </span>
                    <CopyKid kid={key.kid} label={t('keys.table.copy_kid', { kid: key.kid })} />
                  </span>
                </td>
                <td className={cn(tokens.table.cellPadding, 'font-mono-ui')}>{key.algorithm}</td>
                <td className={tokens.table.cellPadding}>
                  <Pill tone={STATUS_TONES[key.status]}>{t(`keys.status.${key.status}.label`)}</Pill>
                </td>
                <td className={cn(tokens.table.cellPadding, 'tnum whitespace-nowrap')}>
                  {formatTimestamp(key.created_at)}
                </td>
                <td className={cn(tokens.table.cellPadding, 'tnum whitespace-nowrap')}>
                  {key.retire_at ? formatTimestamp(key.retire_at) : dash}
                </td>
                <td className={cn(tokens.table.cellPadding, 'tnum whitespace-nowrap')}>
                  {key.revoked_at ? formatTimestamp(key.revoked_at) : dash}
                </td>
                <td className={cn(tokens.table.cellPadding, 'text-right')}>
                  {REVOCABLE.includes(key.status) && (
                    <Button
                      type='button'
                      variant='destructive'
                      size='xs'
                      disabled={isRevoking}
                      onClick={() => setRevokeTarget(key)}
                    >
                      {t('keys.revoke.button')}
                    </Button>
                  )}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    )
  }

  return (
    <>
      <Section
        title={t('keys.title')}
        description={t('keys.description')}
        contained={false}
        action={
          <Button
            type='button'
            size='sm'
            disabled={isRotating}
            onClick={() => setRotateOpen(true)}
          >
            <RefreshCw />
            {t('keys.rotate.button')}
          </Button>
        }
      >
        {body()}
      </Section>

      <ul className='space-y-1 text-xs text-neutral-500 dark:text-neutral-400'>
        {STATUS_ORDER.map((status) => (
          <li key={status} className='flex items-center gap-2'>
            <Pill tone={STATUS_TONES[status]} className='shrink-0'>
              {t(`keys.status.${status}.label`)}
            </Pill>
            <span>{t(`keys.status.${status}.description`)}</span>
          </li>
        ))}
      </ul>

      <AlertDialog open={rotateOpen} onOpenChange={setRotateOpen}>
        <AlertDialogContent className='sm:max-w-md'>
          <AlertDialogHeader>
            <AlertDialogTitle className='text-base'>{t('keys.rotate.confirm_title')}</AlertDialogTitle>
            <AlertDialogDescription>{t('keys.rotate.confirm_description')}</AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel className='outline-none cursor-pointer'>
              {t('keys.dialog.cancel')}
            </AlertDialogCancel>
            <Button size='sm' onClick={confirmRotate}>
              {t('keys.rotate.confirm_action')}
            </Button>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>

      <ConfirmDestructiveDialog
        open={revokeTarget !== null}
        onOpenChange={(open) => {
          if (!open) setRevokeTarget(null)
        }}
        resourceName={revokeTarget ? shortKid(revokeTarget.kid) : ''}
        title={t('keys.revoke.confirm_title')}
        description={
          revokeTarget?.status === 'active'
            ? t('keys.revoke.confirm_description_active')
            : t('keys.revoke.confirm_description_passive')
        }
        consequences={[t('keys.revoke.consequence_tokens'), t('keys.revoke.consequence_irreversible')]}
        confirmLabel={t('keys.revoke.confirm_action')}
        pending={isRevoking}
        onConfirm={confirmRevoke}
      />
    </>
  )
}
