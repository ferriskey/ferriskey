import { MagicCard } from '@/components/magicui/magic-card'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { InputOTP, InputOTPGroup, InputOTPSlot } from '@/components/ui/input-otp'
import { REGEXP_ONLY_DIGITS } from 'input-otp'
import { Shield } from 'lucide-react'
import { ChallengeOtpSchema } from '../schemas/challange-otp.schema'
import { useFormContext } from 'react-hook-form'
import { FormControl, FormField, FormItem } from '@/components/ui/form'

export interface PageOtpChallengeProps {
  handleCancelClick: () => void
  handleClick: (values: ChallengeOtpSchema) => void
  email?: string
}

export default function PageOtpChallenge({
  handleCancelClick,
  handleClick,
  email,
}: PageOtpChallengeProps) {
  const form = useFormContext<ChallengeOtpSchema>()

  return (
    <div className="flex min-h-svh flex-col items-center justify-center bg-muted p-6 md:p-10">
      <div className="w-full max-w-sm md:max-w-xl">
        <div>
          <Card className="overflow-hidden p-0">
            <MagicCard className="p-4" gradientColor="#D9D9D955">
              <CardHeader className="text-center space-y-4">
                <div className="mx-auto w-16 h-16 bg-gradient-to-br from-blue-500 to-indigo-600 rounded-full flex items-center justify-center">
                  <Shield className="w-8 h-8 text-white" />
                </div>

                <div className="space-y-2">
                  <CardTitle className="text-2xl font-bold text-gray-900">
                    Verification Code
                  </CardTitle>
                  <CardDescription className="text-gray-600">
                    Enter the 6-digit code from your authenticator app
                    <span className="block text-sm font-medium text-gray-800 mt-1">
                      for {email}
                    </span>
                  </CardDescription>
                </div>
              </CardHeader>
              <CardContent className="space-y-6 mt-4">
                <div className="space-y-4">
                  <div className="flex justify-center gap-3 w-full">
                    <FormField
                      control={form.control}
                      name="code"
                      render={({ field }) => (
                        <FormItem>
                          <FormControl>
                            <InputOTP {...field} maxLength={6} pattern={REGEXP_ONLY_DIGITS}>
                              <div className="flex items-center gap-4">
                                <InputOTPGroup>
                                  <InputOTPSlot className="w-11 h-11" index={0} />
                                </InputOTPGroup>

                                <InputOTPGroup>
                                  <InputOTPSlot className="w-11 h-11" index={1} />
                                </InputOTPGroup>

                                <InputOTPGroup>
                                  <InputOTPSlot className="w-11 h-11" index={2} />
                                </InputOTPGroup>

                                <InputOTPGroup>
                                  <InputOTPSlot className="w-11 h-11" index={3} />
                                </InputOTPGroup>

                                <InputOTPGroup>
                                  <InputOTPSlot className="w-11 h-11" index={4} />
                                </InputOTPGroup>

                                <InputOTPGroup>
                                  <InputOTPSlot className="w-11 h-11" index={5} />
                                </InputOTPGroup>
                              </div>
                            </InputOTP>
                          </FormControl>
                        </FormItem>
                      )}
                    />
                  </div>

                  <p className="text-xs text-gray-500 text-center">
                    Paste your code or enter it digit by digit
                  </p>
                </div>

                <div className="mt-4 flex flex-col gap-2">
                  <Button
                    disabled={!form.formState.isValid}
                    onClick={form.handleSubmit(handleClick)}
                  >
                    Sign In
                  </Button>

                  <Button variant="outline" onClick={handleCancelClick}>
                    Cancel
                  </Button>
                </div>
              </CardContent>
            </MagicCard>
          </Card>
        </div>
      </div>
    </div>
  )
}
