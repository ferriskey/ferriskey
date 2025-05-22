import { useForm } from "react-hook-form"
import CreateClientModal from "../ui/create-client-modal"
import { zodResolver } from "@hookform/resolvers/zod"
import { z } from "zod"
import { useCreateClient } from "@/api/client.api"

const createClientSchema = z.object({
  clientId: z.string().min(1, { message: "Le nom d'utilisateur est requis" }),
  name: z.string().min(1, { message: "L'email doit être valide" }),
  enabled: z.boolean().optional(),
  clientAuthentication: z.boolean().optional(),
  protocol: z.string().optional(),
})

export type CreateClientSchema = z.infer<typeof createClientSchema>

export default function CreateClientModalFeature() {
  const { mutate } = useCreateClient()

  const form = useForm<CreateClientSchema>({
    resolver: zodResolver(createClientSchema),
    defaultValues: {
      clientId: "",
      name: "",
      enabled: false,
      protocol: "openid-connect",
      clientAuthentication: false
    }
  })

  const onSubmit = () => {
    const data = form.getValues()

    mutate({
      realm: "master",
      payload: {
        ...data,
      }
    })
  }


  return <CreateClientModal form={form} onSubmit={onSubmit} />
}