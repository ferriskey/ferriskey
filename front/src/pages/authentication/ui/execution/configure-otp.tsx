import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Heading } from '@/components/ui/heading'
import { InputText } from '@/components/ui/input-text'
import { Separator } from '@/components/ui/separator'
import { CheckCircle, Copy, Shield, Smartphone } from 'lucide-react'
import { useState } from 'react'
import { QRCodeSVG } from 'qrcode.react'
import { Alert, AlertDescription } from '@/components/ui/alert'
import { InputOTP, InputOTPGroup, InputOTPSlot } from '@/components/ui/input-otp'
import { REGEXP_ONLY_DIGITS } from 'input-otp'
import { Skeleton } from '@/components/ui/skeleton'
import { VerifyOtpSchema } from '../../schemas/verify-otp.schema'
import { useFormContext } from 'react-hook-form'
import { FormControl, FormField, FormItem } from '@/components/ui/form'

export interface ConfigureOtpProps {
  secret?: string
  qrCodeUrl?: string
  handleSubmit: (values: VerifyOtpSchema) => void
}

export default function ConfigureOtp({ secret, qrCodeUrl, handleSubmit }: ConfigureOtpProps) {
  const [secretCopied, setSecretCopied] = useState<boolean>(false)
  const form = useFormContext<VerifyOtpSchema>()

  const copySecret = () => {
    if (!secret) return
    navigator.clipboard.writeText(secret)
    setSecretCopied(true)

    setTimeout(() => setSecretCopied(false), 2000)
  }

  const formIsValid = form.formState.isValid

  return (
    <div className="min-h-screen bg-gradient-to-br from-blue-50 via-white to-indigo-50 dark:from-gray-900 dark:via-gray-800 dark:to-gray-900">
      <div className="container mx-auto px-4 py-8">
        <div className="max-w-4xl mx-auto">
          {/* Header */}
          <div className="text-center mb-8">
            <div className="flex justify-center mb-4">
              <div className="p-3 bg-blue-100 dark:bg-blue-900/30 rounded-full">
                <Shield className="h-8 w-8 text-[#19323C] dark:text-blue-400" />
              </div>
            </div>
            <Heading size={2} className="mb-2 text-center">
              Enable Two-Factor Authentication
            </Heading>
            <p className="text-muted-foreground text-lg">
              Secure your account with an additional layer of protection
            </p>
          </div>

          <div className="grid gap-8 md:grid-cols-2">
            {/* Left Column - Setup Instructions */}
            <div className="space-y-6">
              {/* Step 1 */}
              <Card>
                <CardHeader>
                  <CardTitle className="flex items-center gap-2">
                    <div className="w-6 h-6 bg-[#19323C] text-white rounded-full flex items-center justify-center text-sm font-semibold">
                      1
                    </div>
                    Download an Authenticator App
                  </CardTitle>
                  <CardDescription>
                    Install a TOTP authenticator app on your mobile device
                  </CardDescription>
                </CardHeader>
                <CardContent>
                  <div className="grid grid-cols-2 gap-3">
                    <div className="flex items-center gap-2 p-3 border rounded-lg">
                      <Smartphone className="h-5 w-5 text-gray-600" />
                      <span className="text-sm font-medium">Google Authenticator</span>
                    </div>
                    <div className="flex items-center gap-2 p-3 border rounded-lg">
                      <Smartphone className="h-5 w-5 text-gray-600" />
                      <span className="text-sm font-medium">Authy</span>
                    </div>
                    <div className="flex items-center gap-2 p-3 border rounded-lg">
                      <Smartphone className="h-5 w-5 text-gray-600" />
                      <span className="text-sm font-medium">Microsoft Authenticator</span>
                    </div>
                    <div className="flex items-center gap-2 p-3 border rounded-lg">
                      <Smartphone className="h-5 w-5 text-gray-600" />
                      <span className="text-sm font-medium">1Password</span>
                    </div>
                  </div>
                </CardContent>
              </Card>

              {/* Step 2 */}
              <Card>
                <CardHeader>
                  <CardTitle className="flex items-center gap-2">
                    <div className="w-6 h-6 bg-[#19323C] text-white rounded-full flex items-center justify-center text-sm font-semibold">
                      2
                    </div>
                    Scan QR Code or Enter Secret
                  </CardTitle>
                  <CardDescription>
                    Use your authenticator app to scan the QR code or manually enter the secret
                  </CardDescription>
                </CardHeader>
                <CardContent className="space-y-4">
                  {/* QR Code */}
                  <div className="flex justify-center p-4 bg-white dark:bg-gray-800 rounded-lg border">
                    {qrCodeUrl ? (
                      <QRCodeSVG
                        value={qrCodeUrl}
                        size={160}
                        bgColor="transparent"
                        fgColor="currentColor"
                      />
                    ) : (
                      <Skeleton className="h-40 w-40" />
                    )}
                  </div>

                  <Separator />

                  {/* Manual Entry */}
                  <div className="space-y-2">
                    <p className="text-sm font-medium">Can't scan? Enter this secret manually:</p>
                    <div className="flex items-center gap-2">
                      {secret ? (
                        <code className="flex-1 p-2 bg-muted rounded text-sm break-all font-mono">
                          {secret}
                        </code>
                      ) : (
                        <Skeleton className="h-8 w-full" />
                      )}
                      <Button variant="outline" size="sm" onClick={copySecret} className="shrink-0">
                        {secretCopied ? (
                          <CheckCircle className="h-4 w-4 text-green-500" />
                        ) : (
                          <Copy className="h-4 w-4" />
                        )}
                      </Button>
                    </div>
                    {secretCopied && (
                      <p className="text-sm text-green-600 dark:text-green-400">
                        Secret copied to clipboard!
                      </p>
                    )}
                  </div>
                </CardContent>
              </Card>
            </div>

            {/* Right Column - Verification */}
            <div className="space-y-6">
              {/* Step 3 */}
              <Card>
                <CardHeader>
                  <CardTitle className="flex items-center gap-2">
                    <div className="w-6 h-6 bg-[#19323C] text-white rounded-full flex items-center justify-center text-sm font-semibold">
                      3
                    </div>
                    Verify Setup
                  </CardTitle>
                  <CardDescription>
                    Enter the 6-digit code from your authenticator app to verify the setup
                  </CardDescription>
                </CardHeader>
                <CardContent className="space-y-4">
                  <div className="space-y-4">
                    <div>
                      <FormField
                        control={form.control}
                        name="pin"
                        render={({ field }) => (
                          <FormItem>
                            <FormControl>
                              <InputOTP {...field} maxLength={6} pattern={REGEXP_ONLY_DIGITS}>
                                <div className="flex w-full items-center justify-between">
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

                    <FormField
                      control={form.control}
                      name="deviceName"
                      render={({ field }) => (
                        <InputText
                          label="Device Name (Optional)"
                          name="deviceName"
                          value={field.value}
                          onChange={field.onChange}
                        />
                      )}
                    />
                  </div>

                  <Alert>
                    <Shield className="h-4 w-4" />
                    <AlertDescription>
                      Save your backup codes in a secure location. You can use them to recover
                      access if you lose your device.
                    </AlertDescription>
                  </Alert>

                  <div className="space-y-3">
                    <Button
                      className="w-full"
                      size="lg"
                      disabled={!formIsValid}
                      onClick={form.handleSubmit(handleSubmit)}
                    >
                      <Shield className="mr-2 h-4 w-4" />
                      Enable Two-Factor Authentication
                    </Button>

                    <Button variant="outline" className="w-full">
                      Cancel
                    </Button>
                  </div>
                </CardContent>
              </Card>

              {/* Security Tips */}
              <Card className="border-yellow-200 bg-yellow-50 dark:border-yellow-800 dark:bg-yellow-900/20">
                <CardHeader>
                  <CardTitle className="text-yellow-800 dark:text-yellow-200 flex items-center gap-2">
                    <Shield className="h-5 w-5" />
                    Security Tips
                  </CardTitle>
                </CardHeader>
                <CardContent className="space-y-2 text-sm text-yellow-700 dark:text-yellow-300">
                  <p>• Keep your authenticator app updated</p>
                  <p>• Store backup codes in a secure location</p>
                  <p>• Don't share your secret key with anyone</p>
                  <p>• Use a unique device name for easy identification</p>
                </CardContent>
              </Card>
            </div>
          </div>
        </div>
      </div>
    </div>
  )
}
