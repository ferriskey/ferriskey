import { ArrowLeft } from 'lucide-react'
import { Link } from 'react-router-dom'
import { Button } from '@/components/kit/button'
import { Input } from '@/components/ui/input'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import FloatingActionBar from '@/components/ui/floating-action-bar'
import { ChoiceCards, FieldRow, Pill, Section, SwitchField } from '@/components/kit'
import { NAME_ID_FORMAT_OPTIONS } from '@/lib/saml'
import { cn } from '@/lib/utils'
import { tokens } from '@/styles/style-tokens'
import {
  authenticationChoices,
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
  const nameIdDescription = NAME_ID_FORMAT_OPTIONS.find((o) => o.value === nameIdFormat)?.description

  return (
    <div className={cn('mx-auto', tokens.page.maxWidth, tokens.page.padding)}>
      <Button variant='ghost' size='sm' className='-ml-2 mb-2 text-neutral-500' asChild>
        <Link to={listUrl}>
          <ArrowLeft className='size-3.5' />
          Clients
        </Link>
      </Button>

      <div className='flex flex-wrap items-center gap-2 pb-3'>
        <h1 className={tokens.header.title}>New client</h1>
        <Pill tone='primary' mono>
          {protocol}
        </Pill>
        <Link
          to={pickerUrl}
          className='text-xs text-neutral-500 underline-offset-2 hover:text-fk-primary-text hover:underline'
        >
          change protocol
        </Link>
      </div>

      <div className={tokens.page.blockGap}>
        <Section title='Client details' description='How this client is identified in the realm.'>
          <FieldRow
            label='Client ID'
            description='Unique identifier for this client. Applications send it on every request.'
            htmlFor='new-client-id'
          >
            <Input
              id='new-client-id'
              value={clientId}
              onChange={(e) => onClientIdChange(e.target.value)}
              placeholder='my-application'
              className='max-w-sm font-mono-ui'
              aria-invalid={Boolean(errors.clientId)}
            />
            {errors.clientId && <p className='mt-1.5 text-xs text-fk-danger'>{errors.clientId}</p>}
          </FieldRow>

          <FieldRow
            label='Name'
            description='Display name shown in the UI.'
            htmlFor='new-client-name'
          >
            <Input
              id='new-client-name'
              value={name}
              onChange={(e) => onNameChange(e.target.value)}
              className='max-w-sm'
              aria-invalid={Boolean(errors.name)}
            />
            {errors.name && <p className='mt-1.5 text-xs text-fk-danger'>{errors.name}</p>}
          </FieldRow>

          <FieldRow
            label='Client enabled'
            description='Disabled clients cannot authenticate users.'
          >
            <ChoiceCards
              label='Client state'
              value={enabled ? 'enabled' : 'disabled'}
              onChange={(v: ClientState) => onEnabledChange(v === 'enabled')}
              options={stateChoices}
            />
          </FieldRow>

          <FieldRow
            label='Client authentication'
            description='Set at creation and final: a client cannot move between confidential and public afterwards.'
          >
            <ChoiceCards
              label='Client authentication'
              value={authentication}
              onChange={onAuthenticationChange}
              options={authenticationChoices}
            />
          </FieldRow>
        </Section>

        {protocol === 'openid-connect' ? (
          <Section
            title='Capability config'
            description='OAuth flows this client is allowed to take.'
          >
            <FieldRow
              label='Direct access grants'
              description='Allows exchanging user credentials directly for tokens. Use only for trusted clients.'
            >
              <SwitchField checked={directAccessGrants} onCheckedChange={onDirectAccessGrantsChange} />
            </FieldRow>

            <FieldRow
              label='OAuth 2.0 device authorization grant'
              description='Lets browserless clients (CLI, IoT, TVs) initiate a device flow against this client. Disable unless this client really needs it.'
            >
              <SwitchField checked={deviceCodeGrant} onCheckedChange={onDeviceCodeGrantChange} />
            </FieldRow>
          </Section>
        ) : (
          <Section
            title='SAML'
            description='The two values the application shows on its SAML settings page, plus the identifier format.'
          >
            <FieldRow
              label='Entity ID'
              description='The unique identifier the application publishes for itself, for example https://chat.acme.com/saml/sp/1.'
              htmlFor='new-client-sp-entity-id'
            >
              <Input
                id='new-client-sp-entity-id'
                value={spEntityId}
                onChange={(e) => onSpEntityIdChange(e.target.value)}
                placeholder='https://chat.acme.com/saml/sp/1'
                className='max-w-lg font-mono-ui'
                aria-invalid={Boolean(errors.spEntityId)}
              />
              {errors.spEntityId && (
                <p className='mt-1.5 text-xs text-fk-danger'>{errors.spEntityId}</p>
              )}
            </FieldRow>

            <FieldRow
              label='Assertion Consumer Service URL'
              description='Where the signed assertion is posted after the user signs in.'
              htmlFor='new-client-acs-url'
            >
              <Input
                id='new-client-acs-url'
                value={acsUrl}
                onChange={(e) => onAcsUrlChange(e.target.value)}
                placeholder='https://chat.acme.com/saml/acs'
                className='max-w-lg font-mono-ui'
                aria-invalid={Boolean(errors.acsUrl)}
              />
              {errors.acsUrl && <p className='mt-1.5 text-xs text-fk-danger'>{errors.acsUrl}</p>}
            </FieldRow>

            <FieldRow
              label='Name ID format'
              description='How the user is identified inside the assertion.'
            >
              <div className='max-w-lg'>
                <Select value={nameIdFormat} onValueChange={onNameIdFormatChange}>
                  <SelectTrigger className='w-full'>
                    <SelectValue placeholder='Select a name ID format' />
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
                  <p className='mt-1.5 text-xs text-neutral-500'>{nameIdDescription}</p>
                )}
              </div>
            </FieldRow>
          </Section>
        )}
      </div>

      <FloatingActionBar
        show={canSubmit}
        title='Create client'
        description={
          protocol === 'saml'
            ? 'The client is created, then registered as a SAML service provider.'
            : 'The client is created with the settings above.'
        }
        onCancel={onBack}
        actions={[{ label: 'Create client', onClick: onSubmit }]}
      />
    </div>
  )
}
