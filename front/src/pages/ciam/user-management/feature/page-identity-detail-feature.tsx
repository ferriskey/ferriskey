import { useState } from 'react'
import { useNavigate, useParams } from 'react-router'
import { toast } from 'sonner'
import { useBulkDeleteUser, useGetUser, useUpdateUser } from '@/api/user.api'
import { RouterParams } from '@/routes/router'
import { useRouteTabs } from '@/components/kit'
import { updateUserValidator } from '@/pages/iam/user/validators'
import { Schemas } from '@/api/api.client'
import PageUserDetail from '@/pages/iam/user/ui/page-user-detail'
import UserCredentialsFeature from '@/pages/iam/user/feature/user-credentials-feature'
import UserRoleMappingFeature from '@/pages/iam/user/feature/user-role-mapping-feature'
import UserOrganizationsFeature from '@/pages/iam/user/feature/user-organizations-feature'
import UserAttributesFeature from '@/pages/iam/user/feature/user-attributes-feature'
import { CONSOLE_IDENTITIES_URL, CONSOLE_IDENTITY_URL } from '../urls'

import RequiredAction = Schemas.RequiredAction

interface Draft {
  key: string
  firstname: string
  lastname: string
  email: string
  enabled: boolean
  emailVerified: boolean
  requiredActions: RequiredAction[]
}

const EMPTY_DRAFT: Draft = {
  key: '',
  firstname: '',
  lastname: '',
  email: '',
  enabled: true,
  emailVerified: false,
  requiredActions: [],
}

const IDENTITY_TABS = [
  { key: 'overview', label: 'Overview' },
  { key: 'credentials', label: 'Credentials' },
  { key: 'role-mapping', label: 'Role mapping' },
  { key: 'organizations', label: 'Organizations' },
  { key: 'attributes', label: 'Attributes' },
] as const

export default function PageIdentityDetailFeature() {
  const { realm_name, user_id } = useParams<RouterParams>()
  const navigate = useNavigate()
  const realm = realm_name ?? 'master'

  const { data: userResponse, isLoading } = useGetUser({ realm, userId: user_id })
  const { mutate: updateUser } = useUpdateUser()
  const { mutateAsync: deleteUser } = useBulkDeleteUser()

  const user = userResponse?.data
  const { value: tab, tabs } = useRouteTabs(CONSOLE_IDENTITY_URL(realm, user_id), IDENTITY_TABS)

  const [draft, setDraft] = useState<Draft>(EMPTY_DRAFT)

  const userKey = user?.id ?? ''
  const pristine: Draft = user
    ? {
        key: userKey,
        firstname: user.firstname ?? '',
        lastname: user.lastname ?? '',
        email: user.email ?? '',
        enabled: user.enabled,
        emailVerified: user.email_verified,
        requiredActions: user.required_actions,
      }
    : EMPTY_DRAFT

  if (user && draft.key !== userKey) setDraft(pristine)

  const current = draft.key === userKey ? draft : pristine
  const { firstname, lastname, email, enabled, emailVerified, requiredActions } = current

  const parsed = updateUserValidator.safeParse({
    username: user?.username ?? '',
    firstname,
    lastname,
    email,
    enabled,
    email_verified: emailVerified,
    required_actions: requiredActions,
  })

  const emailError = parsed.success
    ? undefined
    : parsed.error.issues.find((issue) => issue.path[0] === 'email')?.message

  const dirtyCount = user
    ? (firstname !== pristine.firstname ? 1 : 0) +
      (lastname !== pristine.lastname ? 1 : 0) +
      (email !== pristine.email ? 1 : 0) +
      (enabled !== pristine.enabled ? 1 : 0) +
      (emailVerified !== pristine.emailVerified ? 1 : 0) +
      (requiredActions.length !== pristine.requiredActions.length ||
      requiredActions.some((action) => !pristine.requiredActions.includes(action))
        ? 1
        : 0)
    : 0

  const save = () => {
    if (!user_id || !realm_name || !parsed.success) return

    updateUser(
      {
        body: {
          firstname,
          lastname,
          email,
          enabled,
          email_verified: emailVerified,
          required_actions: requiredActions,
        },
        path: { realm_name, user_id },
      },
      {
        onSuccess: () => toast.success('Identity updated'),
        onError: (error) => toast.error(error.message),
      }
    )
  }

  const handleDelete = async () => {
    if (!user_id || !realm_name) return
    try {
      await deleteUser({ path: { realm_name }, body: { ids: [user_id] } })
      navigate(CONSOLE_IDENTITIES_URL(realm))
    } catch {
      toast.error('The identity could not be deleted')
    }
  }

  return (
    <PageUserDetail
      user={user}
      isLoading={isLoading}
      tab={tab}
      tabs={tabs}
      username={user?.username ?? ''}
      firstname={firstname}
      lastname={lastname}
      email={email}
      enabled={enabled}
      emailVerified={emailVerified}
      requiredActions={requiredActions}
      emailError={emailError}
      dirtyCount={dirtyCount}
      onFirstnameChange={(v) => setDraft((d) => ({ ...d, firstname: v }))}
      onLastnameChange={(v) => setDraft((d) => ({ ...d, lastname: v }))}
      onEmailChange={(v) => setDraft((d) => ({ ...d, email: v }))}
      onEnabledChange={(v) => setDraft((d) => ({ ...d, enabled: v }))}
      onEmailVerifiedChange={(v) => setDraft((d) => ({ ...d, emailVerified: v }))}
      onRequiredActionsChange={(v) => setDraft((d) => ({ ...d, requiredActions: v }))}
      onBack={() => navigate(CONSOLE_IDENTITIES_URL(realm))}
      onDiscard={() => setDraft(pristine)}
      onSave={save}
      onDelete={handleDelete}
    >
      {tab === 'credentials' && <UserCredentialsFeature />}
      {tab === 'role-mapping' && <UserRoleMappingFeature />}
      {tab === 'organizations' && <UserOrganizationsFeature />}
      {tab === 'attributes' && <UserAttributesFeature />}
    </PageUserDetail>
  )
}
