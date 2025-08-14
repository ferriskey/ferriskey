import { zodResolver } from '@hookform/resolvers/zod'
import { Dispatch, SetStateAction } from 'react'
import { useForm } from 'react-hook-form'
import { toast } from 'sonner'
import { useCreateUser } from '../../../api/user.api'
import { Form } from '../../../components/ui/form'
import CreateUserModal from '../ui/create-user-modal'
import { CreateUserSchema, createUserValidator } from '../validators'

type Props = {
  realm: string
  open: boolean
  setOpen: Dispatch<SetStateAction<boolean>>
}

export default function CreateUserModalFeature({ realm, open, setOpen }: Props) {
  const { mutate: createUser } = useCreateUser()

  const form = useForm<CreateUserSchema>({
    resolver: zodResolver(createUserValidator),
    defaultValues: {
      username: '',
      email: '',
      firstname: '',
      lastname: '',
      email_verified: false,
    },
  })

  function onSubmit(data: CreateUserSchema) {
    createUser(
      {
        body: data,
        path: {
          realm_name: realm
        }
      },
      {
        onSuccess: () => {
          form.reset()
          setOpen(false)
          toast.success('User was created')
        },
        onError: (error) => {
          toast.error(error.message)
        },
      }
    )
  }

  return (
    <Form {...form}>
      <CreateUserModal realm={realm} onSubmit={onSubmit} openState={[open, setOpen]} />
    </Form>
  )
}
