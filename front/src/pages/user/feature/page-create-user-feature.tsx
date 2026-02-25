import { useCreateUser } from '@/api/user.api'
import { useForm } from 'react-hook-form'
import { CreateUserSchema, createUserValidator } from '@/pages/user/validators'
import { zodResolver } from '@hookform/resolvers/zod'
import { useNavigate, useParams } from 'react-router-dom'
import { RouterParams } from '@/routes/router'
import { useMemo } from 'react'
import { USERS_URL, USER_OVERVIEW_URL } from '@/routes/sub-router/user.router'
import { toast } from 'sonner'
import PageCreateUser from '@/pages/user/ui/page-create-user'
import { Form } from '@/components/ui/form'

export default function PageCreateUserFeature() {
  const { realm_name } = useParams<RouterParams>()
  const navigate = useNavigate()
  const { mutate: createUser } = useCreateUser()

  const form = useForm<CreateUserSchema>({
    resolver: zodResolver(createUserValidator),
    defaultValues: {
      username: '',
      firstname: '',
      lastname: '',
      email: '',
      email_verified: false,
    },
  })

  const overviewUrl = useMemo(() => {
    if (!realm_name) return ''
    return `${USERS_URL(realm_name)}${USER_OVERVIEW_URL}`
  }, [realm_name])

  const handleBack = () => {
    navigate(overviewUrl)
  }

  const onSubmit = () => {
    if (!realm_name) return

    const data = form.getValues()

    createUser(
      {
        body: data,
        path: { realm_name },
      },
      {
        onSuccess: () => {
          toast.success('The user has been successfully created')
          navigate(overviewUrl)
        },
        onError: (error) => {
          toast.error(error.message)
        },
      }
    )
  }

  const formIsValid = form.formState.isValid && form.formState.isDirty

  return (
    <Form {...form}>
      <PageCreateUser
        form={form}
        handleBack={handleBack}
        handleSubmit={onSubmit}
        formIsValid={formIsValid}
      />
    </Form>
  )
}
