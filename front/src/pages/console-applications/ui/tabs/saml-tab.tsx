import { Schemas } from '@/api/api.client'
import { Switch } from '@/components/ui/switch'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { useSamlServiceProvider } from '@/hooks/use-saml-service-provider'
import {
  ATTRIBUTE_NAME_FORMAT_OPTIONS,
  BUILT_IN_SOURCE_OPTIONS,
  COMMON_PROFILE_MAPPERS,
  CUSTOM_ATTRIBUTE_SOURCE,
  DEFAULT_ATTRIBUTE_NAME_FORMAT,
  DEFAULT_NAME_ID_FORMAT,
  NAME_ID_FORMAT_OPTIONS,
  describeAttributeNameFormat,
  describeAttributeSource,
  toCustomAttributeSource,
} from '@/lib/saml'
import { Loader2, Plus, X } from 'lucide-react'
import { useEffect, useMemo, useState } from 'react'
import { Field, Section } from './primitives'

import ClientSamlConfig = Schemas.ClientSamlConfig

interface ServiceProviderValues {
  spEntityId: string
  acsUrl: string
  nameIdFormat: string
  signAssertions: boolean
  signDocuments: boolean
  wantAuthnRequestsSigned: boolean
}

const BLANK_SERVICE_PROVIDER: ServiceProviderValues = {
  spEntityId: '',
  acsUrl: '',
  nameIdFormat: DEFAULT_NAME_ID_FORMAT,
  signAssertions: true,
  signDocuments: false,
  wantAuthnRequestsSigned: false,
}

function valuesFromConfig(config: ClientSamlConfig | null): ServiceProviderValues {
  if (!config) return BLANK_SERVICE_PROVIDER
  return {
    spEntityId: config.sp_entity_id,
    acsUrl: config.acs_url,
    nameIdFormat: config.name_id_format,
    signAssertions: config.sign_assertions,
    signDocuments: config.sign_documents,
    wantAuthnRequestsSigned: config.want_authn_requests_signed,
  }
}

interface Props {
  realm: string
  clientId: string
}

