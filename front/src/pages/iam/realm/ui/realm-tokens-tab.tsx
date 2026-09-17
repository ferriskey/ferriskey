import { useTranslation } from 'react-i18next'
import { DurationInput } from '@/components/ui/duration-input'
import { FieldRow, Section } from '@/components/kit'
import { REALM_NAMESPACE } from '../realm-namespace'

export interface TokensDraft {
  accessTokenLifetime: number
  refreshTokenLifetime: number
  idTokenLifetime: number
  temporaryTokenLifetime: number
}

export interface RealmTokensTabProps {
  value: TokensDraft
  onChange: (patch: Partial<TokensDraft>) => void
}

const LIFETIMES: {
  key: string
  labelKey: string
  read: (draft: TokensDraft) => number
  patch: (v: number) => Partial<TokensDraft>
}[] = [
  {
    key: 'access',
    labelKey: 'tokens.access',
    read: (d) => d.accessTokenLifetime,
    patch: (v) => ({ accessTokenLifetime: v }),
  },
  {
    key: 'refresh',
    labelKey: 'tokens.refresh',
    read: (d) => d.refreshTokenLifetime,
    patch: (v) => ({ refreshTokenLifetime: v }),
  },
  {
    key: 'id',
    labelKey: 'tokens.id',
    read: (d) => d.idTokenLifetime,
    patch: (v) => ({ idTokenLifetime: v }),
  },
  {
    key: 'temporary',
    labelKey: 'tokens.temporary',
    read: (d) => d.temporaryTokenLifetime,
    patch: (v) => ({ temporaryTokenLifetime: v }),
  },
]

export default function RealmTokensTab({ value, onChange }: RealmTokensTabProps) {
  const { t } = useTranslation(REALM_NAMESPACE)

  return (
    <Section title={t('tokens.title')} description={t('tokens.description')}>
      {LIFETIMES.map((lifetime) => (
        <FieldRow
          key={lifetime.key}
          label={t(`${lifetime.labelKey}.label`)}
          description={t(`${lifetime.labelKey}.description`)}
        >
          <div className='max-w-sm'>
            <DurationInput
              label={t(`${lifetime.labelKey}.label`)}
              value={lifetime.read(value)}
              onChange={(seconds) => onChange(lifetime.patch(seconds ?? 0))}
            />
          </div>
        </FieldRow>
      ))}
    </Section>
  )
}
