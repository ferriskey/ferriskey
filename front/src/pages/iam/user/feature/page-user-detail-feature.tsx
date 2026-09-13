import { useState } from 'react'
import { useNavigate, useParams } from 'react-router'
import { toast } from 'sonner'
import { useBulkDeleteUser, useGetUser, useUpdateUser } from '@/api/user.api'
import { RouterParams } from '@/routes/router'
import { useRouteTabs } from '@/components/kit'
import { updateUserValidator } from '@/pages/iam/user/validators'
import { USERS_URL } from '@/routes/router'
import { Schemas } from '@/api/api.client'
import PageUserDetail from '../ui/page-user-detail'
import UserCredentialsFeature from './user-credentials-feature'
import UserRoleMappingFeature from './user-role-mapping-feature'
import UserOrganizationsFeature from './user-organizations-feature'
import UserAttributesFeature from './user-attributes-feature'

import RequiredAction = Schemas.RequiredAction
import { useCrumbLabel } from '@/components/shell/crumb-store'

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

const USER_TABS = [
  { key: 'overview', label: 'Overview' },
  { key: 'credentials', label: 'Credentials' },
  { key: 'role-mapping', label: 'Role mapping' },
  { key: 'organizations', label: 'Organizations' },
  { key: 'attributes', label: 'Attributes' },
] as const

export default function PageUserDetailFeature() {
  const { realm_name, user_id } = useParams<RouterParams>()
  const navigate = useNavigate()
  const realm = realm_name ?? 'master'

  const { data: userResponse, isLoading } = useGetUser({ realm, userId: user_id })
  const { mutate: updateUser } = useUpdateUser()
  const { mutateAsync: deleteUser } = useBulkDeleteUser()

  const user = userResponse?.data
  const basePath = `${USERS_URL(realm)}/${user_id ?? ''}`
  const { value: tab, tabs } = useRouteTabs(basePath, USER_TABS)

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
    : parsed.error.issues.find((i) => i.path[0] === 'email')?.message

  const dirtyCount = user
    ? (firstname !== pristine.firstname ? 1 : 0) +
      (lastname !== pristine.lastname ? 1 : 0) +
      (email !== pristine.email ? 1 : 0) +
      (enabled !== pristine.enabled ? 1 : 0) +
      (emailVerified !== pristine.emailVerified ? 1 : 0) +
      (requiredActions.length !== pristine.requiredActions.length ||
      requiredActions.some((a) => !pristine.requiredActions.includes(a))
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
        onSuccess: () => toast.success('User was updated'),
        onError: (error) => toast.error(error.message),
      }
    )
  }

  const handleDelete = async () => {
    if (!user_id || !realm_name) return
    try {
      await deleteUser({ path: { realm_name }, body: { ids: [user_id] } })
      navigate(USERS_URL(realm))
    } catch {
      toast.error('The user could not be deleted')
    }
  }


  useCrumbLabel(user_id, user?.username)

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
      onBack={() => navigate(USERS_URL(realm))}
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
