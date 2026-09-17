import { ArrowLeft, Building2, Globe } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { Button } from '@/components/kit/button'
import { Input } from '@/components/ui/input'
import { Textarea } from '@/components/ui/textarea'
import SaveBar from '@/components/kit/save-bar'
import { ChoiceCards, FieldRow, PageShell, Section, type Choice } from '@/components/kit'
import { tokens } from '@/styles/style-tokens'
import { Schemas } from '@/api/api.client'
import RoleClientPicker from './role-client-picker'
import RolePermissionsTab from './role-permissions-tab'

import Client = Schemas.Client

export type RoleScope = 'realm' | 'client'

export interface PageCreateRoleProps {
  clients: Client[]
  name: string
  description: string
  scope: RoleScope
  clientId?: string
  permissions: string[]
  errors: { name?: string; clientId?: string }
  canSubmit: boolean
  onNameChange: (v: string) => void
  onDescriptionChange: (v: string) => void
  onScopeChange: (v: RoleScope) => void
  onClientIdChange: (v: string) => void
  onPermissionsChange: (next: string[]) => void
  onBack: () => void
  onSubmit: () => void
}

const SCOPE_CHOICES = [
  { value: 'realm', icon: Globe },
  { value: 'client', icon: Building2 },
] as const

export default function PageCreateRole({
  clients,
  name,
  description,
  scope,
  clientId,
  permissions,
  errors,
  canSubmit,
  onNameChange,
  onDescriptionChange,
  onScopeChange,
  onClientIdChange,
  onPermissionsChange,
  onBack,
  onSubmit,
}: PageCreateRoleProps) {
  const { t } = useTranslation('role')

  const scopeOptions: Choice<RoleScope>[] = SCOPE_CHOICES.map((choice) => ({
    value: choice.value,
    label: t(`form.scope.options.${choice.value}.label`),
    description: t(`form.scope.options.${choice.value}.description`),
    icon: choice.icon,
  }))

  return (
    <PageShell>
      <Button variant='ghost' size='sm' className='-ml-2 mb-2 text-neutral-500 dark:text-neutral-400' onClick={onBack}>
        <ArrowLeft className='size-3.5' />
        {t('create.back')}
      </Button>

      <div className='pb-3'>
        <h1 className={tokens.header.title}>{t('create.title')}</h1>
        <p className='mt-0.5 text-sm text-neutral-500 dark:text-neutral-400'>
          {t('create.subtitle')}
        </p>
      </div>

      <div className={tokens.page.blockGap}>
        <Section title={t('create.section.title')}>
          <FieldRow
            label={t('form.name.label')}
            description={t('form.name.description')}
            htmlFor='new-role-name'
          >
            <Input
              id='new-role-name'
              value={name}
              onChange={(e) => onNameChange(e.target.value)}
              className='max-w-sm'
              aria-invalid={Boolean(errors.name)}
            />
            {errors.name && <p className='mt-1.5 text-fk-danger text-xs'>{errors.name}</p>}
          </FieldRow>

          <FieldRow
            label={t('form.description.label')}
            description={t('form.description.description')}
            htmlFor='new-role-description'
          >
            <Textarea
              id='new-role-description'
              value={description}
              onChange={(e) => onDescriptionChange(e.target.value)}
              className='max-w-lg'
              rows={3}
            />
          </FieldRow>

          <FieldRow
            label={t('form.scope.label')}
            description={t('form.scope.create_description')}
          >
            <ChoiceCards
              label={t('form.scope.picker_label')}
              value={scope}
              onChange={onScopeChange}
              options={scopeOptions}
            />
          </FieldRow>

          {scope === 'client' && (
            <FieldRow label={t('form.client.label')} description={t('form.client.description')}>
              <RoleClientPicker
                clients={clients}
                value={clientId}
                onChange={onClientIdChange}
              />
              {errors.clientId && (
                <p className='mt-1.5 text-xs text-fk-danger'>{errors.clientId}</p>
              )}
            </FieldRow>
          )}
        </Section>

        <RolePermissionsTab value={permissions} onChange={onPermissionsChange} />
      </div>

      <SaveBar
        show={canSubmit}
        title={t('create.save_bar.title')}
        description={t('create.save_bar.description')}
        onCancel={onBack}
        actions={[{ label: t('create.save_bar.submit'), onClick: onSubmit }]}
      />
    </PageShell>
  )
}
