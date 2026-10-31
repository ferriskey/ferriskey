import { useTranslation } from 'react-i18next'
import { Button } from '@/components/kit/button'
import { FieldRow, Section, Segmented } from '@/components/kit'
import { Input } from '@/components/ui/input'
import { Textarea } from '@/components/ui/textarea'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import {
  CONFIDENTIAL_AUTH_METHODS,
  type ClientAuthenticationDraft,
  type ConfidentialAuthMethod,
  type KeySource,
} from '../client-authentication'

const JWKS_URI_PLACEHOLDER = 'https://app.example.com/.well-known/jwks.json'
const JWKS_PLACEHOLDER = '{ "keys": [ … ] }'

export interface ClientAuthenticationSectionProps {
  draft: ClientAuthenticationDraft
  error?: string
  dirty: boolean
  saving: boolean
  onChange: (patch: Partial<ClientAuthenticationDraft>) => void
  onDiscard: () => void
  onSave: () => void
}

export default function ClientAuthenticationSection({
  draft,
  error,
  dirty,
  saving,
  onChange,
  onDiscard,
  onSave,
}: ClientAuthenticationSectionProps) {
  const { t } = useTranslation('client')

  return (
    <Section
      title={t('credentials.authentication.title')}
      description={t('credentials.authentication.description')}
    >
      <FieldRow
        label={t('credentials.authentication.method.label')}
        description={t(`credentials.authentication.method.hint.${draft.method}`)}
        htmlFor='client-auth-method'
      >
        <div className='max-w-lg'>
          <Select
            value={draft.method}
            onValueChange={(value) => onChange({ method: value as ConfidentialAuthMethod })}
          >
            <SelectTrigger id='client-auth-method' className='w-full'>
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              {CONFIDENTIAL_AUTH_METHODS.map((method) => (
                <SelectItem key={method} value={method}>
                  {t(`credentials.authentication.method.options.${method}`)}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>
      </FieldRow>

      {draft.method === 'private_key_jwt' && (
        <FieldRow
          label={t('credentials.authentication.keys.label')}
          description={t('credentials.authentication.keys.description')}
          htmlFor='client-auth-keys'
        >
          <div className='max-w-lg space-y-2'>
            <Segmented
              items={[
                { key: 'jwks_uri', label: t('credentials.authentication.keys.uri') },
                { key: 'jwks', label: t('credentials.authentication.keys.inline') },
              ]}
              value={draft.keySource}
              onChange={(key) => onChange({ keySource: key as KeySource })}
            />
            {draft.keySource === 'jwks_uri' ? (
              <Input
                id='client-auth-keys'
                value={draft.jwksUri}
                placeholder={JWKS_URI_PLACEHOLDER}
                onChange={(e) => onChange({ jwksUri: e.target.value })}
              />
            ) : (
              <Textarea
                id='client-auth-keys'
                value={draft.jwks}
                placeholder={JWKS_PLACEHOLDER}
                className='font-mono-ui text-xs'
                rows={8}
                onChange={(e) => onChange({ jwks: e.target.value })}
              />
            )}
          </div>
        </FieldRow>
      )}

      {error && <p className='pb-2 text-xs text-fk-danger'>{error}</p>}

      {dirty && (
        <div className='flex justify-end gap-2 py-3'>
          <Button type='button' variant='outline' size='sm' onClick={onDiscard}>
            {t('shared.discard')}
          </Button>
          <Button type='button' size='sm' disabled={saving || Boolean(error)} onClick={onSave}>
            {t('shared.save')}
          </Button>
        </div>
      )}
    </Section>
  )
}
