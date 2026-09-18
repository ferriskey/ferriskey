import { useMemo } from 'react'
import { useNavigate, useParams } from 'react-router'
import { toast } from 'sonner'
import { useTranslation } from 'react-i18next'
import {
  useDeleteRealm,
  useGetLoginSettings,
  useGetRealm,
  useGetRealmPasswordPolicy,
  useUpdateRealm,
  useUpdateRealmPasswordPolicy,
  useUpdateRealmSettings,
} from '@/api/realm.api'
import { useGetUsers } from '@/api/user.api'
import { useGetRoles } from '@/api/role.api'
import {
  useAddRealmWhitelistEntry,
  useGetRealmWhitelist,
  useRemoveRealmWhitelistEntry,
} from '@/api/maintenance.api'
import { RouterParams } from '@/routes/router'
import { useRouteTabs } from '@/components/kit'
import { SigningAlgorithm } from '@/api/core.interface'
import {
  updatePasswordPolicyValidator,
  updateRealmValidator,
} from '@/pages/iam/realm/validators'
import { Schemas } from '@/api/api.client'
import { REALM_SETTINGS_URL } from '@/routes/router'
import PageRealmSettings from '../ui/page-realm-settings'
import type { LoginDraft } from '../ui/realm-login-tab'
import type { TokensDraft } from '../ui/realm-tokens-tab'
import type { PolicyDraft } from '../ui/realm-password-policy-tab'
import { REALM_NAMESPACE } from '../realm-namespace'
import { useDraft } from './use-draft'

import LoginAlias = Schemas.LoginAlias
import { apiErrorMessage } from '@/lib/api-error'

const REALM_TABS = [
  { key: 'general', labelKey: 'settings.tabs.general' },
  { key: 'login', labelKey: 'settings.tabs.login' },
  { key: 'tokens', labelKey: 'settings.tabs.tokens' },
  { key: 'password-policy', labelKey: 'settings.tabs.password_policy' },
  { key: 'maintenance', labelKey: 'settings.tabs.maintenance' },
] as const

const MASTER_REALM_OVERVIEW_URL = '/realms/master/overview'

type WhitelistKind = 'user' | 'role'

const USER_WHITELIST: WhitelistKind = 'user'
const ROLE_WHITELIST: WhitelistKind = 'role'

const DEFAULT_LOGIN: LoginDraft = {
  userRegistration: false,
  emailVerification: false,
  forgotPassword: false,
  rememberMe: false,
  passkey: false,
  magicLink: false,
  magicLinkTtl: 15,
  loginAliases: ['username'],
}

const DEFAULT_TOKENS: TokensDraft = {
  accessTokenLifetime: 300,
  refreshTokenLifetime: 86400,
  idTokenLifetime: 300,
  temporaryTokenLifetime: 300,
}

const DEFAULT_POLICY: PolicyDraft = {
  min_length: 8,
  require_uppercase: false,
  require_lowercase: false,
  require_number: false,
  require_special: false,
  max_age_days: 0,
  min_entropy_bits: 0,
  forbid_common: false,
  check_breached: false,
}

