import { useEffect, useState } from 'react'
import { Check, Copy, Eye, EyeOff, Loader2, RefreshCw, ShieldAlert } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { toast } from 'sonner'
import { Button } from '@/components/kit/button'
import { FieldRow, Section } from '@/components/kit'
import {
  AlertDialog,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
  AlertDialogTrigger,
} from '@/components/ui/alert-dialog'
import {
  Dialog,
  DialogBody,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from '@/components/ui/dialog'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { cn } from '@/lib/utils'
import { formatDateTime } from '@/utils/format-date'
import type { Schemas } from '@/api/api.client'
import ClientAuthenticationSection, {
  type ClientAuthenticationSectionProps,
} from './client-authentication-section'

type Client = Schemas.Client

export interface ClientCredentialsTabProps {
  client: Client
  revealed: boolean
  secret: string | null
  isFetching: boolean
  forbidden: boolean
  failed: boolean
  previousSecret: string | null
  rotating: boolean
  revoking: boolean
  onToggleReveal: () => void
  showSecret: boolean
  authentication?: ClientAuthenticationSectionProps
  onRotate: (gracePeriodSecs: number) => void
  onRevokePrevious: () => void
}

const GRACE_PERIODS = [
  { secs: 0, key: 'none' },
  { secs: 3_600, key: 'one_hour' },
  { secs: 86_400, key: 'one_day' },
  { secs: 604_800, key: 'one_week' },
  { secs: 2_592_000, key: 'thirty_days' },
] as const

const DEFAULT_GRACE_SECS = 86_400

const MASKED_SECRET = '••••••••••••••••••••••••'

function CopyButton({ value, disabled }: { value: string; disabled?: boolean }) {
  const { t } = useTranslation('client')
  const [copied, setCopied] = useState(false)

  const handleCopy = async () => {
    try {
      await navigator.clipboard.writeText(value)
    } catch {
      toast.error(t('credentials.copy_failed'))
      return
    }
    setCopied(true)
    setTimeout(() => setCopied(false), 2000)
  }

  return (
    <Button
      type='button'
      variant='outline'
      size='icon'
      disabled={disabled}
      aria-label={t('credentials.copy')}
      onClick={() => void handleCopy()}
    >
      {copied ? <Check className='size-4 text-fk-success' /> : <Copy className='size-4' />}
    </Button>
  )
}

function useStillRunning(expiresAt: string | undefined) {
  const deadline = expiresAt ? Date.parse(expiresAt) : undefined
  const [now, setNow] = useState(() => Date.now())

  useEffect(() => {
    if (deadline === undefined) return
    const refresh = () => setNow(Date.now())
    const timer = setTimeout(refresh, Math.max(0, deadline - Date.now()) + 500)
    document.addEventListener('visibilitychange', refresh)
    return () => {
      clearTimeout(timer)
      document.removeEventListener('visibilitychange', refresh)
    }
  }, [deadline, now])

  return deadline !== undefined && deadline > now
}

function Fingerprint({ value, muted }: { value: string; muted?: boolean }) {
  return (
    <code
      className={cn(
        'flex h-9 min-w-0 flex-1 items-center overflow-x-auto rounded-md border border-fk-line bg-neutral-50 dark:bg-fk-surface px-2.5 font-mono-ui text-xs',
        muted ? 'text-neutral-400 dark:text-neutral-500' : 'text-neutral-700 dark:text-neutral-300'
      )}
    >
      {value}
    </code>
  )
}

function RotateDialogContent({
  graceSecs,
  rotating,
  replacesPrevious,
  onGraceChange,
  onCancel,
  onConfirm,
}: {
  graceSecs: number
  rotating: boolean
  replacesPrevious: boolean
  onGraceChange: (secs: number) => void
  onCancel: () => void
  onConfirm: () => void
}) {
  const { t } = useTranslation('client')

  return (
    <DialogContent className='sm:max-w-md'>
      <DialogHeader>
        <DialogTitle>{t('credentials.rotate.title')}</DialogTitle>
        <DialogDescription>{t('credentials.rotate.description')}</DialogDescription>
      </DialogHeader>
      <DialogBody className='space-y-1.5'>
        <label
          htmlFor='rotate-grace'
          className='block text-sm text-neutral-700 dark:text-neutral-300'
        >
          {t('credentials.rotate.grace.label')}
        </label>
        <Select value={String(graceSecs)} onValueChange={(value) => onGraceChange(Number(value))}>
          <SelectTrigger id='rotate-grace' className='w-full'>
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            {GRACE_PERIODS.map((period) => (
              <SelectItem key={period.key} value={String(period.secs)}>
                {t(`credentials.rotate.grace.${period.key}`)}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
        {replacesPrevious && (
          <p className='flex items-start gap-1.5 pt-1 text-xs text-fk-amber'>
            <ShieldAlert className='mt-0.5 size-3.5 shrink-0' />
            {t('credentials.rotate.replaces_previous')}
          </p>
        )}
      </DialogBody>
      <DialogFooter>
        <Button variant='ghost' size='sm' onClick={onCancel}>
          {t('credentials.rotate.cancel')}
        </Button>
        <Button size='sm' disabled={rotating} onClick={onConfirm}>
          {t('credentials.rotate.confirm')}
        </Button>
      </DialogFooter>
    </DialogContent>
  )
}

export default function ClientCredentialsTab({
  client,
  revealed,
  secret,
  isFetching,
  forbidden,
  failed,
  previousSecret,
  rotating,
  revoking,
  onToggleReveal,
  showSecret,
  authentication,
  onRotate,
  onRevokePrevious,
}: ClientCredentialsTabProps) {
  const { t } = useTranslation('client')
  const [rotateOpen, setRotateOpen] = useState(false)
  const [revokeOpen, setRevokeOpen] = useState(false)
  const [graceSecs, setGraceSecs] = useState(DEFAULT_GRACE_SECS)
  const running = useStillRunning(client.previous_secret?.expires_at)
  const previous = running ? client.previous_secret : null
  const confidential = !client.public_client

  return (
    <>
      <Section title={t('credentials.title')} description={t('credentials.description')}>
        <FieldRow
          label={t('credentials.client_id.label')}
          description={t('credentials.client_id.description')}
        >
          <div className='flex max-w-lg items-center gap-2'>
            <Fingerprint value={client.client_id} />
            <CopyButton value={client.client_id} />
          </div>
        </FieldRow>

        {showSecret && (
          <>
            <FieldRow
              label={t('credentials.secret.label')}
              description={t('credentials.secret.description')}
            >
              <div className='max-w-lg space-y-2'>
                <div className='flex items-center gap-2'>
                  <Fingerprint value={secret ?? MASKED_SECRET} muted={!secret} />
                  <Button
                    type='button'
                    variant='outline'
                    size='sm'
                    disabled={isFetching}
                    aria-label={
                      revealed
                        ? t('credentials.secret.hide_label')
                        : t('credentials.secret.reveal_label')
                    }
                    onClick={onToggleReveal}
                  >
                    {isFetching ? (
                      <Loader2 className='size-4 animate-spin' />
                    ) : revealed ? (
                      <EyeOff className='size-4' />
                    ) : (
                      <Eye className='size-4' />
                    )}
                    {revealed ? t('credentials.secret.hide') : t('credentials.secret.reveal')}
                  </Button>
                  <CopyButton value={secret ?? ''} disabled={!secret} />
                </div>

                {forbidden && (
                  <p className='flex items-start gap-1.5 text-xs text-fk-amber'>
                    <ShieldAlert className='mt-0.5 size-3.5 shrink-0' />
                    {t('credentials.secret.forbidden')}
                  </p>
                )}
                {failed && (
                  <p className='flex items-start gap-1.5 text-xs text-fk-danger'>
                    <ShieldAlert className='mt-0.5 size-3.5 shrink-0' />
                    {t('credentials.secret.failed')}
                  </p>
                )}
                {confidential && (
                  <div className='flex items-center gap-3 pt-1'>
                    <Dialog open={rotateOpen} onOpenChange={setRotateOpen}>
                      <DialogTrigger asChild>
                        <Button type='button' variant='outline' size='sm' disabled={rotating}>
                          {rotating ? (
                            <Loader2 className='size-4 animate-spin' />
                          ) : (
                            <RefreshCw className='size-4' />
                          )}
                          {t('credentials.rotate.action')}
                        </Button>
                      </DialogTrigger>
                      <RotateDialogContent
                        graceSecs={graceSecs}
                        rotating={rotating}
                        replacesPrevious={Boolean(previous)}
                        onGraceChange={setGraceSecs}
                        onCancel={() => setRotateOpen(false)}
                        onConfirm={() => {
                          setRotateOpen(false)
                          onRotate(graceSecs)
                        }}
                      />
                    </Dialog>
                    <p className='text-xs text-neutral-400 dark:text-neutral-500'>
                      {t('credentials.secret.rotation_hint')}
                    </p>
                  </div>
                )}
              </div>
            </FieldRow>

            {previous && (
              <FieldRow
                label={t('credentials.previous.label')}
                description={t('credentials.previous.description')}
              >
                <div className='max-w-lg space-y-2'>
                  <div className='flex items-center gap-2'>
                    <Fingerprint value={previousSecret ?? MASKED_SECRET} muted={!previousSecret} />
                    <CopyButton value={previousSecret ?? ''} disabled={!previousSecret} />
                    <AlertDialog open={revokeOpen} onOpenChange={setRevokeOpen}>
                      <AlertDialogTrigger asChild>
                        <Button
                          type='button'
                          variant='destructive'
                          size='sm'
                          disabled={revoking || rotating}
                        >
                          {revoking && <Loader2 className='size-4 animate-spin' />}
                          {t('credentials.previous.revoke.action')}
                        </Button>
                      </AlertDialogTrigger>
                      <AlertDialogContent className='sm:max-w-md'>
                        <AlertDialogHeader>
                          <AlertDialogTitle className='text-base'>
                            {t('credentials.previous.revoke.title')}
                          </AlertDialogTitle>
                          <AlertDialogDescription>
                            {t('credentials.previous.revoke.description')}
                          </AlertDialogDescription>
                        </AlertDialogHeader>
                        <AlertDialogFooter>
                          <Button variant='ghost' size='sm' onClick={() => setRevokeOpen(false)}>
                            {t('credentials.rotate.cancel')}
                          </Button>
                          <Button
                            variant='destructive'
                            size='sm'
                            disabled={revoking || rotating}
                            onClick={() => {
                              setRevokeOpen(false)
                              onRevokePrevious()
                            }}
                          >
                            {t('credentials.previous.revoke.confirm')}
                          </Button>
                        </AlertDialogFooter>
                      </AlertDialogContent>
                    </AlertDialog>
                  </div>
                  <p className='text-xs text-neutral-500 dark:text-neutral-400'>
                    {t('credentials.previous.expires', {
                      date: formatDateTime(previous.expires_at),
                    })}
                  </p>
                </div>
              </FieldRow>
            )}
          </>
        )}
      </Section>

      {authentication && <ClientAuthenticationSection {...authentication} />}
    </>
  )
}
