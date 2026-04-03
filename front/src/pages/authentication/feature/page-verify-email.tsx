import { useVerifyEmail } from '../../../hooks/use-verify-email'
import { Link } from 'react-router'
import { CheckCircle, XCircle, Loader2 } from 'lucide-react'
import { match } from 'ts-pattern'

export default function PageVerifyEmail() {
  const { state, realm_name } = useVerifyEmail()

  return (
    <div className="flex flex-col items-center justify-center min-h-screen p-4">
      <div className="max-w-md w-full space-y-6 text-center">
        {match(state)
          .with('loading', () => (
            <>
              <Loader2 className="w-12 h-12 animate-spin mx-auto text-primary" />
              <p>Verifying your email...</p>
            </>
          ))
          .with('success', () => (
            <>
              <CheckCircle className="w-16 h-16 mx-auto text-green-600" />
              <h1 className="text-2xl font-bold">Email verified!</h1>
              <p className="text-muted-foreground">
                Your email has been verified. You can now sign in.
              </p>
              <Link
                to={`/realms/${realm_name}/authentication/login`}
                className="inline-block px-6 py-2 bg-primary text-primary-foreground rounded-md"
              >
                Go to login
              </Link>
            </>
          ))
          .with('expired', () => (
            <>
              <XCircle className="w-16 h-16 mx-auto text-amber-500" />
              <h1 className="text-2xl font-bold">Link expired</h1>
              <p className="text-muted-foreground">
                This verification link has expired. Please register again or request a new link.
              </p>
            </>
          ))
          .with('error', () => (
            <>
              <XCircle className="w-16 h-16 mx-auto text-red-500" />
              <h1 className="text-2xl font-bold">Verification failed</h1>
              <p className="text-muted-foreground">The verification link is invalid.</p>
            </>
          ))
          .exhaustive()}
      </div>
    </div>
  )
}
