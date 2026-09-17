import { Mail, User } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { Input } from '@/components/ui/input'
import { FieldRow, OrderedChoiceCards, Section, SwitchField } from '@/components/kit'
import type { Choice } from '@/components/kit'
import { Schemas } from '@/api/api.client'
import { REALM_NAMESPACE } from '../realm-namespace'

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

const ALIAS_CHOICES = [
  { value: 'username', labelKey: 'login.alias.username', icon: User },
  { value: 'email', labelKey: 'login.alias.email', icon: Mail },
] as const

const CAPABILITIES: {
  key: string
  labelKey: string
  read: (draft: LoginDraft) => boolean
  patch: (v: boolean) => Partial<LoginDraft>
}[] = [
  {
    key: 'user-registration',
    labelKey: 'login.capabilities.user_registration',
    read: (d) => d.userRegistration,
    patch: (v) => ({ userRegistration: v }),
  },
  {
    key: 'email-verification',
    labelKey: 'login.capabilities.email_verification',
    read: (d) => d.emailVerification,
    patch: (v) => ({ emailVerification: v }),
  },
  {
    key: 'forgot-password',
    labelKey: 'login.capabilities.forgot_password',
    read: (d) => d.forgotPassword,
    patch: (v) => ({ forgotPassword: v }),
  },
  {
    key: 'remember-me',
    labelKey: 'login.capabilities.remember_me',
    read: (d) => d.rememberMe,
    patch: (v) => ({ rememberMe: v }),
  },
  {
    key: 'passkey',
    labelKey: 'login.capabilities.passkey',
    read: (d) => d.passkey,
    patch: (v) => ({ passkey: v }),
  },
  {
    key: 'magic-link',
    labelKey: 'login.capabilities.magic_link',
    read: (d) => d.magicLink,
    patch: (v) => ({ magicLink: v }),
  },
]

export default function RealmLoginTab({
  value,
  aliasesError,
  onChange,
}: RealmLoginTabProps) {
  const { t } = useTranslation(REALM_NAMESPACE)

  const aliasChoices: Choice<LoginAlias>[] = ALIAS_CHOICES.map((choice) => ({
    value: choice.value,
    label: t(`${choice.labelKey}.label`),
    description: t(`${choice.labelKey}.description`),
    icon: choice.icon,
  }))

  return (
    <>
      <Section
        title={t('login.identifiers.title')}
        description={t('login.identifiers.description')}
      >
        <FieldRow
          label={t('login.identifiers.field.label')}
          description={t('login.identifiers.field.description')}
        >
          <OrderedChoiceCards
            label={t('login.identifiers.picker_label')}
            value={value.loginAliases}
            onChange={(next) => onChange({ loginAliases: next })}
            options={aliasChoices}
            minSelectedReason={t('login.identifiers.min_selected')}
          />
          {aliasesError && (
            <p className='mt-1.5 text-xs text-fk-danger'>{aliasesError}</p>
          )}
        </FieldRow>
      </Section>

      <Section
        title={t('login.capabilities.title')}
        description={t('login.capabilities.description')}
      >
        {CAPABILITIES.map((capability) => (
          <FieldRow
            key={capability.key}
            label={t(`${capability.labelKey}.label`)}
            description={t(`${capability.labelKey}.description`)}
          >
            <SwitchField
              checked={capability.read(value)}
              onCheckedChange={(v) => onChange(capability.patch(v))}
            />
          </FieldRow>
        ))}

        {value.magicLink && (
          <FieldRow
            label={t('login.magic_link_ttl.label')}
            description={t('login.magic_link_ttl.description')}
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
              <span className='text-xs text-neutral-500 dark:text-neutral-400'>
                {t('login.magic_link_ttl.unit', { count: value.magicLinkTtl })}
              </span>
            </div>
          </FieldRow>
        )}
      </Section>
    </>
  )
}
