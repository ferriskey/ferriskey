import { useTranslation } from 'react-i18next'
import { Input } from '@/components/ui/input'
import { Textarea } from '@/components/ui/textarea'
import { FieldRow, Pill, Section } from '@/components/kit'
import { DangerZone } from '@/components/kit/danger-zone'
import { Schemas } from '@/api/api.client'
import { roleScopeHintKey, roleScopeLabelKey } from '../role-scope'

import Role = Schemas.Role

export interface RoleSettingsTabProps {
  role: Role
  name: string
  description: string
  nameError?: string
  onNameChange: (v: string) => void
  onDescriptionChange: (v: string) => void
  onDelete: () => void
}

export default function RoleSettingsTab({
  role,
  name,
  description,
  nameError,
  onNameChange,
  onDescriptionChange,
  onDelete,
}: RoleSettingsTabProps) {
  const { t } = useTranslation('role')
  const isClientRole = Boolean(role.client_id)

  return (
    <>
      <Section title={t('detail.settings.title')}>
        <FieldRow
          label={t('form.name.label')}
          description={t('form.name.description')}
          htmlFor='role-name'
        >
          <Input
            id='role-name'
            value={name}
            onChange={(e) => onNameChange(e.target.value)}
            className='max-w-sm'
            aria-invalid={Boolean(nameError)}
          />
          {nameError && <p className='mt-1.5 text-fk-danger text-xs'>{nameError}</p>}
        </FieldRow>

        <FieldRow
          label={t('form.description.label')}
          description={t('form.description.description')}
          htmlFor='role-description'
        >
          <Textarea
            id='role-description'
            value={description}
            onChange={(e) => onDescriptionChange(e.target.value)}
            className='max-w-lg'
            rows={3}
          />
        </FieldRow>

        <FieldRow label={t('form.scope.label')} description={t(roleScopeHintKey(isClientRole))}>
          <div className='flex items-center gap-2'>
            <Pill tone={isClientRole ? 'violet' : 'info'} mono>
              {t(roleScopeLabelKey(isClientRole))}
            </Pill>
            {role.client?.client_id && <Pill mono>{role.client.client_id}</Pill>}
          </div>
        </FieldRow>
      </Section>

      <DangerZone
        resourceName={role.name}
        label={t('detail.settings.danger.label')}
        description={t('detail.settings.danger.description')}
        buttonLabel={t('detail.settings.danger.button')}
        confirmTitle={t('detail.settings.danger.confirm_title')}
        confirmDescription={t('detail.settings.danger.confirm_description', { name: role.name })}
        onConfirm={onDelete}
      />
    </>
  )
}
