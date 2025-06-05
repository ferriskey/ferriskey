import { useForm } from "react-hook-form";
import SetPassword from "../../ui/modals/set-password";
import { setCredentialPasswordSchema, SetCredentialPasswordSchema } from "../../schemas";
import { zodResolver } from "@hookform/resolvers/zod";
import { Form } from "@/components/ui/form";
import { useEffect, useState } from "react";
import { useResetUserPassword } from "@/api/user.api";
import { useParams } from "react-router";
import { RouterParams } from "@/routes/router";
import { toast } from "sonner";

export default function SetPasswordFeature() {
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
    const values = form.getValues()

    resetPassword({
      realm: realm_name || '',
      userId: user_id || '',
      payload: {
        credential_type: 'password',
        value: values.password,
        temporary: values.temporary,
      }
    })

    form.reset()
    setOpen(false)
  }

  useEffect(() => {
    if (data) {
      toast.success("Password has been set successfully")
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
       />

    </Form>
   
  )
}