export default function SamlTab({ realm, clientId }: Props) {
  const {
    config,
    mappers,
    isConfigured,
    isLoading,
    isSavingConfig,
    isCreatingMapper,
    isDeletingMapper,
    saveConfig,
    addAttributeMapper,
    deleteAttributeMapper,
    addCommonProfileMappers,
  } = useSamlServiceProvider(realm, clientId)

  const baseline = useMemo(() => valuesFromConfig(config), [config])
  const [values, setValues] = useState<ServiceProviderValues>(baseline)

  useEffect(() => {
    setValues(baseline)
  }, [baseline])

  const [mapperName, setMapperName] = useState('')
  const [mapperSource, setMapperSource] = useState<string>(BUILT_IN_SOURCE_OPTIONS[0].value)
  const [mapperCustomKey, setMapperCustomKey] = useState('')
  const [mapperNameFormat, setMapperNameFormat] = useState<string>(DEFAULT_ATTRIBUTE_NAME_FORMAT)

  const hasChanges = JSON.stringify(values) !== JSON.stringify(baseline)
  const canSave =
    values.spEntityId.trim().length > 0 &&
    values.acsUrl.trim().length > 0 &&
    (hasChanges || !isConfigured)

  const set = <K extends keyof ServiceProviderValues>(key: K, value: ServiceProviderValues[K]) =>
    setValues((prev) => ({ ...prev, [key]: value }))

  const resolvedSource =
    mapperSource === CUSTOM_ATTRIBUTE_SOURCE ? toCustomAttributeSource(mapperCustomKey) : mapperSource

  const canAddMapper =
    mapperName.trim().length > 0 &&
    (mapperSource !== CUSTOM_ATTRIBUTE_SOURCE || mapperCustomKey.trim().length > 0)

  const handleSubmit = async (event: React.FormEvent) => {
    event.preventDefault()
    if (!canSave || isSavingConfig) return

    await saveConfig({
      sp_entity_id: values.spEntityId.trim(),
      acs_url: values.acsUrl.trim(),
      name_id_format: values.nameIdFormat,
      sign_assertions: values.signAssertions,
      sign_documents: values.signDocuments,
      want_authn_requests_signed: values.wantAuthnRequestsSigned,
    })
  }

  const handleAddMapper = async () => {
    if (!canAddMapper || isCreatingMapper) return

    const added = await addAttributeMapper({
      name: mapperName,
      source: resolvedSource,
      nameFormat: mapperNameFormat,
    })

    if (added) {
      setMapperName('')
      setMapperCustomKey('')
    }
  }

  if (isLoading) {
    return (
      <div className='flex items-center gap-2 text-sm text-muted-foreground'>
        <Loader2 className='h-4 w-4 animate-spin' />
        Loading the SAML configuration…
      </div>
    )
  }

  return (
    <div className='flex flex-col gap-6'>
      {!isConfigured && (
        <div className='rounded-md border border-border bg-muted/40 p-4'>
          <p className='text-sm font-medium'>This application does not use SAML yet</p>
          <p className='text-xs text-muted-foreground mt-1'>
            Fill in the two values your application shows on its SAML settings page, then save.
            FerrisKey will start answering SAML sign-in requests for it.
          </p>
        </div>
      )}

      <form onSubmit={handleSubmit} className='flex flex-col gap-6'>
        <Section
          title='Service provider'
          description='Identifies the application and tells FerrisKey where to send the assertion.'
        >
          <Field
            label='Entity ID'
            hint='The unique identifier the application publishes for itself, for example https://chat.acme.com/saml/sp/1.'
          >
            <input
              type='text'
              value={values.spEntityId}
              onChange={(e) => set('spEntityId', e.target.value)}
              placeholder='https://chat.acme.com/saml/sp/1'
              className='w-full rounded-md border border-border bg-background px-3 py-2 text-sm font-mono outline-none placeholder:text-muted-foreground focus:border-primary/40 focus:ring-1 focus:ring-primary/30'
            />
          </Field>

          <Field
            label='Assertion Consumer Service URL'
            hint='Where the signed assertion is posted after the user signs in.'
          >
            <input
              type='text'
              value={values.acsUrl}
              onChange={(e) => set('acsUrl', e.target.value)}
              placeholder='https://chat.acme.com/omniauth/saml/callback?account_id=1'
              className='w-full rounded-md border border-border bg-background px-3 py-2 text-sm font-mono outline-none placeholder:text-muted-foreground focus:border-primary/40 focus:ring-1 focus:ring-primary/30'
            />
          </Field>

          <SelectRow
            label='Name ID format'
            hint='How the user is identified inside the assertion.'
            value={values.nameIdFormat}
            onChange={(value) => set('nameIdFormat', value)}
            options={NAME_ID_FORMAT_OPTIONS}
          />
        </Section>

        <Section title='Signatures' description='How assertions and requests are signed.'>
          <ToggleRow
            label='Sign assertions'
            description='Sign the assertion itself. Almost every application expects this.'
            checked={values.signAssertions}
            onChange={(v) => set('signAssertions', v)}
          />
          <ToggleRow
            label='Sign documents'
            description='Sign the whole SAML response in addition to the assertion.'
            checked={values.signDocuments}
            onChange={(v) => set('signDocuments', v)}
          />
          <ToggleRow
            label='Require signed authentication requests'
            description='Reject sign-in requests from this application unless they carry a valid signature.'
            checked={values.wantAuthnRequestsSigned}
            onChange={(v) => set('wantAuthnRequestsSigned', v)}
          />
        </Section>

        <div className='sticky bottom-0 -mx-8 md:-mx-12 px-8 md:px-12 py-4 border-t border-border bg-background/80 backdrop-blur flex items-center justify-end gap-3'>
          {hasChanges && isConfigured && (
            <button
              type='button'
              onClick={() => setValues(baseline)}
              className='rounded-md border border-border bg-background px-4 py-2 text-sm font-medium hover:bg-muted transition-colors'
            >
              Discard
            </button>
          )}
          <button
            type='submit'
            disabled={!canSave || isSavingConfig}
            className='inline-flex items-center gap-2 rounded-md bg-primary px-4 py-2 text-sm font-medium text-primary-foreground hover:bg-primary/90 transition-colors disabled:opacity-50 disabled:cursor-not-allowed'
          >
            {isSavingConfig && <Loader2 className='h-3.5 w-3.5 animate-spin' />}
            {isConfigured ? 'Save changes' : 'Enable SAML'}
          </button>
        </div>
      </form>

      {isConfigured && (
        <Section
          title='Attribute mappings'
          description='User details sent alongside the assertion. Match the attribute names the application asks for.'
        >
          <div className='flex flex-col gap-2'>
            {mappers.length === 0 && (
              <div className='flex flex-col gap-3 rounded-md border border-dashed border-border p-4'>
                <p className='text-xs text-muted-foreground'>
                  No attributes are sent yet. Most applications need at least an email address.
                </p>
                <button
                  type='button'
                  onClick={() => void addCommonProfileMappers(COMMON_PROFILE_MAPPERS)}
                  disabled={isCreatingMapper}
                  className='self-start inline-flex items-center gap-1.5 rounded-md border border-border bg-background px-3 py-2 text-sm font-medium hover:bg-muted transition-colors disabled:opacity-40'
                >
                  {isCreatingMapper ? (
                    <Loader2 className='h-3.5 w-3.5 animate-spin' />
                  ) : (
                    <Plus className='h-3.5 w-3.5' />
                  )}
                  Add email, first_name and last_name
                </button>
              </div>
            )}

            {mappers.map((mapper) => (
              <div
                key={mapper.id}
                className='flex items-center gap-3 rounded-md border border-border bg-background px-3 py-2'
              >
                <span className='text-sm font-mono truncate'>{mapper.name}</span>
                <span className='text-xs text-muted-foreground shrink-0'>from</span>
                <span className='flex-1 text-sm truncate'>
                  {describeAttributeSource(mapper.source)}
                </span>
                <span className='text-xs text-muted-foreground shrink-0'>
                  {describeAttributeNameFormat(mapper.name_format)}
                </span>
                <button
                  type='button'
                  onClick={() => void deleteAttributeMapper(mapper.id)}
                  disabled={isDeletingMapper}
                  className='inline-flex h-7 w-7 items-center justify-center rounded-md text-muted-foreground hover:text-red-500 hover:bg-muted transition-colors disabled:opacity-40 shrink-0'
                  aria-label={`Remove the ${mapper.name} attribute mapping`}
                >
                  <X className='h-3.5 w-3.5' />
                </button>
              </div>
            ))}

            <div className='mt-2 grid grid-cols-1 sm:grid-cols-2 gap-4 rounded-md border border-border p-4'>
              <Field label='Attribute name' hint='The name the application looks for, e.g. email.'>
                <input
                  type='text'
                  value={mapperName}
                  onChange={(e) => setMapperName(e.target.value)}
                  placeholder='email'
                  className='w-full rounded-md border border-border bg-background px-3 py-2 text-sm font-mono outline-none placeholder:text-muted-foreground focus:border-primary/40 focus:ring-1 focus:ring-primary/30'
                />
              </Field>

              <SelectRow
                label='User detail to send'
                value={mapperSource}
                onChange={setMapperSource}
                options={[
                  ...BUILT_IN_SOURCE_OPTIONS.map((option) => ({
                    value: option.value as string,
                    label: option.label,
                    description: '',
                  })),
                  { value: CUSTOM_ATTRIBUTE_SOURCE, label: 'Custom user attribute', description: '' },
                ]}
              />

              {mapperSource === CUSTOM_ATTRIBUTE_SOURCE && (
                <Field label='Attribute key' hint='The key stored on the user profile.'>
                  <input
                    type='text'
                    value={mapperCustomKey}
                    onChange={(e) => setMapperCustomKey(e.target.value)}
                    placeholder='department'
                    className='w-full rounded-md border border-border bg-background px-3 py-2 text-sm font-mono outline-none placeholder:text-muted-foreground focus:border-primary/40 focus:ring-1 focus:ring-primary/30'
                  />
                </Field>
              )}

              <SelectRow
                label='Name format'
                value={mapperNameFormat}
                onChange={setMapperNameFormat}
                options={ATTRIBUTE_NAME_FORMAT_OPTIONS}
              />

              <div className='sm:col-span-2 flex justify-end'>
                <button
                  type='button'
                  onClick={() => void handleAddMapper()}
                  disabled={!canAddMapper || isCreatingMapper}
                  className='inline-flex items-center gap-1.5 rounded-md border border-border bg-background px-3 py-2 text-sm font-medium hover:bg-muted transition-colors disabled:opacity-40 disabled:cursor-not-allowed'
                >
                  {isCreatingMapper ? (
                    <Loader2 className='h-3.5 w-3.5 animate-spin' />
                  ) : (
                    <Plus className='h-3.5 w-3.5' />
                  )}
                  Add attribute
                </button>
              </div>
            </div>
          </div>
        </Section>
      )}
    </div>
  )
}

