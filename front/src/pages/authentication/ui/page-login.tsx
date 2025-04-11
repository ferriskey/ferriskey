import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Form, FormField } from '@/components/ui/form'
import { Input } from '@/components/ui/input'
import { UseFormReturn } from 'react-hook-form'
import { AuthenticateSchema } from '../feature/page-login-feature'

export interface PageLoginProps {
  form: UseFormReturn<AuthenticateSchema>
  onSubmit: (data: AuthenticateSchema) => void
  isError?: boolean
}

export default function PageLogin({ form, onSubmit, isError }: PageLoginProps) {
  console.log(isError)
  if (isError) return <ErrorMessage />
  return (
    <div className="flex min-h-svh flex-col items-center justify-center gap-6 bg-muted p-6 md:p-10">
      <div className="flex w-full max-w-sm flex-col gap-6">
        <a href="#" className="flex items-center gap-2 self-center font-medium">
          <div className="flex h-10 w-10 border rounded-md overflow-hidden items-center justify-center bg-primary text-primary-foreground">
            {/* <GalleryVerticalEnd className='size-4' /> */}
            <img src="/logo_ferriskey.png" alt="" />
          </div>
          FerrisKey
        </a>
        <div className="flex flex-col gap-6">
          <Card>
            <CardHeader className="text-center">
              <CardTitle className="text-xl">Welcome back</CardTitle>
            </CardHeader>
            <CardContent>
              <Form {...form}>
                <div className="flex flex-col gap-3">
                  <FormField
                    control={form.control}
                    name="username"
                    render={({ field }) => (
                      <Input {...field} placeholder="Username" className="w-full" />
                    )}
                  />

                  <FormField
                    control={form.control}
                    name="password"
                    render={({ field }) => (
                      <Input {...field} placeholder="Password" className="w-full" />
                    )}
                  />

                  <Button onClick={() => onSubmit(form.getValues())}>Submit</Button>
                </div>
              </Form>
            </CardContent>
          </Card>
        </div>
      </div>
    </div>
  )
}

function ErrorMessage() {
  return (
    <div>
      <p>Une erreur est survenue lors de la connexion</p>
      <p>Veuillez réessayer</p>
    </div>
  )
}
