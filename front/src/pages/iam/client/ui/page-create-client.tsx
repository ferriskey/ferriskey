import { ArrowLeft } from 'lucide-react'
import { Link } from 'react-router-dom'
import { useTranslation } from 'react-i18next'
import { Button } from '@/components/kit/button'
import { Input } from '@/components/ui/input'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import SaveBar from '@/components/kit/save-bar'
import { ChoiceCards, FieldRow, PageShell, Pill, Section, SwitchField } from '@/components/kit'
import { NAME_ID_FORMAT_OPTIONS } from '@/lib/saml'
import { tokens } from '@/styles/style-tokens'
import {
  authenticationChoices,
  clientStateOf,
  isEnabledState,
  stateChoices,
  type ClientAuthentication,
  type ClientProtocol,
  type ClientState,
} from '../client-choices'

export interface CreateClientErrors {
  clientId?: string
  name?: string
  spEntityId?: string
  acsUrl?: string
}

export interface PageCreateClientProps {
  protocol: ClientProtocol
  listUrl: string
  pickerUrl: string
  clientId: string
  name: string
  enabled: boolean
  authentication: ClientAuthentication
  directAccessGrants: boolean
  deviceCodeGrant: boolean
  spEntityId: string
  acsUrl: string
  nameIdFormat: string
  errors: CreateClientErrors
  canSubmit: boolean
  onClientIdChange: (v: string) => void
  onNameChange: (v: string) => void
  onEnabledChange: (v: boolean) => void
  onAuthenticationChange: (v: ClientAuthentication) => void
  onDirectAccessGrantsChange: (v: boolean) => void
  onDeviceCodeGrantChange: (v: boolean) => void
  onSpEntityIdChange: (v: string) => void
  onAcsUrlChange: (v: string) => void
  onNameIdFormatChange: (v: string) => void
  onBack: () => void
  onSubmit: () => void
}

