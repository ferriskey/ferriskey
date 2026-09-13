import { ArrowLeft } from 'lucide-react'
import { Link } from 'react-router-dom'
import { Button } from '@/components/kit/button'
import { Input } from '@/components/ui/input'
import SaveBar from '@/components/kit/save-bar'
import { ChipInput, FieldRow, PageShell, Pill, Section } from '@/components/kit'
import { tokens } from '@/styles/style-tokens'
import {
  APPLICATION_FIELDS,
  applicationTypeMeta,
  type ApplicationType,
} from '../application-types'

export interface CreateApplicationErrors {
  name?: string
  clientId?: string
  callbacks?: string
  origins?: string
}

export interface PageCreateApplicationProps {
  type: ApplicationType
  listUrl: string
  pickerUrl: string
  name: string
  clientId: string
  callbacks: string[]
  origins: string[]
  errors: CreateApplicationErrors
  canSubmit: boolean
  onNameChange: (value: string) => void
  onClientIdChange: (value: string) => void
  onCallbacksChange: (next: string[]) => void
  onOriginsChange: (next: string[]) => void
  onCancel: () => void
  onSubmit: () => void
}

export default function PageCreateApplication({
  type,
  listUrl,
  pickerUrl,
  name,
  clientId,
  callbacks,
  origins,
  errors,
  canSubmit,
  onNameChange,
  onClientIdChange,
  onCallbacksChange,
  onOriginsChange,
  onCancel,
  onSubmit,
}: PageCreateApplicationProps) {
  const meta = applicationTypeMeta(type)
  const fields = APPLICATION_FIELDS[type]

  return (
    <PageShell>
      <Button
        variant='ghost'
        size='sm'
        className='-ml-2 mb-2 text-neutral-500 dark:text-neutral-400'
        asChild
      >
        <Link to={listUrl}>
          <ArrowLeft className='size-3.5' />
          Applications
        </Link>
      </Button>

      <div className='flex flex-wrap items-center gap-2 pb-3'>
        <h1 className={tokens.header.title}>New application</h1>
        <Pill tone={meta.tone}>{meta.label}</Pill>
        <Link
          to={pickerUrl}
          className='text-xs text-neutral-500 underline-offset-2 hover:text-fk-primary-text hover:underline dark:text-neutral-400'
        >
          change type
        </Link>
      </div>

      <div className={tokens.page.blockGap}>
        <Section
          title='What this type gives you'
          description='Chosen at creation. It decides the sign-in flow and whether the application gets a secret.'
        >
          <FieldRow label='Sign-in flow' description={meta.description}>
            <p className='font-mono-ui text-xs text-neutral-700 dark:text-neutral-300'>
              {meta.flow}
            </p>
          </FieldRow>

          <FieldRow
            label='Client secret'
            description={
              meta.holdsSecret
                ? 'A secret is generated when the application is created. Keep it on your server: anyone holding it can sign in as this application.'
                : 'No secret is generated. This application ships to the user, so a secret could be read out of it — the sign-in is protected by PKCE instead.'
            }
          >
            <Pill tone={meta.holdsSecret ? 'violet' : 'info'}>
              {meta.holdsSecret ? 'Generated' : 'None'}
            </Pill>
          </FieldRow>
        </Section>

        <Section title='Identity' description='How this application is named and identified.'>
          <FieldRow
            label='Application name'
            description='Shown to your users on the sign-in page.'
            htmlFor='new-application-name'
          >
            <Input
              id='new-application-name'
              value={name}
              onChange={(e) => onNameChange(e.target.value)}
              placeholder='Acme Mobile'
              className='max-w-sm'
              aria-invalid={Boolean(errors.name)}
            />
            {errors.name && <p className='mt-1.5 text-xs text-fk-danger'>{errors.name}</p>}
          </FieldRow>

          <FieldRow
            label='Client ID'
            description='Sent on every OAuth request. Derived from the name; change it before creating, it is what your code will hardcode.'
            htmlFor='new-application-client-id'
          >
            <Input
              id='new-application-client-id'
              value={clientId}
              onChange={(e) => onClientIdChange(e.target.value)}
              placeholder='acme-mobile'
              className='max-w-sm font-mono-ui text-xs'
              aria-invalid={Boolean(errors.clientId)}
            />
            {errors.clientId && (
              <p className='mt-1.5 text-xs text-fk-danger'>{errors.clientId}</p>
            )}
          </FieldRow>
        </Section>

        {(fields.showCallbacks || fields.showOrigins) && (
          <Section
            title='Where users come back'
            description='Addresses FerrisKey is allowed to send the user to once they have signed in.'
          >
            {fields.showCallbacks && (
              <FieldRow label={fields.callbacksLabel} description={fields.callbacksHint}>
                <ChipInput
                  values={callbacks}
                  onChange={onCallbacksChange}
                  placeholder={fields.callbacksPlaceholder}
                  emptyHint={
                    fields.callbackRequired
                      ? 'No callback URL — sign-in cannot complete.'
                      : 'No callback URL registered.'
                  }
                />
                {errors.callbacks && (
                  <p className='mt-1.5 text-xs text-fk-danger'>{errors.callbacks}</p>
                )}
              </FieldRow>
            )}

            {fields.showOrigins && (
              <FieldRow label={fields.originsLabel} description={fields.originsHint}>
                <ChipInput
                  values={origins}
                  onChange={onOriginsChange}
                  placeholder={fields.originsPlaceholder}
                  emptyHint='No origin — browser calls from another host will be refused.'
                />
                {errors.origins && (
                  <p className='mt-1.5 text-xs text-fk-danger'>{errors.origins}</p>
                )}
              </FieldRow>
            )}
          </Section>
        )}

        {type === 'm2m' && (
          <Section
            title='No callback URL needed'
            description='Machine-to-machine applications never send a user through a browser.'
          >
            <FieldRow
              label='How it gets a token'
              description='It posts its client ID and secret to the token endpoint with the client_credentials grant. The secret is on the Credentials tab once the application exists.'
            >
              <p className='font-mono-ui text-xs text-neutral-700 dark:text-neutral-300'>
                grant_type=client_credentials
              </p>
            </FieldRow>
          </Section>
        )}

        {type === 'device' && (
          <Section
            title='No callback URL needed'
            description='The device flow never redirects: the user opens a verification page on another screen.'
          >
            <FieldRow
              label='How it gets a token'
              description='The device asks for a user code, shows it, and polls the token endpoint until the user has approved it.'
            >
              <p className='font-mono-ui text-xs text-neutral-700 dark:text-neutral-300'>
                grant_type=urn:ietf:params:oauth:grant-type:device_code
              </p>
            </FieldRow>
          </Section>
        )}
      </div>

      <SaveBar
        show={canSubmit}
        title='Create application'
        description='The application is created, then its callback URLs and origins are registered.'
        onCancel={onCancel}
        cancelLabel='Cancel'
        actions={[{ label: 'Create application', onClick: onSubmit }]}
      />
    </PageShell>
  )
}
