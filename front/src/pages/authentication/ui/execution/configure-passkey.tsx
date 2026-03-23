import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Heading } from '@/components/ui/heading'
import { CheckCircle, Fingerprint, KeyRound, ShieldCheck } from 'lucide-react'
import LoaderSpinner from '@/components/ui/loader-spinner'

export interface ConfigurePasskeyProps {
  onRegister: () => void
  isLoading: boolean
  isSuccess: boolean
}

export default function ConfigurePasskey({ onRegister, isLoading, isSuccess }: ConfigurePasskeyProps) {
  return (
    <div className='min-h-screen bg-gradient-to-br from-blue-50 via-white to-indigo-50 dark:from-gray-900 dark:via-gray-800 dark:to-gray-900'>
      <div className='container mx-auto px-4 py-8'>
        <div className='max-w-lg mx-auto'>
          <div className='text-center mb-8'>
            <div className='flex justify-center mb-4'>
              <div className='p-3 bg-primary dark:bg-primary/30 rounded-full'>
                <Fingerprint className='h-8 w-8 text-[#19323C] dark:text-white' />
              </div>
            </div>
            <Heading size={2} className='mb-2 text-center'>
              Register a Passkey
            </Heading>
            <p className='text-muted-foreground text-lg'>
              Set up a passkey for passwordless sign-in
            </p>
          </div>

          <Card>
            <CardHeader>
              <CardTitle className='flex items-center gap-2'>
                <KeyRound className='h-5 w-5' />
                Passkey Setup
              </CardTitle>
              <CardDescription>
                Your browser will prompt you to create a passkey using your device&apos;s built-in
                authenticator (fingerprint, face recognition, or security key).
              </CardDescription>
            </CardHeader>
            <CardContent className='space-y-4'>
              {isSuccess ? (
                <div className='text-center py-6'>
                  <div className='flex justify-center mb-4'>
                    <div className='p-3 bg-green-100 dark:bg-green-900/30 rounded-full'>
                      <CheckCircle className='h-8 w-8 text-green-600 dark:text-green-400' />
                    </div>
                  </div>
                  <p className='text-lg font-semibold text-green-700 dark:text-green-300'>
                    Passkey registered successfully!
                  </p>
                  <p className='text-sm text-muted-foreground mt-2'>
                    Redirecting...
                  </p>
                </div>
              ) : (
                <>
                  <div className='rounded-lg border border-border bg-background p-4 space-y-3'>
                    <div className='flex items-start gap-3'>
                      <ShieldCheck className='h-5 w-5 text-primary mt-0.5 shrink-0' />
                      <div>
                        <p className='text-sm font-medium'>Secure & convenient</p>
                        <p className='text-xs text-muted-foreground'>
                          Passkeys use biometrics or your device PIN — no password to remember.
                        </p>
                      </div>
                    </div>
                    <div className='flex items-start gap-3'>
                      <Fingerprint className='h-5 w-5 text-primary mt-0.5 shrink-0' />
                      <div>
                        <p className='text-sm font-medium'>Works across devices</p>
                        <p className='text-xs text-muted-foreground'>
                          Synced via iCloud Keychain, Google Password Manager, or Windows Hello.
                        </p>
                      </div>
                    </div>
                  </div>

                  <Button
                    className='w-full'
                    size='lg'
                    onClick={onRegister}
                    disabled={isLoading}
                  >
                    {isLoading ? (
                      <LoaderSpinner />
                    ) : (
                      <>
                        <KeyRound className='mr-2 h-4 w-4' />
                        Register Passkey
                      </>
                    )}
                  </Button>
                </>
              )}
            </CardContent>
          </Card>
        </div>
      </div>
    </div>
  )
}
