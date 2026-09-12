import SaveBar from '@/components/kit/save-bar'
import { PageShell, PageTabs, type TabItem } from '@/components/kit'
import type { PickableEntity } from '@/components/kit'
import { cn } from '@/lib/utils'
import { tokens } from '@/styles/style-tokens'
import { Schemas } from '@/api/api.client'
import RealmGeneralTab from './realm-general-tab'
import RealmLoginTab, { type LoginDraft } from './realm-login-tab'
import RealmTokensTab, { type TokensDraft } from './realm-tokens-tab'
import RealmPasswordPolicyTab, { type PolicyDraft } from './realm-password-policy-tab'
import RealmMaintenanceTab from './realm-maintenance-tab'

import Realm = Schemas.Realm

export interface PageRealmSettingsProps {
  realm?: Realm
  isLoading: boolean
  tab: string
  tabs: TabItem[]
  displayName: string
  displayNameError?: string
  signingAlgorithm: string
  isMaster: boolean
  login: LoginDraft
  loginAliasesError?: string
  tokensValue: TokensDraft
  policy: PolicyDraft
  policyErrors: Partial<Record<keyof PolicyDraft, string>>
  policyLoading: boolean
  policyFailed: boolean
  maintenanceUsers: PickableEntity[]
  maintenanceRoles: PickableEntity[]
  whitelistedUserIds: string[]
  whitelistedRoleIds: string[]
  dirtyCount: number
  canSave: boolean
  onDisplayNameChange: (v: string) => void
  onLoginChange: (patch: Partial<LoginDraft>) => void
  onTokensChange: (patch: Partial<TokensDraft>) => void
  onPolicyChange: (patch: Partial<PolicyDraft>) => void
  onWhitelistedUsersChange: (next: string[]) => void
  onWhitelistedRolesChange: (next: string[]) => void
  onDeleteRealm: () => void
  onDiscard: () => void
  onSave: () => void
}

export default function PageRealmSettings({
  realm,
  isLoading,
  tab,
  tabs,
  displayName,
  displayNameError,
  signingAlgorithm,
  isMaster,
  login,
  loginAliasesError,
  tokensValue,
  policy,
  policyErrors,
  policyLoading,
  policyFailed,
  maintenanceUsers,
  maintenanceRoles,
  whitelistedUserIds,
  whitelistedRoleIds,
  dirtyCount,
  canSave,
  onDisplayNameChange,
  onLoginChange,
  onTokensChange,
  onPolicyChange,
  onWhitelistedUsersChange,
  onWhitelistedRolesChange,
  onDeleteRealm,
  onDiscard,
  onSave,
}: PageRealmSettingsProps) {
  if (isLoading) {
    return (
      <PageShell>
        <div className='h-5 w-48 animate-pulse rounded bg-neutral-100 dark:bg-fk-raised' />
        <div className='mt-2 h-4 w-72 animate-pulse rounded bg-neutral-100 dark:bg-fk-raised' />
        <div className='mt-6 h-40 animate-pulse rounded bg-neutral-100 dark:bg-fk-raised' />
      </PageShell>
    )
  }

  if (!realm) {
    return (
      <PageShell>
        <div className={cn(tokens.surface.panel, 'grid place-items-center px-6 py-16')}>
          <p className='text-sm font-medium text-neutral-700 dark:text-neutral-300'>Realm not found</p>
          <p className='mt-1 max-w-sm text-center text-sm text-neutral-500 dark:text-neutral-400'>
            It may have been deleted, or your account no longer has access to it.
          </p>
        </div>
      </PageShell>
    )
  }

  return (
    <PageShell>
      <div
        className={cn(
          'flex flex-wrap items-start justify-between gap-3',
          tokens.header.spacing
        )}
      >
        <div className='min-w-0'>
          <h1 className={tokens.header.title}>Realm Settings</h1>
          <p className='mt-0.5 text-sm text-neutral-500 dark:text-neutral-400'>
            Global configuration of the realm {realm.name}.
          </p>
        </div>
        <span className='shrink-0 font-mono-ui text-[11px] text-neutral-400 dark:text-neutral-500'>
          {realm.id}
        </span>
      </div>

      <PageTabs tabs={tabs} value={tab}>
        <div className={tokens.page.blockGap}>
          {tab === 'general' && (
            <RealmGeneralTab
              realmName={realm.name}
              displayName={displayName}
              displayNameError={displayNameError}
              signingAlgorithm={signingAlgorithm}
              isMaster={isMaster}
              onDisplayNameChange={onDisplayNameChange}
              onDelete={onDeleteRealm}
            />
          )}

          {tab === 'login' && (
            <RealmLoginTab
              value={login}
              aliasesError={loginAliasesError}
              onChange={onLoginChange}
            />
          )}

          {tab === 'tokens' && (
            <RealmTokensTab value={tokensValue} onChange={onTokensChange} />
          )}

          {tab === 'password-policy' &&
            (policyLoading ? (
              <div className='h-40 animate-pulse rounded bg-neutral-100 dark:bg-fk-raised' />
            ) : policyFailed ? (
              <div
                className={cn(tokens.surface.panel, 'grid place-items-center px-6 py-12')}
              >
                <p className='text-sm font-medium text-neutral-700 dark:text-neutral-300'>
                  Failed to load password policy
                </p>
                <p className='mt-1 max-w-sm text-center text-sm text-neutral-500 dark:text-neutral-400'>
                  The realm has no password policy the console can read.
                </p>
              </div>
            ) : (
              <RealmPasswordPolicyTab
                value={policy}
                errors={policyErrors}
                onChange={onPolicyChange}
              />
            ))}

          {tab === 'maintenance' && (
            <RealmMaintenanceTab
              users={maintenanceUsers}
              roles={maintenanceRoles}
              selectedUserIds={whitelistedUserIds}
              selectedRoleIds={whitelistedRoleIds}
              onUsersChange={onWhitelistedUsersChange}
              onRolesChange={onWhitelistedRolesChange}
            />
          )}
        </div>
      </PageTabs>

      <SaveBar
        show={dirtyCount > 0}
        title={`${dirtyCount} unsaved change${dirtyCount > 1 ? 's' : ''}`}
        description='Review the realm configuration before applying the changes.'
        onCancel={onDiscard}
        cancelLabel='Discard'
        actions={[
          { label: 'Save changes', onClick: onSave, variant: canSave ? 'default' : 'secondary' },
        ]}
      />
    </PageShell>
  )
}
