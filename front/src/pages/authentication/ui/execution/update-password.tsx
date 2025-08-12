import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card.tsx'
import { Lock } from 'lucide-react'
import { InputText } from '@/components/ui/input-text.tsx'
import { Button } from '@/components/ui/button.tsx'

export interface UpdatePasswordProps {}

export default function UpdatePassword({}: UpdatePasswordProps) {

  return (
    <div className="min-h-screen flex items-center justify-center bg-gradient-to-br from-blue-50 to-indigo-100 p-4">
      <Card className='w-full max-w-md'>
        <CardHeader className="space-y-1 text-center">
          <div className="mx-auto w-12 h-12 bg-amber-100 rounded-full flex items-center justify-center mb-4">
            <Lock className="w-6 h-6 text-amber-600" />
          </div>
          <CardTitle className="text-2xl font-bold">Update Password Required</CardTitle>
          <CardDescription>
            Your password is temporary and must be updated before continuing
          </CardDescription>
        </CardHeader>

        <CardContent className="space-y-4">
          <div className="space-y-2">
            <InputText name={"password"} label={"New Password"} />

            <InputText name={"password"} label={"Confirm Password"} />
          </div>

          <div>
            <Button disabled className="w-full">
              Submit
            </Button>
          </div>

        </CardContent>

      </Card>
    </div>
  )

}