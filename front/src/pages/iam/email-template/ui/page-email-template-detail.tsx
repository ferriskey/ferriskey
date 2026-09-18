import { useState } from 'react'
import { AlertTriangle, ArrowLeft, Copy, Download, Pencil } from 'lucide-react'
import { Trans, useTranslation } from 'react-i18next'
import { Button } from '@/components/kit/button'
import { Input } from '@/components/ui/input'
import SaveBar from '@/components/kit/save-bar'
import { DangerZone } from '@/components/kit/danger-zone'
import { DetailHeader, FieldRow, IconTile, PageShell, Pill, Section } from '@/components/kit'
import { cn } from '@/lib/utils'
import { tokens } from '@/styles/style-tokens'
import { Schemas } from '@/api/api.client'
import {
  EMAIL_TEMPLATE_NAMESPACE,
  EXPORT_JSON,
  EXPORT_MJML,
  citedVariables,
  emailTypeSpec,
  variableToken,
  type EmailTypeSpec,
} from '../email-types'

import EmailTemplate = Schemas.EmailTemplate
import TemplateVariable = Schemas.TemplateVariable
import { formatDate } from '@/utils/format-date'

const LIST_SEPARATOR = ', '

const COPIED_FEEDBACK_MS = 1500

export interface PageEmailTemplateDetailProps {
  template?: EmailTemplate
  isLoading: boolean
  variables: TemplateVariable[]
  assignedTo: EmailTypeSpec[]
  name: string
  nameError?: string
  dirty: boolean
  onNameChange: (value: string) => void
  onBack: () => void
  onOpenBuilder: () => void
  onExport: (format: 'json' | 'mjml') => void
  onDiscard: () => void
  onSave: () => void
  onDelete: () => void
}

