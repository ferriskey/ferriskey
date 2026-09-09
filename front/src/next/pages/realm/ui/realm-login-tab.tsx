import { Mail, User } from 'lucide-react'
import { Input } from '@/components/ui/input'
import { FieldRow, OrderedChoiceCards, Section, SwitchField } from '@/components/kit'
import type { Choice } from '@/components/kit'
import { Schemas } from '@/api/api.client'

import LoginAlias = Schemas.LoginAlias

export interface LoginDraft {
  userRegistration: boolean
  emailVerification: boolean
  forgotPassword: boolean
  rememberMe: boolean
  passkey: boolean
  magicLink: boolean
  magicLinkTtl: number
  loginAliases: LoginAlias[]
}

export interface RealmLoginTabProps {
  value: LoginDraft
  aliasesError?: string
  onChange: (patch: Partial<LoginDraft>) => void
}

const ALIAS_CHOICES: Choice<LoginAlias>[] = [
  {
    value: 'username',
    label: 'Username',
    description: 'The account username.',
    icon: User,
  },
  {
    value: 'email',
    label: 'Email',
    description: 'The verified email address of the account.',
    icon: Mail,
  },
]

const CAPABILITIES: {
  key: string
  label: string
  description: string
  read: (draft: LoginDraft) => boolean
  patch: (v: boolean) => Partial<LoginDraft>
}[] = [
  {
    key: 'user-registration',
    label: 'User Registration',
    description: 'Allow users to register themselves through the login page.',
    read: (d) => d.userRegistration,
    patch: (v) => ({ userRegistration: v }),
  },
  {
    key: 'email-verification',
    label: 'Email Verification',
    description:
      'Require users to verify their email address before they can sign in.',
    read: (d) => d.emailVerification,
    patch: (v) => ({ emailVerification: v }),
  },
  {
    key: 'forgot-password',
    label: 'Forgot Password',
    description: 'Show a forgot password link on the login page.',
    read: (d) => d.forgotPassword,
    patch: (v) => ({ forgotPassword: v }),
  },
  {
    key: 'remember-me',
    label: 'Remember Me',
    description: 'Show a remember me checkbox on the login page.',
    read: (d) => d.rememberMe,
    patch: (v) => ({ rememberMe: v }),
  },
  {
    key: 'passkey',
    label: 'Passkey Authentication',
    description: 'Allow users to sign in with a passkey instead of a password.',
    read: (d) => d.passkey,
    patch: (v) => ({ passkey: v }),
  },
  {
    key: 'magic-link',
    label: 'Magic Link',
    description: 'Allow users to sign in via a magic link sent by email.',
    read: (d) => d.magicLink,
    patch: (v) => ({ magicLink: v }),
  },
]

export default function RealmLoginTab({
  value,
  aliasesError,
  onChange,
}: RealmLoginTabProps) {
  return (
    <>
      <Section
        title='Login identifiers'
        description='What users type to authenticate.'
      >
        <FieldRow
          label='Login identifiers'
          description='Which identifiers users may sign in with. Order sets precedence.'
        >
          <OrderedChoiceCards
            label='Login identifiers'
            value={value.loginAliases}
            onChange={(next) => onChange({ loginAliases: next })}
            options={ALIAS_CHOICES}
            minSelectedReason='Select at least one login identifier'
          />
          {aliasesError && (
            <p className='mt-1.5 text-xs text-fk-danger'>{aliasesError}</p>
          )}
        </FieldRow>
      </Section>

      <Section
        title='Authentication capabilities'
        description='What users can do from the login screen.'
      >
        {CAPABILITIES.map((capability) => (
          <FieldRow
            key={capability.key}
            label={capability.label}
            description={capability.description}
          >
            <SwitchField
              checked={capability.read(value)}
              onCheckedChange={(v) => onChange(capability.patch(v))}
            />
          </FieldRow>
        ))}

        {value.magicLink && (
          <FieldRow
            label='Magic Link TTL'
            description='How long a magic link remains valid, in minutes.'
            htmlFor='realm-magic-link-ttl'
          >
            <div className='flex max-w-[10rem] items-center gap-2'>
              <Input
                id='realm-magic-link-ttl'
                type='number'
                min={1}
                value={value.magicLinkTtl}
                onChange={(e) => onChange({ magicLinkTtl: Number(e.target.value) })}
                className='tnum'
              />
              <span className='text-xs text-neutral-500'>minutes</span>
            </div>
          </FieldRow>
        )}
      </Section>
    </>
  )
}
