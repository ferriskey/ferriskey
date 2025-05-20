import { z } from "zod";
import CreateUserModal from "../ui/create-user-modal";
import { useForm } from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";

export const createUserSchema = z.object({
  username: z.string().min(1, { message: "Le nom d'utilisateur est requis" }),
  email: z.string().email({ message: "L'email doit être valide" }),
  firstName: z.string().min(1, { message: "Le prénom est requis" }),
  lastName: z.string().min(1, { message: "Le nom est requis" }),
  emailVerified: z.boolean().optional(),
})

export type CreateUserSchema = z.infer<typeof createUserSchema>

export default function CreateUserModalFeature() {
  const form = useForm<CreateUserSchema>({
    resolver: zodResolver(createUserSchema),
    defaultValues: {
      username: "",
      email: "",
      firstName: "",
      lastName: "",
      emailVerified: false,
    }
  })

  const onSubmit = (data: CreateUserSchema) => {
    console.log(data)
  }

  return (
    <CreateUserModal form={form} onSubmit={onSubmit}  />
  )
}