import { AlertTriangle } from 'lucide-react'
import { Input } from '@/components/ui/input'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import FloatingActionBar from '@/components/ui/floating-action-bar'
import { DangerZone } from '@/components/danger-zone'
import { FieldRow, Pill, Section } from '@/components/kit'
import { tokens } from '@/styles/style-tokens'
import { ENCRYPTIONS, type SmtpConfigDraft } from '../smtp-form'

export interface SmtpTabProps {
  draft: SmtpConfigDraft
  errors: Partial<Record<keyof SmtpConfigDraft, string>>
  hasConfig: boolean
  isLoading: boolean
  dirty: boolean
  onChange: <K extends keyof SmtpConfigDraft>(field: K, value: SmtpConfigDraft[K]) => void
  onDiscard: () => void
  onSave: () => void
  onDelete: () => void
}

export default function SmtpTab({
  draft,
  errors,
  hasConfig,
  isLoading,
  dirty,
  onChange,
  onDiscard,
  onSave,
  onDelete,
}: SmtpTabProps) {
  if (isLoading) {
    return (
      <div className={tokens.page.blockGap}>
        <div className='h-4 w-32 animate-pulse rounded bg-neutral-100' />
        <div className='h-64 animate-pulse rounded-sm bg-neutral-100' />
      </div>
    )
  }

  const selected = ENCRYPTIONS.find((option) => option.value === draft.encryption)

  return (
    <div className={tokens.page.blockGap}>
      {!hasConfig && (
        <div className='flex items-start gap-2.5 rounded-sm border border-fk-amber-border bg-fk-amber-soft/50 px-4 py-3'>
          <AlertTriangle className='mt-0.5 size-4 shrink-0 text-fk-amber' strokeWidth={2} />
          <p className='text-xs text-neutral-700'>
            This realm has no SMTP server. No transactional email leaves — password resets, magic
            links and verifications are stored but never delivered.
          </p>
        </div>
      )}

      <Section title='SMTP server' description='The server this realm sends its emails through.'>
        <FieldRow label='Host' description='Hostname of the SMTP server.' htmlFor='smtp-host'>
          <Input
            id='smtp-host'
            value={draft.host}
            onChange={(event) => onChange('host', event.target.value)}
            placeholder='smtp.example.com'
            className='max-w-sm font-mono-ui'
            aria-invalid={Boolean(errors.host)}
          />
          {errors.host && <p className='mt-1.5 text-xs text-fk-danger'>{errors.host}</p>}
        </FieldRow>

        <FieldRow label='Port' description='Port of the SMTP server — 587, 465 or 25.' htmlFor='smtp-port'>
          <Input
            id='smtp-port'
            type='number'
            inputMode='numeric'
            value={Number.isNaN(draft.port) ? '' : draft.port}
            onChange={(event) => onChange('port', event.target.valueAsNumber)}
            onBlur={() => {
              if (Number.isNaN(draft.port)) onChange('port', 587)
            }}
            className='tnum max-w-[8rem]'
            aria-invalid={Boolean(errors.port)}
          />
          {errors.port && <p className='mt-1.5 text-xs text-fk-danger'>{errors.port}</p>}
        </FieldRow>

        <FieldRow label='Encryption' description='How the connection to the server is encrypted.'>
          <div className='max-w-sm'>
            <Select
              value={draft.encryption}
              onValueChange={(value) =>
                onChange('encryption', value as SmtpConfigDraft['encryption'])
              }
            >
              <SelectTrigger className='w-full'>
                <SelectValue />
              </SelectTrigger>
              <SelectContent position='popper'>
                {ENCRYPTIONS.map((option) => (
                  <SelectItem key={option.value} value={option.value}>
                    {option.label}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
            {selected && <p className='mt-1.5 text-xs text-neutral-500'>{selected.hint}</p>}
          </div>
        </FieldRow>

        <FieldRow
          label='Username'
          description='Account used to authenticate against the server.'
          htmlFor='smtp-username'
        >
          <Input
            id='smtp-username'
            value={draft.username}
            onChange={(event) => onChange('username', event.target.value)}
            className='max-w-sm font-mono-ui'
            aria-invalid={Boolean(errors.username)}
          />
          {errors.username && <p className='mt-1.5 text-xs text-fk-danger'>{errors.username}</p>}
        </FieldRow>

        <FieldRow
          label='Password'
          description='Never returned by the API, so it cannot be pre-filled: it has to be typed again to save any change on this tab.'
          htmlFor='smtp-password'
        >
          <Input
            id='smtp-password'
            type='password'
            value={draft.password}
            onChange={(event) => onChange('password', event.target.value)}
            placeholder='••••••••'
            className='max-w-sm font-mono-ui'
            aria-invalid={Boolean(errors.password)}
          />
          {hasConfig && !draft.password && (
            <div className='mt-1.5 flex items-center gap-2'>
              <Pill tone='neutral'>stored, write-only</Pill>
            </div>
          )}
          {errors.password && <p className='mt-1.5 text-xs text-fk-danger'>{errors.password}</p>}
        </FieldRow>
      </Section>

      <Section title='Sender' description='What recipients see in their inbox.'>
        <FieldRow
          label='From email'
          description='Address the outgoing emails are sent from.'
          htmlFor='smtp-from-email'
        >
          <Input
            id='smtp-from-email'
            type='email'
            value={draft.from_email}
            onChange={(event) => onChange('from_email', event.target.value)}
            className='max-w-sm font-mono-ui'
            aria-invalid={Boolean(errors.from_email)}
          />
          {errors.from_email && <p className='mt-1.5 text-xs text-fk-danger'>{errors.from_email}</p>}
        </FieldRow>

        <FieldRow
          label='From name'
          description='Display name shown next to the address.'
          htmlFor='smtp-from-name'
        >
          <Input
            id='smtp-from-name'
            value={draft.from_name}
            onChange={(event) => onChange('from_name', event.target.value)}
            className='max-w-sm'
            aria-invalid={Boolean(errors.from_name)}
          />
          {errors.from_name && <p className='mt-1.5 text-xs text-fk-danger'>{errors.from_name}</p>}
        </FieldRow>
      </Section>

      {hasConfig && (
        <DangerZone
          label='Delete SMTP configuration'
          description='Remove the SMTP configuration for this realm. Email features (password reset, magic links, verification) will stop working.'
          buttonLabel='Delete SMTP config'
          confirmTitle='Delete SMTP configuration'
          confirmDescription='This will permanently remove the SMTP configuration for this realm. Email delivery will be disabled.'
          confirmText='delete'
          onConfirm={onDelete}
        />
      )}

      <FloatingActionBar
        show={dirty}
        title={hasConfig ? 'Unsaved SMTP changes' : 'SMTP server not configured yet'}
        description='Saving replaces the whole configuration, password included.'
        onCancel={onDiscard}
        cancelLabel='Discard'
        actions={[{ label: hasConfig ? 'Save changes' : 'Save configuration', onClick: onSave }]}
      />
    </div>
  )
}
