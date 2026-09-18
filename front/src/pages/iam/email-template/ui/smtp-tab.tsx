import { AlertTriangle } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { Input } from '@/components/ui/input'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import SaveBar from '@/components/kit/save-bar'
import { DangerZone } from '@/components/kit/danger-zone'
import { FieldRow, Pill, Section } from '@/components/kit'
import { tokens } from '@/styles/style-tokens'
import {
  DEFAULT_SMTP_PORT,
  ENCRYPTIONS,
  SMTP_DELETE_TOKEN,
  SMTP_FIELD,
  SMTP_HOST_PLACEHOLDER,
  SMTP_PASSWORD_PLACEHOLDER,
  type SmtpConfigDraft,
} from '../smtp-form'
import { EMAIL_TEMPLATE_NAMESPACE } from '../email-types'

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
  const { t } = useTranslation(EMAIL_TEMPLATE_NAMESPACE)

  if (isLoading) {
    return (
      <div className={tokens.page.blockGap}>
        <div className='h-4 w-32 animate-pulse rounded bg-neutral-100 dark:bg-fk-raised' />
        <div className='h-64 animate-pulse rounded-sm bg-neutral-100 dark:bg-fk-raised' />
      </div>
    )
  }

  const selected = ENCRYPTIONS.find((option) => option.value === draft.encryption)

  return (
    <div className={tokens.page.blockGap}>
      {!hasConfig && (
        <div className='flex items-start gap-2.5 rounded-sm border border-fk-amber-border bg-fk-amber-soft/50 px-4 py-3'>
          <AlertTriangle className='mt-0.5 size-4 shrink-0 text-fk-amber' strokeWidth={2} />
          <p className='text-xs text-neutral-700 dark:text-neutral-300'>{t('smtp.alert.no_server')}</p>
        </div>
      )}

      <Section title={t('smtp.server.title')} description={t('smtp.server.description')}>
        <FieldRow
          label={t('smtp.server.host.label')}
          description={t('smtp.server.host.description')}
          htmlFor='smtp-host'
        >
          <Input
            id='smtp-host'
            value={draft.host}
            onChange={(event) => onChange(SMTP_FIELD.host, event.target.value)}
            placeholder={SMTP_HOST_PLACEHOLDER}
            className='max-w-sm'
            aria-invalid={Boolean(errors.host)}
          />
          {errors.host && <p className='mt-1.5 text-xs text-fk-danger'>{errors.host}</p>}
        </FieldRow>

        <FieldRow
          label={t('smtp.server.port.label')}
          description={t('smtp.server.port.description')}
          htmlFor='smtp-port'
        >
          <Input
            id='smtp-port'
            type='number'
            inputMode='numeric'
            value={Number.isNaN(draft.port) ? '' : draft.port}
            onChange={(event) => onChange(SMTP_FIELD.port, event.target.valueAsNumber)}
            onBlur={() => {
              if (Number.isNaN(draft.port)) onChange(SMTP_FIELD.port, DEFAULT_SMTP_PORT)
            }}
            className='tnum max-w-[8rem]'
            aria-invalid={Boolean(errors.port)}
          />
          {errors.port && <p className='mt-1.5 text-xs text-fk-danger'>{errors.port}</p>}
        </FieldRow>

        <FieldRow
          label={t('smtp.server.encryption.label')}
          description={t('smtp.server.encryption.description')}
        >
          <div className='max-w-sm'>
            <Select
              value={draft.encryption}
              onValueChange={(value) =>
                onChange(SMTP_FIELD.encryption, value as SmtpConfigDraft['encryption'])
              }
            >
              <SelectTrigger className='w-full'>
                <SelectValue />
              </SelectTrigger>
              <SelectContent position='popper'>
                {ENCRYPTIONS.map((option) => (
                  <SelectItem key={option.value} value={option.value}>
                    {t(option.labelKey)}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
            {selected && (
              <p className='mt-1.5 text-xs text-neutral-500 dark:text-neutral-400'>
                {t(selected.hintKey)}
              </p>
            )}
          </div>
        </FieldRow>

        <FieldRow
          label={t('smtp.server.username.label')}
          description={t('smtp.server.username.description')}
          htmlFor='smtp-username'
        >
          <Input
            id='smtp-username'
            value={draft.username}
            onChange={(event) => onChange(SMTP_FIELD.username, event.target.value)}
            className='max-w-sm'
            aria-invalid={Boolean(errors.username)}
          />
          {errors.username && <p className='mt-1.5 text-fk-danger text-xs'>{errors.username}</p>}
        </FieldRow>

        <FieldRow
          label={t('smtp.server.password.label')}
          description={t('smtp.server.password.description')}
          htmlFor='smtp-password'
        >
          <Input
            id='smtp-password'
            type='password'
            value={draft.password}
            onChange={(event) => onChange(SMTP_FIELD.password, event.target.value)}
            placeholder={SMTP_PASSWORD_PLACEHOLDER}
            className='max-w-sm'
            aria-invalid={Boolean(errors.password)}
          />
          {hasConfig && !draft.password && (
            <div className='mt-1.5 flex items-center gap-2'>
              <Pill tone='neutral'>{t('smtp.server.password.stored')}</Pill>
            </div>
          )}
          {errors.password && <p className='mt-1.5 text-xs text-fk-danger'>{errors.password}</p>}
        </FieldRow>
      </Section>

      <Section title={t('smtp.sender.title')} description={t('smtp.sender.description')}>
        <FieldRow
          label={t('smtp.sender.from_email.label')}
          description={t('smtp.sender.from_email.description')}
          htmlFor='smtp-from-email'
        >
          <Input
            id='smtp-from-email'
            type='email'
            value={draft.from_email}
            onChange={(event) => onChange(SMTP_FIELD.fromEmail, event.target.value)}
            className='max-w-sm'
            aria-invalid={Boolean(errors.from_email)}
          />
          {errors.from_email && <p className='mt-1.5 text-xs text-fk-danger'>{errors.from_email}</p>}
        </FieldRow>

        <FieldRow
          label={t('smtp.sender.from_name.label')}
          description={t('smtp.sender.from_name.description')}
          htmlFor='smtp-from-name'
        >
          <Input
            id='smtp-from-name'
            value={draft.from_name}
            onChange={(event) => onChange(SMTP_FIELD.fromName, event.target.value)}
            className='max-w-sm'
            aria-invalid={Boolean(errors.from_name)}
          />
          {errors.from_name && <p className='mt-1.5 text-fk-danger text-xs'>{errors.from_name}</p>}
        </FieldRow>
      </Section>

      {hasConfig && (
        <DangerZone
          label={t('smtp.danger.label')}
          description={t('smtp.danger.description')}
          buttonLabel={t('smtp.danger.button')}
          confirmTitle={t('smtp.danger.confirm_title')}
          confirmDescription={t('smtp.danger.confirm_description')}
          confirmText={SMTP_DELETE_TOKEN}
          onConfirm={onDelete}
        />
      )}

      <SaveBar
        show={dirty}
        title={hasConfig ? t('smtp.save.title_dirty') : t('smtp.save.title_new')}
        description={t('smtp.save.description')}
        onCancel={onDiscard}
        cancelLabel={t('smtp.save.cancel')}
        actions={[
          {
            label: hasConfig ? t('smtp.save.action_dirty') : t('smtp.save.action_new'),
            onClick: onSave,
          },
        ]}
      />
    </div>
  )
}
