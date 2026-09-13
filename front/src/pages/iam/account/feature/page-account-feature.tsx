import { useState } from 'react'
import { useParams } from 'react-router'
import { toast } from 'sonner'
import { useGetOwnProfile, useUpdateOwnProfile } from '@/api/user.api'
import { useGetRealm } from '@/api/realm.api'
import { RouterParams } from '@/routes/router'
import { updateOwnProfileValidator } from '@/pages/iam/account/validators'
import PageAccount from '../ui/page-account'

interface Draft {
  key: string
  username: string
  firstname: string
  lastname: string
  email: string
}

const EMPTY_DRAFT: Draft = { key: '', username: '', firstname: '', lastname: '', email: '' }

export default function PageAccountFeature() {
  const { realm_name } = useParams<RouterParams>()
  const realm = realm_name ?? 'master'

  const { data: profileResponse, isLoading } = useGetOwnProfile({ realm })
  const { data: realmData } = useGetRealm({ realm })
  const { mutate: updateOwnProfile } = useUpdateOwnProfile()

  const profile = profileResponse?.data
  const usernameEditable = realmData?.settings?.edit_username_enabled ?? false

  const [draft, setDraft] = useState<Draft>(EMPTY_DRAFT)

  const profileKey = profile?.id ?? ''
  const pristine: Draft = profile
    ? {
        key: profileKey,
        username: profile.username,
        firstname: profile.firstname ?? '',
        lastname: profile.lastname ?? '',
        email: profile.email ?? '',
      }
    : EMPTY_DRAFT

  if (profile && draft.key !== profileKey) setDraft(pristine)

  const current = draft.key === profileKey ? draft : pristine
  const { username, firstname, lastname, email } = current

  const parsed = updateOwnProfileValidator.safeParse({
    username,
    firstname,
    lastname,
    email,
  })

  const errors = parsed.success
    ? {}
    : {
        username: parsed.error.issues.find((i) => i.path[0] === 'username')?.message,
        email: parsed.error.issues.find((i) => i.path[0] === 'email')?.message,
      }

  const dirtyCount = profile
    ? (username !== pristine.username ? 1 : 0) +
      (email !== pristine.email ? 1 : 0) +
      (firstname !== pristine.firstname ? 1 : 0) +
      (lastname !== pristine.lastname ? 1 : 0)
    : 0

  const save = () => {
    if (!parsed.success) return

    updateOwnProfile(
      {
        body: { username, firstname, lastname, email },
        path: { realm_name: realm },
      },
      {
        onSuccess: () => toast.success('Your profile was updated'),
        onError: (error) => toast.error(error.message),
      }
    )
  }

  return (
    <PageAccount
      profile={profile}
      isLoading={isLoading}
      username={username}
      firstname={firstname}
      lastname={lastname}
      email={email}
      usernameEditable={usernameEditable}
      errors={errors}
      dirtyCount={dirtyCount}
      onUsernameChange={(v) => setDraft((d) => ({ ...d, username: v }))}
      onFirstnameChange={(v) => setDraft((d) => ({ ...d, firstname: v }))}
      onLastnameChange={(v) => setDraft((d) => ({ ...d, lastname: v }))}
      onEmailChange={(v) => setDraft((d) => ({ ...d, email: v }))}
      onDiscard={() => setDraft(pristine)}
      onSave={save}
    />
  )
}
