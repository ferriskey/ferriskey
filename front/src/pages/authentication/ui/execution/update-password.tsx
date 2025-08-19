import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card.tsx'
import { Lock } from 'lucide-react'
import { InputText } from '@/components/ui/input-text.tsx'
import { Button } from '@/components/ui/button.tsx'
import { FormField } from '@/components/ui/form'
import { useFormContext } from 'react-hook-form'
import { UpdatePasswordSchema } from '../../schemas/update-password.schema'

export interface UpdatePasswordProps {
  handleClick: () => void
}

export default function UpdatePassword({ handleClick }: UpdatePasswordProps) {

  const form = useFormContext<UpdatePasswordSchema>()
  const isFormValid = form.formState.isValid

  return (
    <div className='min-h-screen flex items-center justify-center bg-gradient-to-br from-blue-50 to-indigo-100 p-4'>
      <Card className='w-full max-w-md'>
        <CardHeader className='space-y-1 text-center'>
          <div className='mx-auto w-12 h-12 bg-amber-100 rounded-full flex items-center justify-center mb-4'>
            <Lock className='w-6 h-6 text-amber-600' />
          </div>
          <CardTitle className='text-2xl font-bold'>Update Password Required</CardTitle>
          <CardDescription>
            Your password is temporary and must be updated before continuing
          </CardDescription>
        </CardHeader>

        <CardContent className='space-y-4'>
          <div className='space-y-2'>
            <FormField
              control={form.control}
              name='password'
              render={({ field }) => (
                <InputText
                  name={'password'} label={'New Password'}
                  value={field.value}
                  type='password'
                  onChange={field.onChange}
                />
              )}
            />

            <FormField
              control={form.control}
              name='confirmPassword'
              render={({ field }) => (
                <InputText
                  name={'confirmPassword'} label={'Confirm Password'}
                  value={field.value}
                  type='password'
                  onChange={field.onChange}
                />
              )}
            />
          </div>

          <div>
            <Button
              disabled={!isFormValid}
              className='w-full'
              onClick={handleClick}
            >
              Submit
            </Button>
          </div>

        </CardContent>

      </Card>
    </div>
  )

}
