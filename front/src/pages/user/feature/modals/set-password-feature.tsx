import { useForm } from 'react-hook-form'
import SetPassword from '../../ui/modals/set-password'
import { setCredentialPasswordSchema, SetCredentialPasswordSchema } from '../../schemas'
import { zodResolver } from '@hookform/resolvers/zod'
import { Form } from '@/components/ui/form'
import { useEffect, useState } from 'react'
import { useResetUserPassword } from '@/api/user.api'
import { useParams } from 'react-router'
import { RouterParams } from '@/routes/router'
import { toast } from 'sonner'

export interface SetPasswordFeatureProps {
  contentText?: string
}

export default function SetPasswordFeature({ contentText }: SetPasswordFeatureProps) {
  const [open, setOpen] = useState(false)
  const { realm_name, user_id } = useParams<RouterParams>()
  const { mutate: resetPassword, data } = useResetUserPassword()
  const form = useForm<SetCredentialPasswordSchema>({
    resolver: zodResolver(setCredentialPasswordSchema),
    defaultValues: {
      password: '',
      confirmPassword: '',
      temporary: false,
    }
  })

  const handleCloseModal = () => {
    form.reset()
    setOpen(false)
  }

  const handleSubmit = () => {
    if (!user_id || !realm_name) {
      toast.error("User ID or Realm Name is missing")
      return
    }
    const values = form.getValues()

    resetPassword({
      body: {
        credential_type: 'password',
        temporary: values.temporary,
        value: values.password
      },
      path: {
        realm_name,
        user_id
      }
    })

    form.reset()
    setOpen(false)
  }

  useEffect(() => {
    if (data) {
      toast.success('Password has been set successfully')
    }
  }, [data])


  return (
    <Form {...form}>
      <SetPassword
        form={form}
        open={open}
        setOpen={setOpen}
        handleCloseModal={handleCloseModal}
        handleSubmit={handleSubmit}
        contentText={contentText}
      />

    </Form>

  )
}