interface SelectRowProps {
  label: string
  hint?: string
  value: string
  onChange: (value: string) => void
  options: { value: string; label: string; description: string }[]
}

function SelectRow({ label, hint, value, onChange, options }: SelectRowProps) {
  const selected = options.find((option) => option.value === value)

  return (
    <div className='flex flex-col gap-1.5'>
      <span className='text-sm font-medium'>{label}</span>
      <Select value={value} onValueChange={onChange}>
        <SelectTrigger aria-label={label}>
          <SelectValue placeholder={`Select ${label.toLowerCase()}`} />
        </SelectTrigger>
        <SelectContent>
          {options.map((option) => (
            <SelectItem key={option.value} value={option.value}>
              {option.label}
            </SelectItem>
          ))}
        </SelectContent>
      </Select>
      {(selected?.description || hint) && (
        <p className='text-xs text-muted-foreground'>{selected?.description || hint}</p>
      )}
    </div>
  )
}

interface ToggleRowProps {
  label: string
  description: string
  checked: boolean
  onChange: (checked: boolean) => void
}

function ToggleRow({ label, description, checked, onChange }: ToggleRowProps) {
  return (
    <div className='flex items-center justify-between gap-5 rounded-md border border-border p-3'>
      <div className='space-y-0.5'>
        <p className='text-sm font-medium'>{label}</p>
        <p className='text-xs text-muted-foreground'>{description}</p>
      </div>
      <Switch checked={checked} onCheckedChange={onChange} />
    </div>
  )
}
