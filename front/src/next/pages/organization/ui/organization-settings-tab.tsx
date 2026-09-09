import { Input } from '@/components/ui/input'
import { Textarea } from '@/components/ui/textarea'
import { FieldRow, Section, SwitchField } from '@/components/kit'
import { DangerZone } from '@/components/danger-zone'
import { Schemas } from '@/api/api.client'

import Organization = Schemas.Organization

export interface OrganizationDraft {
  name: string
  alias: string
  enabled: boolean
  domain: string
  redirectUrl: string
  description: string
}

export interface OrganizationSettingsTabProps {
  organization: Organization
  draft: OrganizationDraft
  errors: { name?: string; alias?: string }
  onChange: (patch: Partial<OrganizationDraft>) => void
  onDelete: () => void
}

export default function OrganizationSettingsTab({
  organization,
  draft,
  errors,
  onChange,
  onDelete,
}: OrganizationSettingsTabProps) {
  return (
    <>
      <Section title='General'>
        <FieldRow
          label='Name'
          description='Shown to administrators wherever the organization is listed.'
          htmlFor='organization-name'
        >
          <Input
            id='organization-name'
            value={draft.name}
            onChange={(e) => onChange({ name: e.target.value })}
            className='max-w-sm'
            aria-invalid={Boolean(errors.name)}
          />
          {errors.name && <p className='mt-1.5 text-xs text-fk-danger'>{errors.name}</p>}
        </FieldRow>

        <FieldRow
          label='Alias'
          description='Stable identifier used in URLs and lookups. Renaming it breaks the links that already point here.'
          htmlFor='organization-alias'
        >
          <Input
            id='organization-alias'
            value={draft.alias}
            onChange={(e) => onChange({ alias: e.target.value })}
            className='max-w-sm font-mono-ui text-xs'
            aria-invalid={Boolean(errors.alias)}
          />
          {errors.alias && <p className='mt-1.5 text-xs text-fk-danger'>{errors.alias}</p>}
        </FieldRow>

        <FieldRow
          label='Enabled'
          description='A disabled organization is hidden from end-user flows.'
          htmlFor='organization-enabled'
        >
          <SwitchField
            id='organization-enabled'
            checked={draft.enabled}
            onCheckedChange={(enabled) => onChange({ enabled })}
          />
        </FieldRow>
      </Section>

      <Section title='Contact & routing' description='Optional metadata, none of it required.'>
        <FieldRow
          label='Domain'
          description='Primary domain associated with this organization, for example acme.com.'
          htmlFor='organization-domain'
        >
          <Input
            id='organization-domain'
            value={draft.domain}
            onChange={(e) => onChange({ domain: e.target.value })}
            className='max-w-sm font-mono-ui text-xs'
          />
        </FieldRow>

        <FieldRow
          label='Redirect URL'
          description='Where members land once they finished authenticating.'
          htmlFor='organization-redirect-url'
        >
          <Input
            id='organization-redirect-url'
            value={draft.redirectUrl}
            onChange={(e) => onChange({ redirectUrl: e.target.value })}
            className='max-w-lg font-mono-ui text-xs'
          />
        </FieldRow>

        <FieldRow
          label='Description'
          description='Notes visible to administrators only.'
          htmlFor='organization-description'
        >
          <Textarea
            id='organization-description'
            value={draft.description}
            onChange={(e) => onChange({ description: e.target.value })}
            className='max-w-lg'
            rows={3}
          />
        </FieldRow>
      </Section>

      <DangerZone
        label='Delete this organization'
        description='All members, attributes, groups and configuration are permanently removed. The accounts themselves are kept.'
        buttonLabel='Delete organization'
        confirmTitle='Delete organization'
        confirmDescription={`This will permanently delete "${organization.name}" and all its associated data.`}
        onConfirm={onDelete}
      />
    </>
  )
}
