import { useState } from 'react'
import { AlertTriangle, ArrowLeft, Copy, Download, Pencil } from 'lucide-react'
import { Button } from '@/components/kit/button'
import { Input } from '@/components/ui/input'
import SaveBar from '@/components/kit/save-bar'
import { DangerZone } from '@/components/kit/danger-zone'
import { FieldRow, IconTile, Pill, Section } from '@/components/kit'
import { cn } from '@/lib/utils'
import { tokens } from '@/styles/style-tokens'
import { Schemas } from '@/api/api.client'
import { citedVariables, emailTypeSpec, type EmailTypeSpec } from '../email-types'

import EmailTemplate = Schemas.EmailTemplate
import TemplateVariable = Schemas.TemplateVariable
import { formatDate } from '@/next/shared/format-date'

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
  const [copied, setCopied] = useState(false)
  const container = cn('mx-auto', tokens.page.maxWidth, tokens.page.padding)

  if (isLoading) {
    return (
      <div className={container}>
        <div className='h-4 w-24 animate-pulse rounded bg-neutral-100' />
        <div className='mt-4 flex items-center gap-3'>
          <div className='size-15 animate-pulse rounded-md bg-neutral-100' />
          <div className='space-y-2'>
            <div className='h-5 w-48 animate-pulse rounded bg-neutral-100' />
            <div className='h-4 w-32 animate-pulse rounded bg-neutral-100' />
          </div>
        </div>
      </div>
    )
  }

  if (!template) {
    return (
      <div className={container}>
        <Button variant='ghost' size='sm' className='-ml-2 mb-2 text-neutral-500' onClick={onBack}>
          <ArrowLeft className='size-3.5' />
          Emails
        </Button>
        <div className={cn(tokens.surface.panel, 'grid place-items-center px-6 py-16')}>
          <p className='text-sm font-medium text-neutral-700'>Email template not found</p>
          <p className='mt-1 max-w-sm text-center text-sm text-neutral-500'>
            It may have been deleted, or it belongs to another realm.
          </p>
        </div>
      </div>
    )
  }

  const spec = emailTypeSpec(template.email_type)
  const cited = citedVariables(template.mjml)
  const unsupplied = [...cited].filter((used) => !variables.some((v) => v.name === used))

  return (
    <div className={container}>
      <Button variant='ghost' size='sm' className='-ml-2 mb-2 text-neutral-500' onClick={onBack}>
        <ArrowLeft className='size-3.5' />
        Emails
      </Button>

      <div className='flex flex-wrap items-start justify-between gap-4'>
        <div className='flex items-center gap-3'>
          <IconTile tone={spec.tone} className='size-15'>
            <spec.icon className='size-6' strokeWidth={1.5} />
          </IconTile>
          <div className='min-w-0'>
            <h1 className={tokens.header.title}>{template.name}</h1>
            <div className='mt-1.5 flex flex-wrap items-center gap-2'>
              <Pill tone='neutral' mono>
                {template.email_type}
              </Pill>
              {assignedTo.length > 0 ? (
                <Pill tone='success'>assigned</Pill>
              ) : (
                <Pill tone='amber'>not assigned</Pill>
              )}
            </div>
          </div>
        </div>

        <div className='flex shrink-0 items-start gap-4'>
          <dl className='text-right text-xs text-neutral-500'>
            <dt className='sr-only'>Updated at</dt>
            <dd className='tnum'>
              Updated {formatDate(template.updated_at)}
            </dd>
            <dt className='sr-only'>Identifier</dt>
            <dd className='font-mono-ui text-[11px] text-neutral-400'>{template.id}</dd>
          </dl>
          <div className='flex gap-2'>
            <Button variant='outline' onClick={() => onExport('json')}>
              <Download /> Export
            </Button>
            <Button onClick={onOpenBuilder}>
              <Pencil /> Open the builder
            </Button>
          </div>
        </div>
      </div>

      <div className={cn('mt-5', tokens.page.blockGap)}>
        <Section title='General' description={spec.trigger}>
          <FieldRow
            label='Name'
            description='How this template is named in the console. Required.'
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
            label='Type'
            description='Fixed at creation: it decides which variables the engine supplies and which email uses this template.'
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
            label='Used for'
            description='Set on the Emails listing. An email with no template falls back to the plain-text default.'
          >
            <div className='flex flex-wrap items-center gap-2'>
              {assignedTo.length > 0 ? (
                assignedTo.map((used) => (
                  <Pill key={used.key} tone='success'>
                    {used.label}
                  </Pill>
                ))
              ) : (
                <span className='text-sm text-neutral-400'>No email uses this template</span>
              )}
            </div>
          </FieldRow>
        </Section>

        <Section
          title='Available variables'
          description='Interpolated at send time and HTML-escaped. Any other expression is left as it is.'
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
                      : 'border-fk-line bg-neutral-50 text-neutral-500'
                  )}
                >
                  {`{{${variable.name}}}`}
                </code>
                <p className='min-w-0 text-xs text-neutral-500'>
                  {variable.description}
                  {!used && <span className='text-neutral-400'> — absent from this template</span>}
                </p>
              </div>
            )
          })}
        </Section>

        {unsupplied.length > 0 && (
          <div className='flex items-start gap-2.5 rounded-sm border border-fk-amber-border bg-fk-amber-soft/50 px-4 py-3'>
            <AlertTriangle className='mt-0.5 size-4 shrink-0 text-fk-amber' strokeWidth={2} />
            <p className='text-xs text-neutral-700'>
              {unsupplied.map((variable) => `{{${variable}}}`).join(', ')}{' '}
              {unsupplied.length > 1 ? 'are not supplied' : 'is not supplied'} for the{' '}
              <code className='font-mono-ui'>{template.email_type}</code> type:{' '}
              {unsupplied.length > 1 ? 'they leave' : 'it leaves'} verbatim in the email the user
              receives.
            </p>
          </div>
        )}

        <Section
          title='MJML'
          description='Produced by the builder from the stored structure.'
          action={
            <div className='flex items-center gap-2'>
              <Button variant='ghost' size='sm' className='text-xs' onClick={() => onExport('mjml')}>
                <Download /> Export MJML
              </Button>
              <Button
                variant='ghost'
                size='sm'
                className='text-xs'
                onClick={() => {
                  void navigator.clipboard.writeText(template.mjml)
                  setCopied(true)
                  window.setTimeout(() => setCopied(false), 1500)
                }}
              >
                <Copy /> {copied ? 'Copied' : 'Copy'}
              </Button>
            </div>
          }
          contained={false}
        >
          <pre className='max-h-96 overflow-auto rounded-sm border border-fk-line bg-neutral-50 px-3 py-2 font-mono-ui text-xs leading-relaxed text-neutral-700'>
            {template.mjml}
          </pre>
        </Section>

        <DangerZone
        resourceName={template.name}
          label='Delete this template'
          description={
            assignedTo.length > 0
              ? `It is assigned to ${assignedTo
                  .map((used) => used.label)
                  .join(', ')}. Deleting it succeeds and unassigns it: ${
                  assignedTo.length > 1 ? 'those emails' : 'that email'
                } falls back to the plain-text default.`
              : 'Its structure and its rendered MJML are lost for good.'
          }
          buttonLabel='Delete template'
          confirmTitle='Delete email template'
          confirmDescription={`This will permanently delete the template "${template.name}".`}
          onConfirm={onDelete}
        />
      </div>

      <SaveBar
        show={dirty}
        title='1 unsaved change'
        description='Only the name is editable here; the content is edited in the builder.'
        onCancel={onDiscard}
        cancelLabel='Discard'
        actions={[{ label: 'Save changes', onClick: onSave }]}
      />
    </div>
  )
}