export default function PageCreateClient({
  protocol,
  listUrl,
  pickerUrl,
  clientId,
  name,
  enabled,
  authentication,
  directAccessGrants,
  deviceCodeGrant,
  spEntityId,
  acsUrl,
  nameIdFormat,
  errors,
  canSubmit,
  onClientIdChange,
  onNameChange,
  onEnabledChange,
  onAuthenticationChange,
  onDirectAccessGrantsChange,
  onDeviceCodeGrantChange,
  onSpEntityIdChange,
  onAcsUrlChange,
  onNameIdFormatChange,
  onBack,
  onSubmit,
}: PageCreateClientProps) {
  const { t } = useTranslation('client')
  const nameIdDescription = NAME_ID_FORMAT_OPTIONS.find((o) => o.value === nameIdFormat)?.description

  return (
    <PageShell>
      <Button variant='ghost' size='sm' className='-ml-2 mb-2 text-neutral-500 dark:text-neutral-400' asChild>
        <Link to={listUrl}>
          <ArrowLeft className='size-3.5' />
          {t('create.back')}
        </Link>
      </Button>

      <div className='flex flex-wrap items-center gap-2 pb-3'>
        <h1 className={tokens.header.title}>{t('create.title')}</h1>
        <Pill tone='primary' mono>
          {protocol}
        </Pill>
        <Link
          to={pickerUrl}
          className='text-xs text-neutral-500 dark:text-neutral-400 underline-offset-2 hover:text-fk-primary-text hover:underline'
        >
          {t('create.change_protocol')}
        </Link>
      </div>

      <div className={tokens.page.blockGap}>
        <Section title={t('create.details.title')} description={t('create.details.description')}>
          <FieldRow
            label={t('create.details.client_id.label')}
            description={t('create.details.client_id.description')}
            htmlFor='new-client-id'
          >
            <Input
              id='new-client-id'
              value={clientId}
              onChange={(e) => onClientIdChange(e.target.value)}
              placeholder={t('create.details.client_id.placeholder')}
              className='max-w-sm'
              aria-invalid={Boolean(errors.clientId)}
            />
            {errors.clientId && <p className='mt-1.5 text-xs text-fk-danger'>{errors.clientId}</p>}
          </FieldRow>

          <FieldRow
            label={t('create.details.name.label')}
            description={t('create.details.name.description')}
            htmlFor='new-client-name'
          >
            <Input
              id='new-client-name'
              value={name}
              onChange={(e) => onNameChange(e.target.value)}
              className='max-w-sm'
              aria-invalid={Boolean(errors.name)}
            />
            {errors.name && <p className='mt-1.5 text-fk-danger text-xs'>{errors.name}</p>}
          </FieldRow>

          <FieldRow
            label={t('create.details.enabled.label')}
            description={t('create.details.enabled.description')}
          >
            <ChoiceCards
              label={t('create.details.state_picker')}
              value={clientStateOf(enabled)}
              onChange={(v: ClientState) => onEnabledChange(isEnabledState(v))}
              options={stateChoices(t)}
            />
          </FieldRow>

          <FieldRow
            label={t('create.details.authentication.label')}
            description={t('create.details.authentication.description')}
          >
            <ChoiceCards
              label={t('create.details.authentication.label')}
              value={authentication}
              onChange={onAuthenticationChange}
              options={authenticationChoices(t)}
            />
          </FieldRow>
        </Section>

        {protocol === 'openid-connect' ? (
          <Section
            title={t('create.capability.title')}
            description={t('create.capability.description')}
          >
            <FieldRow
              label={t('create.capability.direct_access_grants.label')}
              description={t('create.capability.direct_access_grants.description')}
            >
              <SwitchField checked={directAccessGrants} onCheckedChange={onDirectAccessGrantsChange} />
            </FieldRow>

            <FieldRow
              label={t('create.capability.device_code_grant.label')}
              description={t('create.capability.device_code_grant.description')}
            >
              <SwitchField checked={deviceCodeGrant} onCheckedChange={onDeviceCodeGrantChange} />
            </FieldRow>
          </Section>
        ) : (
          <Section title={t('create.saml.title')} description={t('create.saml.description')}>
            <FieldRow
              label={t('create.saml.entity_id.label')}
              description={t('create.saml.entity_id.description')}
              htmlFor='new-client-sp-entity-id'
            >
              <Input
                id='new-client-sp-entity-id'
                value={spEntityId}
                onChange={(e) => onSpEntityIdChange(e.target.value)}
                placeholder={t('create.saml.entity_id.placeholder')}
                className='max-w-lg'
                aria-invalid={Boolean(errors.spEntityId)}
              />
              {errors.spEntityId && (
                <p className='mt-1.5 text-xs text-fk-danger'>{errors.spEntityId}</p>
              )}
            </FieldRow>

            <FieldRow
              label={t('create.saml.acs_url.label')}
              description={t('create.saml.acs_url.description')}
              htmlFor='new-client-acs-url'
            >
              <Input
                id='new-client-acs-url'
                value={acsUrl}
                onChange={(e) => onAcsUrlChange(e.target.value)}
                placeholder={t('create.saml.acs_url.placeholder')}
                className='max-w-lg'
                aria-invalid={Boolean(errors.acsUrl)}
              />
              {errors.acsUrl && <p className='mt-1.5 text-xs text-fk-danger'>{errors.acsUrl}</p>}
            </FieldRow>

            <FieldRow
              label={t('create.saml.name_id_format.label')}
              description={t('create.saml.name_id_format.description')}
            >
              <div className='max-w-lg'>
                <Select value={nameIdFormat} onValueChange={onNameIdFormatChange}>
                  <SelectTrigger className='w-full'>
                    <SelectValue placeholder={t('create.saml.name_id_format.placeholder')} />
                  </SelectTrigger>
                  <SelectContent>
                    {NAME_ID_FORMAT_OPTIONS.map((option) => (
                      <SelectItem key={option.value} value={option.value}>
                        {option.label}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
                {nameIdDescription && (
                  <p className='mt-1.5 text-xs text-neutral-500 dark:text-neutral-400'>{nameIdDescription}</p>
                )}
              </div>
            </FieldRow>
          </Section>
        )}
      </div>

      <SaveBar
        show={canSubmit}
        title={t('create.save.title')}
        description={
          protocol === 'saml'
            ? t('create.save.description_saml')
            : t('create.save.description_oidc')
        }
        onCancel={onBack}
        actions={[{ label: t('create.save.submit'), onClick: onSubmit }]}
      />
    </PageShell>
  )
}
