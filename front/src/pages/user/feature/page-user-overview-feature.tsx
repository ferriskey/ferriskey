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
  const { data, isLoading } = useGetUser({ realm: realm_name, userId: user_id })
  const { mutate: updateUser } = useUpdateUser()

  const form = useForm<UpdateUserSchema>({
    resolver: zodResolver(updateUserValidator),
    mode: 'all',
    values: {
      username: data?.username ?? '',
      firstname: data?.firstname ?? '',
      lastname: data?.lastname ?? '',
      email: data?.email ?? '',
      enabled: data?.enabled,
      email_verified: data?.email_verified,
    },
  })

  const hasChanges = useFormChanges(form, data && {
    username: data.username ?? '',
    firstname: data.firstname ?? '',
    lastname: data.lastname ?? '',
    email: data.email ?? '',
    enabled: data.enabled,
    email_verified: data.email_verified,
  })

  function handleSubmit(payload: UpdateUserSchema) {
    updateUser({ realm: realm_name, userId: user_id, payload }, {
      onSuccess: () => toast.success("User was updated"),
      onError: (error) => toast.error(error.message)
    })
  }

  if (!user_id || isLoading) return null

  return (
    <Form {...form}>
      <PageUserOverview
        onSubmit={handleSubmit}
        hasChanges={hasChanges}
      />
    </Form >
  )
}
