import { zodResolver } from '@hookform/resolvers/zod';
import { useForm } from 'react-hook-form';
import { useParams } from 'react-router';
import { toast } from 'sonner';
import { useGetUser, useUpdateUser } from '../../../api/user.api';
import { Form } from '../../../components/ui/form';
import { UserRouterParams } from '../../../routes/sub-router/user.router';
import EditUserOverview from '../ui/edit-user-overview';
import { UpdateUserSchema, updateUserValidator } from '../validators';

export default function EditUserOverviewFeature() {
  const { realm_name, user_id } = useParams<UserRouterParams>()
  const { data, isLoading } = useGetUser({ realm: realm_name, userId: user_id })
  const { mutate: updateUser } = useUpdateUser()

  const form = useForm<UpdateUserSchema>({
    resolver: zodResolver(updateUserValidator),
    values: {
      username: data?.username ?? '',
      firstname: data?.firstname ?? '',
      lastname: data?.lastname ?? '',
      email: data?.email ?? '',
      enabled: data?.enabled,
      email_verified: data?.email_verified,
    },
  })

  if (!user_id || isLoading) return

  function handleSubmit(payload: UpdateUserSchema) {
    updateUser({ realm: realm_name, userId: user_id, payload }, {
      onSuccess: () => toast.success("User was updated"),
      onError: (error) => toast.error(error.message)
    })
  }

  return (
    <Form {...form}>
      <EditUserOverview onSubmit={handleSubmit} />
    </Form >
  )
}