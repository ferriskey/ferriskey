import { zodResolver } from '@hookform/resolvers/zod'
import { useState } from 'react'
import { useForm } from 'react-hook-form'
import { useParams } from 'react-router'
import { useForgotPasswordMutation } from '@/api/password-reset.api'
import {
  forgotPasswordSchema,
  type ForgotPasswordSchema,
} from '../schemas/forgot-password.schema'
import PageForgotPassword from '../ui/page-forgot-password'
import { Form } from '@/components/ui/form'

export default function PageForgotPasswordFeature() {
  const { realm_name } = useParams()
  const [submitted, setSubmitted] = useState(false)

  const { mutate: forgotPassword, isPending } = useForgotPasswordMutation()

  const form = useForm<ForgotPasswordSchema>({
    resolver: zodResolver(forgotPasswordSchema),
    defaultValues: { email: '' },
  })

  function onSubmit(data: ForgotPasswordSchema) {
    forgotPassword(
      { realm: realm_name ?? 'master', email: data.email },
      { onSuccess: () => setSubmitted(true) }
    )
  }

  return (
    <Form {...form}>
      <PageForgotPassword
        form={form}
        onSubmit={onSubmit}
        submitted={submitted}
        isPending={isPending}
      />
    </Form>
  )
}
