import { useCreateUser } from '@/api/user.api'
import { RouterParams } from '@/routes/router'
import { IDENTITIES_URL } from '@/routes/sub-router/user-management.router'
import { useNavigate, useParams } from 'react-router'
import { toast } from 'sonner'
import PageCreateIdentity, { CreateIdentityValues } from '../ui/page-create-identity'

export default function PageCreateIdentityFeature() {
  const { realm_name } = useParams<RouterParams>()
  const navigate = useNavigate()
  const { mutate, isPending } = useCreateUser()

  const handleCancel = () => {
    if (!realm_name) return
    navigate(IDENTITIES_URL(realm_name))
  }

  const handleSubmit = (values: CreateIdentityValues) => {
    if (!realm_name) return
    mutate(
      {
        body: {
          username: values.username,
          email: values.email || undefined,
          firstname: values.firstname || undefined,
          lastname: values.lastname || undefined,
          email_verified: values.emailVerified,
        },
        path: { realm_name },
      },
      {
        onSuccess: () => {
          toast.success('Identity created')
          navigate(IDENTITIES_URL(realm_name))
        },
        onError: (error) => toast.error(error.message ?? 'Failed to create identity'),
      },
    )
  }

  return <PageCreateIdentity onCancel={handleCancel} onSubmit={handleSubmit} isSubmitting={isPending} />
}
