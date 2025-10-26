import { z } from "zod";
import PageRegister from "../ui/page-register";
import { useForm } from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";
import { useNavigate } from "react-router";

const registerSchema = z.object({
  username: z.string().min(1),
  email: z.string().email(),
  password: z.string().min(6, 'Password must be at least 6 characters long'),
  confirmPassword: z.string().min(6, 'Confirm Password must be at least 6 characters long'),
  firstName: z.string().optional(),
  lastName: z.string().optional(),
}).refine((data) => data.password === data.confirmPassword, {
  message: 'Passwords do not match',
})

export type RegisterSchema = z.infer<typeof registerSchema>

export default function PageRegisterFeature() {
  const navigate = useNavigate()

  const backToLogin = () => {
    navigate('../login')
  }

  const form = useForm<RegisterSchema>({
    resolver: zodResolver(registerSchema),
    defaultValues: {
      username: '',
      email: '',
    },
  })
  return (
    <PageRegister form={form} backToLogin={backToLogin} />
  )
}