export default function PageEmailTemplateDetail({
  template,
  isLoading,
  variables,
  assignedTo,
  name,
  nameError,
  dirty,
  onNameChange,
  onBack,
  onOpenBuilder,
  onExport,
  onDiscard,
  onSave,
  onDelete,
}: PageEmailTemplateDetailProps) {
  const { t } = useTranslation(EMAIL_TEMPLATE_NAMESPACE)
  const [copied, setCopied] = useState(false)

  if (isLoading) {
    return (
      <PageShell>
        <div className='h-4 w-24 animate-pulse rounded bg-neutral-100 dark:bg-fk-raised' />
        <div className='mt-4 flex items-center gap-3'>
          <div className='size-15 animate-pulse rounded-md bg-neutral-100 dark:bg-fk-raised' />
          <div className='space-y-2'>
            <div className='h-5 w-48 animate-pulse rounded bg-neutral-100 dark:bg-fk-raised' />
            <div className='h-4 w-32 animate-pulse rounded bg-neutral-100 dark:bg-fk-raised' />
          </div>
        </div>
      </PageShell>
    )
  }

  if (!template) {
    return (
      <PageShell>
        <Button variant='ghost' size='sm' className='-ml-2 mb-2 text-neutral-500 dark:text-neutral-400' onClick={onBack}>
          <ArrowLeft className='size-3.5' />
          {t('detail.back')}
        </Button>
        <div className={cn(tokens.surface.panel, 'grid place-items-center px-6 py-16')}>
          <p className='text-sm font-medium text-neutral-700 dark:text-neutral-300'>
            {t('detail.not_found.title')}
          </p>
          <p className='mt-1 max-w-sm text-center text-sm text-neutral-500 dark:text-neutral-400'>
            {t('detail.not_found.hint')}
          </p>
        </div>
      </PageShell>
    )
  }

  const spec = emailTypeSpec(template.email_type)
  const cited = citedVariables(template.mjml)
  const unsupplied = [...cited].filter((used) => !variables.some((v) => v.name === used))

  return (
    <PageShell>
      <DetailHeader
        onBack={onBack}
        backLabel={t('detail.back')}
        icon={
          <IconTile tone={spec.tone} className='size-15'>
            <spec.icon className='size-6' strokeWidth={1.5} />
          </IconTile>
        }
        title={template.name}
        pills={
          <>
            <Pill tone='neutral' mono>
              {template.email_type}
            </Pill>
            {assignedTo.length > 0 ? (
              <Pill tone='success'>{t('detail.pills.assigned')}</Pill>
            ) : (
              <Pill tone='amber'>{t('detail.pills.not_assigned')}</Pill>
            )}
          </>
        }
        meta={
          <div className='flex shrink-0 items-start gap-4'>
            <dl className='text-right text-xs text-neutral-500 dark:text-neutral-400'>
              <dt className='sr-only'>{t('detail.meta.updated_at')}</dt>
              <dd className='tnum'>
                {t('detail.meta.updated', { date: formatDate(template.updated_at) })}
              </dd>
              <dt className='sr-only'>{t('detail.meta.identifier')}</dt>
              <dd className='font-mono-ui text-[11px] text-neutral-400 dark:text-neutral-500'>{template.id}</dd>
            </dl>
            <div className='flex gap-2'>
              <Button variant='outline' onClick={() => onExport(EXPORT_JSON)}>
                <Download /> {t('detail.meta.export')}
              </Button>
              <Button onClick={onOpenBuilder}>
                <Pencil /> {t('detail.meta.open_builder')}
              </Button>
            </div>
          </div>
        }
      />

      <div className={cn('mt-5', tokens.page.blockGap)}>
        <Section title={t('detail.general.title')} description={t(spec.triggerKey)}>
          <FieldRow
            label={t('detail.general.name.label')}
            description={t('detail.general.name.description')}
            htmlFor='email-template-name'
          >
            <Input
              id='email-template-name'
              value={name}
              onChange={(event) => onNameChange(event.target.value)}
              className='max-w-sm'
              aria-invalid={Boolean(nameError)}
            />
            {nameError && <p className='mt-1.5 text-fk-danger text-xs'>{nameError}</p>}
          </FieldRow>

          <FieldRow
            label={t('detail.general.type.label')}
            description={t('detail.general.type.description')}
            htmlFor='email-template-type'
          >
            <Input
              id='email-template-type'
              value={template.email_type}
              disabled
              className='max-w-sm'
            />
          </FieldRow>

          <FieldRow
            label={t('detail.general.used_for.label')}
            description={t('detail.general.used_for.description')}
          >
            <div className='flex flex-wrap items-center gap-2'>
              {assignedTo.length > 0 ? (
                assignedTo.map((used) => (
                  <Pill key={used.key} tone='success'>
                    {t(used.labelKey)}
                  </Pill>
                ))
              ) : (
                <span className='text-sm text-neutral-400 dark:text-neutral-500'>
                  {t('detail.general.used_for.empty')}
                </span>
              )}
            </div>
          </FieldRow>
        </Section>

        <Section
          title={t('detail.variables.title')}
          description={t('detail.variables.description')}
        >
          {variables.map((variable) => {
            const used = cited.has(variable.name)
            return (
              <div
                key={variable.name}
                className='grid gap-x-8 gap-y-1 py-2.5 md:grid-cols-[minmax(0,20rem)_minmax(0,1fr)]'
              >
                <code
                  className={cn(
                    'w-fit rounded border px-1.5 py-0.5 font-mono-ui text-xs',
                    used
                      ? 'border-fk-success-border bg-fk-success-soft text-fk-success'
                      : 'border-fk-line bg-neutral-50 text-neutral-500 dark:bg-fk-surface dark:text-neutral-400'
                  )}
                >
                  {variableToken(variable.name)}
                </code>
                <p className='min-w-0 text-xs text-neutral-500 dark:text-neutral-400'>
                  {variable.description}
                  {!used && (
                    <span className='text-neutral-400 dark:text-neutral-500'>
                      {' '}
                      {t('detail.variables.absent')}
                    </span>
                  )}
                </p>
              </div>
            )
          })}
        </Section>

        {unsupplied.length > 0 && (
          <div className='flex items-start gap-2.5 rounded-sm border border-fk-amber-border bg-fk-amber-soft/50 px-4 py-3'>
            <AlertTriangle className='mt-0.5 size-4 shrink-0 text-fk-amber' strokeWidth={2} />
            <p className='text-xs text-neutral-700 dark:text-neutral-300'>
              <Trans
                i18nKey='detail.variables.unsupplied'
                ns={EMAIL_TEMPLATE_NAMESPACE}
                count={unsupplied.length}
                values={{
                  names: unsupplied.map(variableToken).join(LIST_SEPARATOR),
                  type: template.email_type,
                }}
                components={{ code: <code className='font-mono-ui' /> }}
              />
            </p>
          </div>
        )}

        <Section
          title={t('detail.mjml.title')}
          description={t('detail.mjml.description')}
          action={
            <div className='flex items-center gap-2'>
              <Button variant='ghost' size='sm' className='text-xs' onClick={() => onExport(EXPORT_MJML)}>
                <Download /> {t('detail.mjml.export')}
              </Button>
              <Button
                variant='ghost'
                size='sm'
                className='text-xs'
                onClick={() => {
                  void navigator.clipboard.writeText(template.mjml)
                  setCopied(true)
                  window.setTimeout(() => setCopied(false), COPIED_FEEDBACK_MS)
                }}
              >
                <Copy /> {copied ? t('detail.mjml.copied') : t('detail.mjml.copy')}
              </Button>
            </div>
          }
          contained={false}
        >
          <pre className='max-h-96 overflow-auto rounded-sm border border-fk-line bg-neutral-50 px-3 py-2 font-mono-ui text-xs leading-relaxed text-neutral-700 dark:bg-fk-surface dark:text-neutral-300'>
            {template.mjml}
          </pre>
        </Section>

        <DangerZone
          resourceName={template.name}
          label={t('detail.danger.label')}
          description={
            assignedTo.length > 0
              ? t('detail.danger.assigned', {
                  count: assignedTo.length,
                  targets: assignedTo.map((used) => t(used.labelKey)).join(LIST_SEPARATOR),
                })
              : t('detail.danger.plain')
          }
          buttonLabel={t('detail.danger.button')}
          confirmTitle={t('detail.danger.confirm_title')}
          confirmDescription={t('detail.danger.confirm_description', { name: template.name })}
          onConfirm={onDelete}
        />
      </div>

      <SaveBar
        show={dirty}
        title={t('detail.save.title')}
        description={t('detail.save.description')}
        onCancel={onDiscard}
        cancelLabel={t('detail.save.cancel')}
        actions={[{ label: t('detail.save.action'), onClick: onSave }]}
      />
    </PageShell>
  )
}
