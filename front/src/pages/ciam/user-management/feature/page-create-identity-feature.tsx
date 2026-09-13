import { useState } from 'react'
import { useNavigate, useParams } from 'react-router'
import { toast } from 'sonner'
import { useCreateUser } from '@/api/user.api'
import { RouterParams } from '@/routes/router'
import { createUserValidator } from '@/pages/iam/user/validators'
import PageCreateUser from '@/pages/iam/user/ui/page-create-user'
import { CONSOLE_IDENTITIES_URL } from '../urls'

export default function PageCreateIdentityFeature() {
  const { realm_name } = useParams<RouterParams>()
  const navigate = useNavigate()
  const realm = realm_name ?? 'master'

  const { mutate: createUser } = useCreateUser()

  const [username, setUsername] = useState('')
  const [firstname, setFirstname] = useState('')
  const [lastname, setLastname] = useState('')
  const [email, setEmail] = useState('')
  const [emailVerified, setEmailVerified] = useState(false)

  const parsed = createUserValidator.safeParse({
    username,
    firstname,
    lastname,
    email,
    email_verified: emailVerified,
  })

  const errors = parsed.success
    ? {}
    : {
        username: parsed.error.issues.find((issue) => issue.path[0] === 'username')?.message,
        email: parsed.error.issues.find((issue) => issue.path[0] === 'email')?.message,
      }

  const dirty =
    username !== '' || firstname !== '' || lastname !== '' || email !== '' || emailVerified

  const back = () => navigate(CONSOLE_IDENTITIES_URL(realm))

  const handleSubmit = () => {
    if (!parsed.success) return

    createUser(
      {
        body: { username, firstname, lastname, email, email_verified: emailVerified },
        path: { realm_name: realm },
      },
      {
        onSuccess: () => {
          toast.success('The identity has been successfully created')
          navigate(CONSOLE_IDENTITIES_URL(realm))
        },
        onError: (error) => toast.error(error.message),
      }
    )
  }

  return (
    <PageCreateUser
      username={username}
      firstname={firstname}
      lastname={lastname}
      email={email}
      emailVerified={emailVerified}
      errors={errors}
      canSubmit={parsed.success && dirty}
      onUsernameChange={setUsername}
      onFirstnameChange={setFirstname}
      onLastnameChange={setLastname}
      onEmailChange={setEmail}
      onEmailVerifiedChange={setEmailVerified}
      onBack={back}
      onSubmit={handleSubmit}
    />
  )
}
