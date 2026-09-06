import { zodResolver } from '@hookform/resolvers/zod'
import { useForm } from 'react-hook-form'
import { useParams } from 'react-router'
import { toast } from 'sonner'
import { useGetOwnProfile, useUpdateOwnProfile } from '@/api/user.api.ts'
import { useGetRealm } from '@/api/realm.api.ts'
import { Form } from '@/components/ui/form.tsx'
import { RouterParams } from '@/routes/router'
import { useFormChanges } from '@/hooks/use-form-changes.ts'
import PageAccount from '../ui/page-account'
import { UpdateOwnProfileSchema, updateOwnProfileValidator } from '../validators'

export default function PageAccountFeature() {
  const { realm_name } = useParams<RouterParams>()
  const { data: profileResponse, isLoading } = useGetOwnProfile({ realm: realm_name })
  const { data: realm } = useGetRealm({ realm: realm_name })
  const { mutate: updateOwnProfile } = useUpdateOwnProfile()

  const usernameEditable = realm?.settings?.edit_username_enabled ?? false

  const form = useForm<UpdateOwnProfileSchema>({
    resolver: zodResolver(updateOwnProfileValidator),
    mode: 'all',
    values: {
      username: profileResponse?.data.username ?? '',
      firstname: profileResponse?.data.firstname ?? '',
      lastname: profileResponse?.data.lastname ?? '',
      email: profileResponse?.data.email ?? '',
    },
  })

  const hasChanges = useFormChanges(
    form,
    profileResponse && {
      username: profileResponse.data.username ?? '',
      firstname: profileResponse.data.firstname ?? '',
      lastname: profileResponse.data.lastname ?? '',
      email: profileResponse.data.email ?? '',
    }
  )

  function handleSubmit(payload: UpdateOwnProfileSchema) {
    if (!realm_name) return
    updateOwnProfile(
      {
        body: payload,
        path: { realm_name },
      },
      {
        onSuccess: () => toast.success('Your profile was updated'),
        onError: (error) => {
          toast.error(error.message)
        },
      }
    )
  }

  if (isLoading || !profileResponse) return null

  return (
    <Form {...form}>
      <PageAccount onSubmit={handleSubmit} hasChanges={hasChanges} usernameEditable={usernameEditable} />
    </Form>
  )
}