export default function PageRealmSettingsFeature() {
  const { realm_name } = useParams<RouterParams>()
  const navigate = useNavigate()
  const { t } = useTranslation(REALM_NAMESPACE)
  const realm = realm_name ?? 'master'

  const { data: realmData, isLoading: realmLoading } = useGetRealm({ realm })
  const { data: loginSettings } = useGetLoginSettings({ realm })
  const {
    data: policyData,
    isLoading: policyLoading,
    isError: policyError,
  } = useGetRealmPasswordPolicy({ realm })

  const { data: usersResponse } = useGetUsers({ realm })
  const { data: rolesResponse } = useGetRoles({ realm })
  const { data: whitelistResponse } = useGetRealmWhitelist({ realm })

  const { mutate: updateRealm } = useUpdateRealm()
  const { mutate: updateSettings } = useUpdateRealmSettings()
  const { mutate: updatePolicy } = useUpdateRealmPasswordPolicy()
  const { mutate: deleteRealm } = useDeleteRealm()
  const { mutate: addWhitelistEntry } = useAddRealmWhitelistEntry()
  const { mutate: removeWhitelistEntry } = useRemoveRealmWhitelistEntry()

  const translatedTabs = useMemo(
    () => REALM_TABS.map((item) => ({ key: item.key, label: t(item.labelKey) })),
    [t]
  )

  const { value: tab, tabs } = useRouteTabs(REALM_SETTINGS_URL(realm), translatedTabs)

  const settings = realmData?.settings

  const pristineGeneral = { displayName: realmData?.display_name ?? '' }
  const general = useDraft(
    `${realmData?.id ?? ''}:${realmData?.updated_at ?? ''}`,
    pristineGeneral
  )

  const pristineLogin: LoginDraft = loginSettings
    ? {
        userRegistration: loginSettings.user_registration_enabled,
        emailVerification: loginSettings.email_verification_enabled,
        forgotPassword: loginSettings.forgot_password_enabled,
        rememberMe: loginSettings.remember_me_enabled,
        passkey: loginSettings.passkey_enabled,
        magicLink: loginSettings.magic_link_enabled,
        magicLinkTtl: loginSettings.magic_link_ttl,
        loginAliases:
          loginSettings.login_aliases.length > 0
            ? loginSettings.login_aliases
            : (['username'] as LoginAlias[]),
      }
    : DEFAULT_LOGIN
  const login = useDraft(
    loginSettings ? `${loginSettings.name}:${settings?.updated_at ?? ''}` : '',
    pristineLogin
  )

  const pristineTokens: TokensDraft = settings
    ? {
        accessTokenLifetime: settings.access_token_lifetime,
        refreshTokenLifetime: settings.refresh_token_lifetime,
        idTokenLifetime: settings.id_token_lifetime,
        temporaryTokenLifetime: settings.temporary_token_lifetime,
      }
    : DEFAULT_TOKENS
  const tokensDraft = useDraft(
    settings ? `${settings.id}:${settings.updated_at}` : '',
    pristineTokens
  )

  const pristinePolicy: PolicyDraft = policyData
    ? {
        min_length: policyData.min_length,
        require_uppercase: policyData.require_uppercase,
        require_lowercase: policyData.require_lowercase,
        require_number: policyData.require_number,
        require_special: policyData.require_special,
        max_age_days: policyData.max_age_days ?? 0,
        min_entropy_bits: policyData.min_entropy_bits,
        forbid_common: policyData.forbid_common,
        check_breached: policyData.check_breached,
      }
    : DEFAULT_POLICY
  const policy = useDraft(
    policyData ? `${policyData.id}:${policyData.updated_at}` : '',
    pristinePolicy
  )

  const whitelist = useMemo(() => whitelistResponse?.data ?? [], [whitelistResponse])
  const users = useMemo(() => usersResponse?.data ?? [], [usersResponse])
  const roles = useMemo(() => rolesResponse?.data ?? [], [rolesResponse])

  const whitelistedUserIds = whitelist
    .filter((entry) => entry.user_id)
    .map((entry) => entry.user_id as string)
  const whitelistedRoleIds = whitelist
    .filter((entry) => entry.role_id)
    .map((entry) => entry.role_id as string)

  const entryIdFor = (kind: WhitelistKind, id: string) =>
    whitelist.find((entry) => (kind === USER_WHITELIST ? entry.user_id : entry.role_id) === id)
      ?.id

  const generalParsed = updateRealmValidator.safeParse({
    name: realmData?.name ?? '',
    display_name: general.value.displayName,
    default_signing_algorithm: SigningAlgorithm.RS256,
  })
  const displayNameError = generalParsed.success
    ? undefined
    : generalParsed.error.issues.find((issue) => issue.path[0] === 'display_name')?.message

  const loginAliasesError =
    login.value.loginAliases.length === 0
      ? t('login.identifiers.min_selected')
      : undefined

  const policyParsed = updatePasswordPolicyValidator.safeParse(policy.value)
  const policyErrors: Partial<Record<keyof PolicyDraft, string>> = {}
  if (!policyParsed.success) {
    for (const issue of policyParsed.error.issues) {
      const field = issue.path[0] as keyof PolicyDraft
      if (!policyErrors[field]) policyErrors[field] = issue.message
    }
  }

  const generalDirty = general.value.displayName !== pristineGeneral.displayName
  const loginDirty =
    login.value.userRegistration !== pristineLogin.userRegistration ||
    login.value.emailVerification !== pristineLogin.emailVerification ||
    login.value.forgotPassword !== pristineLogin.forgotPassword ||
    login.value.rememberMe !== pristineLogin.rememberMe ||
    login.value.passkey !== pristineLogin.passkey ||
    login.value.magicLink !== pristineLogin.magicLink ||
    login.value.magicLinkTtl !== pristineLogin.magicLinkTtl ||
    login.value.loginAliases.join() !== pristineLogin.loginAliases.join()
  const tokensDirty =
    tokensDraft.value.accessTokenLifetime !== pristineTokens.accessTokenLifetime ||
    tokensDraft.value.refreshTokenLifetime !== pristineTokens.refreshTokenLifetime ||
    tokensDraft.value.idTokenLifetime !== pristineTokens.idTokenLifetime ||
    tokensDraft.value.temporaryTokenLifetime !== pristineTokens.temporaryTokenLifetime
  const policyDirty = (
    Object.keys(pristinePolicy) as (keyof PolicyDraft)[]
  ).some((key) => policy.value[key] !== pristinePolicy[key])

  const dirtyCount =
    (generalDirty ? 1 : 0) +
    (loginDirty ? 1 : 0) +
    (tokensDirty ? 1 : 0) +
    (policyDirty ? 1 : 0)

  const canSave = !displayNameError && !loginAliasesError && policyParsed.success

  const discard = () => {
    general.reset()
    login.reset()
    tokensDraft.reset()
    policy.reset()
  }

  const save = () => {
    if (!realm_name || !canSave) return

    if (generalDirty && realmData) {
      const displayName = general.value.displayName.trim()
      updateRealm(
        {
          path: { name: realm_name },
          body: { name: realmData.name, display_name: displayName ? displayName : null },
        },
        { onSuccess: () => toast.success(t('settings.toast.updated')) }
      )
    }

    if (loginDirty || tokensDirty) {
      updateSettings({
        path: { name: realm_name },
        body: {
          ...(loginDirty
            ? {
                user_registration_enabled: login.value.userRegistration,
                email_verification_enabled: login.value.emailVerification,
                forgot_password_enabled: login.value.forgotPassword,
                remember_me_enabled: login.value.rememberMe,
                passkey_enabled: login.value.passkey,
                magic_link_enabled: login.value.magicLink,
                magic_link_ttl: login.value.magicLinkTtl,
                login_aliases: login.value.loginAliases,
              }
            : {}),
          ...(tokensDirty
            ? {
                access_token_lifetime: tokensDraft.value.accessTokenLifetime,
                refresh_token_lifetime: tokensDraft.value.refreshTokenLifetime,
                id_token_lifetime: tokensDraft.value.idTokenLifetime,
                temporary_token_lifetime: tokensDraft.value.temporaryTokenLifetime,
              }
            : {}),
        },
      })
    }

    if (policyDirty) {
      updatePolicy(
        { path: { realm_name }, body: policy.value },
        {
          onSuccess: () => toast.success(t('settings.toast.policy_updated')),
          onError: (error: Error) =>
            toast.error(apiErrorMessage(error, t('settings.toast.policy_failed'))),
        }
      )
    }
  }

  const handleDeleteRealm = () => {
    if (!realm_name) return

    deleteRealm(
      { path: { name: realm_name } },
      {
        onSuccess: () => {
          toast.success(t('settings.toast.deleted', { name: realm_name }))
          navigate(MASTER_REALM_OVERVIEW_URL)
        },
      }
    )
  }

  const syncWhitelist = (kind: WhitelistKind, current: string[], next: string[]) => {
    for (const id of next) {
      if (current.includes(id)) continue
      addWhitelistEntry({
        body: kind === USER_WHITELIST ? { user_id: id } : { role_id: id },
        path: { realm_name: realm },
      })
    }

    for (const id of current) {
      if (next.includes(id)) continue
      const entryId = entryIdFor(kind, id)
      if (entryId) removeWhitelistEntry({ path: { realm_name: realm, entry_id: entryId } })
    }
  }

  return (
    <PageRealmSettings
      realm={realmData}
      isLoading={realmLoading}
      tab={tab}
      tabs={tabs}
      displayName={general.value.displayName}
      displayNameError={displayNameError}
      signingAlgorithm={settings?.default_signing_algorithm ?? SigningAlgorithm.RS256}
      isMaster={realm_name === 'master'}
      login={login.value}
      loginAliasesError={loginAliasesError}
      tokensValue={tokensDraft.value}
      policy={policy.value}
      policyErrors={policyErrors}
      policyLoading={policyLoading}
      policyFailed={policyError || !policyData}
      maintenanceUsers={users.map((user) => ({
        id: user.id,
        label: user.username,
        sublabel: user.email ?? undefined,
      }))}
      maintenanceRoles={roles.map((role) => ({
        id: role.id,
        label: role.name,
        sublabel: role.description ?? undefined,
      }))}
      whitelistedUserIds={whitelistedUserIds}
      whitelistedRoleIds={whitelistedRoleIds}
      dirtyCount={dirtyCount}
      canSave={canSave}
      onDisplayNameChange={(v) => general.patch({ displayName: v })}
      onLoginChange={login.patch}
      onTokensChange={tokensDraft.patch}
      onPolicyChange={policy.patch}
      onWhitelistedUsersChange={(next) =>
        syncWhitelist(USER_WHITELIST, whitelistedUserIds, next)
      }
      onWhitelistedRolesChange={(next) =>
        syncWhitelist(ROLE_WHITELIST, whitelistedRoleIds, next)
      }
      onDeleteRealm={handleDeleteRealm}
      onDiscard={discard}
      onSave={save}
    />
  )
}
