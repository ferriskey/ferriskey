import { useTranslation } from 'react-i18next'
import { Input } from '@/components/ui/input'
import { Textarea } from '@/components/ui/textarea'
import { FieldRow, Section, SwitchField } from '@/components/kit'
import { DangerZone } from '@/components/kit/danger-zone'
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
  const { t } = useTranslation('organization')

  return (
    <>
      <Section title={t('form.sections.general')}>
        <FieldRow
          label={t('form.name.label')}
          description={t('form.name.description')}
          htmlFor='organization-name'
        >
          <Input
            id='organization-name'
            value={draft.name}
            onChange={(e) => onChange({ name: e.target.value })}
            className='max-w-sm'
            aria-invalid={Boolean(errors.name)}
          />
          {errors.name && <p className='mt-1.5 text-fk-danger text-xs'>{errors.name}</p>}
        </FieldRow>

        <FieldRow
          label={t('form.alias.label')}
          description={t('form.alias.description.settings')}
          htmlFor='organization-alias'
        >
          <Input
            id='organization-alias'
            value={draft.alias}
            onChange={(e) => onChange({ alias: e.target.value })}
            className='max-w-sm'
            aria-invalid={Boolean(errors.alias)}
          />
          {errors.alias && <p className='mt-1.5 text-fk-danger text-xs'>{errors.alias}</p>}
        </FieldRow>

        <FieldRow
          label={t('form.enabled.label')}
          description={t('form.enabled.description.settings')}
          htmlFor='organization-enabled'
        >
          <SwitchField
            id='organization-enabled'
            checked={draft.enabled}
            onCheckedChange={(enabled) => onChange({ enabled })}
          />
        </FieldRow>
      </Section>

      <Section
        title={t('form.sections.routing.title')}
        description={t('form.sections.routing.description')}
      >
        <FieldRow
          label={t('form.domain.label')}
          description={t('form.domain.description.settings')}
          htmlFor='organization-domain'
        >
          <Input
            id='organization-domain'
            value={draft.domain}
            onChange={(e) => onChange({ domain: e.target.value })}
            className='max-w-sm'
          />
        </FieldRow>

        <FieldRow
          label={t('form.redirect_url.label')}
          description={t('form.redirect_url.description')}
          htmlFor='organization-redirect-url'
        >
          <Input
            id='organization-redirect-url'
            value={draft.redirectUrl}
            onChange={(e) => onChange({ redirectUrl: e.target.value })}
            className='max-w-lg'
          />
        </FieldRow>

        <FieldRow
          label={t('form.description.label')}
          description={t('form.description.description')}
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
        resourceName={organization.name}
        label={t('detail.settings.danger.label')}
        description={t('detail.settings.danger.description')}
        buttonLabel={t('detail.settings.danger.button')}
        confirmTitle={t('detail.settings.danger.confirm_title')}
        confirmDescription={t('detail.settings.danger.confirm_description', {
          name: organization.name,
        })}
        onConfirm={onDelete}
      />
    </>
  )
}
