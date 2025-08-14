import { zodResolver } from '@hookform/resolvers/zod'
import { useForm } from 'react-hook-form'
import { useParams } from 'react-router'
import { toast } from 'sonner'
import { useGetUser, useUpdateUser } from '@/api/user.api.ts'
import { Form } from '@/components/ui/form.tsx'
import { UserRouterParams } from '@/routes/sub-router/user.router.ts'
import PageUserOverview from '../ui/page-user-overview'
import { UpdateUserSchema, updateUserValidator } from '../validators'
import { useFormChanges } from '@/hooks/use-form-changes.ts'

export default function PageUserOverviewFeature() {
  const { realm_name, user_id } = useParams<UserRouterParams>()
  const { data: userResponse, isLoading } = useGetUser({ realm: realm_name, userId: user_id })
  const { mutate: updateUser } = useUpdateUser()

  const form = useForm<UpdateUserSchema>({
    resolver: zodResolver(updateUserValidator),
    mode: 'all',
    values: {
      username: userResponse?.data.username ?? '',
      firstname: userResponse?.data.firstname ?? '',
      lastname: userResponse?.data.lastname ?? '',
      email: userResponse?.data.email ?? '',
      enabled: userResponse?.data.enabled,
      email_verified: userResponse?.data.email_verified,
      required_actions: userResponse?.data.required_actions,
    },
  })

  const hasChanges = useFormChanges(
    form,
    userResponse && {
      username: userResponse.data.username ?? '',
      firstname: userResponse.data.firstname ?? '',
      lastname: userResponse.data.lastname ?? '',
      email: userResponse.data.email ?? '',
      enabled: userResponse.data.enabled,
      email_verified: userResponse.data.email_verified,
      required_actions: userResponse.data.required_actions ?? [],
    }
  )

  function handleSubmit(payload: UpdateUserSchema) {
    if (!user_id || !realm_name) return;
    updateUser(
      {
        body: payload,
        path: {
          realm_name: realm_name,
          user_id: user_id
        }
      },
      {
        onSuccess: () => toast.success('User was updated'),
        onError: (error) => toast.error(error.message),
      }
    )
  }

  if (!user_id || isLoading || !userResponse) return null

  return (
    <Form {...form}>
      <PageUserOverview onSubmit={handleSubmit} hasChanges={hasChanges} user={userResponse.data} />
    </Form>
  )
}
