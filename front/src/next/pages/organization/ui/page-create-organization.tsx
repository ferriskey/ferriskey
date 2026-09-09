import { ArrowLeft } from 'lucide-react'
import { Button } from '@/components/kit/button'
import { Input } from '@/components/ui/input'
import { Textarea } from '@/components/ui/textarea'
import SaveBar from '@/components/kit/save-bar'
import { FieldRow, Section, SwitchField } from '@/components/kit'
import { cn } from '@/lib/utils'
import { tokens } from '@/styles/style-tokens'

export interface CreateOrganizationDraft {
  name: string
  alias: string
  enabled: boolean
  domain: string
  redirectUrl: string
  description: string
}

export interface PageCreateOrganizationProps {
  draft: CreateOrganizationDraft
  errors: { name?: string; alias?: string }
  canSubmit: boolean
  onChange: (patch: Partial<CreateOrganizationDraft>) => void
  onBack: () => void
  onSubmit: () => void
}

export default function PageCreateOrganization({
  draft,
  errors,
  canSubmit,
  onChange,
  onBack,
  onSubmit,
}: PageCreateOrganizationProps) {
  return (
    <div className={cn('mx-auto', tokens.page.maxWidth, tokens.page.padding)}>
      <Button variant='ghost' size='sm' className='-ml-2 mb-2 text-neutral-500' onClick={onBack}>
        <ArrowLeft className='size-3.5' />
        Organizations
      </Button>

      <div className='pb-3'>
        <h1 className={tokens.header.title}>New organization</h1>
        <p className='mt-0.5 text-sm text-neutral-500'>
          An organization groups accounts of a same company, with its own domain and roles.
        </p>
      </div>

      <div className={tokens.page.blockGap}>
        <Section title='Definition'>
          <FieldRow
            label='Name'
            description='Shown to administrators wherever the organization is listed.'
            htmlFor='new-organization-name'
          >
            <Input
              id='new-organization-name'
              value={draft.name}
              onChange={(e) => onChange({ name: e.target.value })}
              className='max-w-sm'
              aria-invalid={Boolean(errors.name)}
            />
            {errors.name && <p className='mt-1.5 text-fk-danger'>{errors.name}</p>}
          </FieldRow>

          <FieldRow
            label='Alias'
            description='Stable identifier used in URLs and lookups. Lowercase letters, digits, hyphens and underscores only.'
            htmlFor='new-organization-alias'
          >
            <Input
              id='new-organization-alias'
              value={draft.alias}
              onChange={(e) => onChange({ alias: e.target.value })}
              className='max-w-sm'
              aria-invalid={Boolean(errors.alias)}
            />
            {errors.alias && <p className='mt-1.5 text-fk-danger'>{errors.alias}</p>}
          </FieldRow>

          <FieldRow
            label='Enabled'
            description='A disabled organization is not accessible to its members.'
            htmlFor='new-organization-enabled'
          >
            <SwitchField
              id='new-organization-enabled'
              checked={draft.enabled}
              onCheckedChange={(enabled) => onChange({ enabled })}
            />
          </FieldRow>
        </Section>

        <Section title='Contact & routing'>
          <FieldRow
            label='Domain'
            description='Email domain associated with this organization, for example acme.com.'
            htmlFor='new-organization-domain'
          >
            <Input
              id='new-organization-domain'
              value={draft.domain}
              onChange={(e) => onChange({ domain: e.target.value })}
              className='max-w-sm'
            />
          </FieldRow>

          <FieldRow
            label='Redirect URL'
            description='Where members land once they finished authenticating.'
            htmlFor='new-organization-redirect-url'
          >
            <Input
              id='new-organization-redirect-url'
              value={draft.redirectUrl}
              onChange={(e) => onChange({ redirectUrl: e.target.value })}
              className='max-w-lg'
            />
          </FieldRow>

          <FieldRow
            label='Description'
            description='Notes visible to administrators only.'
            htmlFor='new-organization-description'
          >
            <Textarea
              id='new-organization-description'
              value={draft.description}
              onChange={(e) => onChange({ description: e.target.value })}
              className='max-w-lg'
              rows={3}
            />
          </FieldRow>
        </Section>
      </div>

      <SaveBar
        show={canSubmit}
        title='Create organization'
        description='The organization is created in this realm, with no member yet.'
        onCancel={onBack}
        actions={[{ label: 'Create organization', onClick: onSubmit }]}
      />
    </div>
  )
}
