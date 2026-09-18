import { useTranslation } from 'react-i18next'
import { Input } from '@/components/ui/input'
import { FieldRow, Section } from '@/components/kit'
import { DangerZone } from '@/components/kit/danger-zone'
import { cn } from '@/lib/utils'
import { tokens } from '@/styles/style-tokens'
import { Schemas } from '@/api/api.client'
import {
  USER_FEDERATION_NAMESPACE,
  flattenConfig,
  humanizeConfigKey,
  isLdapLike,
  isSecretKey,
  type LdapSettings,
  type ProviderPriority,
} from '../provider-config'
import {
  BasicSettingsFields,
  LdapConnectionFields,
  type ProviderErrors,
} from './provider-form-fields'

import ProviderResponse = Schemas.ProviderResponse

export interface ProviderSettingsTabProps {
  provider: ProviderResponse
  name: string
  enabled: boolean
  priority: ProviderPriority
  ldap: LdapSettings
  bindPassword: string
  secretStored: boolean
  secretRequired: boolean
  errors: ProviderErrors
  onNameChange: (v: string) => void
  onEnabledChange: (v: boolean) => void
  onPriorityChange: (v: ProviderPriority) => void
  onLdapChange: (patch: Partial<LdapSettings>) => void
  onBindPasswordChange: (v: string) => void
  onDelete: () => void
}

export default function ProviderSettingsTab({
  provider,
  name,
  enabled,
  priority,
  ldap,
  bindPassword,
  secretStored,
  secretRequired,
  errors,
  onNameChange,
  onEnabledChange,
  onPriorityChange,
  onLdapChange,
  onBindPasswordChange,
  onDelete,
}: ProviderSettingsTabProps) {
  const { t } = useTranslation(USER_FEDERATION_NAMESPACE)
  const editable = isLdapLike(provider.provider_type)
  const entries = flattenConfig(provider.config)

  return (
    <>
      <BasicSettingsFields
        name={name}
        enabled={enabled}
        priority={priority}
        errors={errors}
        onNameChange={onNameChange}
        onEnabledChange={onEnabledChange}
        onPriorityChange={onPriorityChange}
      >
        <FieldRow
          label={t('detail.settings.type.label')}
          description={t('detail.settings.type.description')}
          htmlFor='provider-type'
        >
          <Input
            id='provider-type'
            value={provider.provider_type}
            disabled
            className='max-w-sm'
          />
        </FieldRow>
      </BasicSettingsFields>

      {editable ? (
        <LdapConnectionFields
          settings={ldap}
          bindPassword={bindPassword}
          secretStored={secretStored}
          secretRequired={secretRequired}
          errors={errors}
          onChange={onLdapChange}
          onBindPasswordChange={onBindPasswordChange}
        />
      ) : (
        <Section
          title={t('detail.settings.config.title')}
          description={t('detail.settings.config.description')}
        >
          {entries.length > 0 ? (
            <dl className={tokens.surface.divider}>
              {entries.map(([key, value]) => (
                <div
                  key={key}
                  className='grid gap-x-6 gap-y-1 py-3 md:grid-cols-[minmax(0,18rem)_minmax(0,1fr)]'
                >
                  <dt className='text-xs font-medium text-neutral-900 dark:text-neutral-100'>
                    {humanizeConfigKey(key)}
                  </dt>
                  <dd className='min-w-0'>
                    <span
                      className={cn(
                        'block break-all font-mono-ui text-xs',
                        isSecretKey(key)
                          ? 'text-neutral-400 dark:text-neutral-500'
                          : 'text-neutral-700 dark:text-neutral-300'
                      )}
                    >
                      {value}
                    </span>
                  </dd>
                </div>
              ))}
            </dl>
          ) : (
            <p className='py-3 text-xs text-neutral-400 dark:text-neutral-500'>
              {t('detail.settings.config.empty')}
            </p>
          )}
        </Section>
      )}

      <DangerZone
        resourceName={provider.name}
        label={t('detail.settings.danger.label')}
        description={t('detail.settings.danger.description')}
        buttonLabel={t('detail.settings.danger.button')}
        confirmTitle={t('detail.settings.danger.confirm_title')}
        confirmDescription={t('detail.settings.danger.confirm_description', {
          name: provider.name,
        })}
        onConfirm={onDelete}
      />
    </>
  )
}
