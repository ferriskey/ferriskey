import { DurationInput } from '@/components/ui/duration-input'
import { FieldRow, Section } from '@/components/kit'

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
  label: string
  description: string
  read: (draft: TokensDraft) => number
  patch: (v: number) => Partial<TokensDraft>
}[] = [
  {
    key: 'access',
    label: 'Access Token Lifetime',
    description: 'How long access tokens remain valid.',
    read: (d) => d.accessTokenLifetime,
    patch: (v) => ({ accessTokenLifetime: v }),
  },
  {
    key: 'refresh',
    label: 'Refresh Token Lifetime',
    description: 'How long refresh tokens remain valid.',
    read: (d) => d.refreshTokenLifetime,
    patch: (v) => ({ refreshTokenLifetime: v }),
  },
  {
    key: 'id',
    label: 'ID Token Lifetime',
    description: 'How long ID tokens remain valid.',
    read: (d) => d.idTokenLifetime,
    patch: (v) => ({ idTokenLifetime: v }),
  },
  {
    key: 'temporary',
    label: 'Temporary Token Lifetime',
    description:
      'How long temporary tokens remain valid — password reset, for example.',
    read: (d) => d.temporaryTokenLifetime,
    patch: (v) => ({ temporaryTokenLifetime: v }),
  },
]

export default function RealmTokensTab({ value, onChange }: RealmTokensTabProps) {
  return (
    <Section
      title='Default token lifetimes'
      description='Inherited by every client of this realm that does not override them.'
    >
      {LIFETIMES.map((lifetime) => (
        <FieldRow
          key={lifetime.key}
          label={lifetime.label}
          description={lifetime.description}
        >
          <div className='max-w-sm'>
            <DurationInput
              label={lifetime.label}
              value={lifetime.read(value)}
              onChange={(seconds) => onChange(lifetime.patch(seconds ?? 0))}
            />
          </div>
        </FieldRow>
      ))}
    </Section>
  )
}
