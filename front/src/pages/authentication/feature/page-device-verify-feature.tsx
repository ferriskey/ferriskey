import { useDeviceVerify } from '@/api/device.api'
import { Form } from '@/components/ui/form'
import { useAuth } from '@/hooks/use-auth'
import { zodResolver } from '@hookform/resolvers/zod'
import { useEffect, useMemo, useRef, useState } from 'react'
import { useForm } from 'react-hook-form'
import { useLocation, useNavigate, useParams } from 'react-router'
import { toast } from 'sonner'
import {
  DeviceVerifySchema,
  deviceVerifySchema,
  USER_CODE_CHARSET,
} from '../schemas/device-verify.schema'
import PageDeviceVerify, { DeviceVerifyStatus } from '../ui/page-device-verify'

// sessionStorage key used to bring the user back to this page after the
// OAuth login round-trip. We can't rely on the OAuth `state` channel because
// the existing login flow always lands on `/overview` — see
// `page-callback-feature.tsx`.
const POST_LOGIN_RETURN_KEY = 'ferriskey:post_login_return_to'

const USER_CODE_RE = new RegExp(
  `^[${USER_CODE_CHARSET}]{4}-?[${USER_CODE_CHARSET}]{4}$`,
  'i'
)

function normalisePrefill(raw: string | null): string {
  if (!raw) return ''
  const trimmed = raw.trim().toUpperCase()
  if (!USER_CODE_RE.test(trimmed)) return ''
  return trimmed.includes('-') ? trimmed : `${trimmed.slice(0, 4)}-${trimmed.slice(4)}`
}

export default function PageDeviceVerifyFeature() {
  const { realm_name } = useParams()
  const realm = realm_name ?? 'master'
  const location = useLocation()
  const navigate = useNavigate()
  const { isAuthenticated, isLoading: isAuthLoading } = useAuth()

  const prefill = useMemo(() => {
    const params = new URLSearchParams(location.search)
    return normalisePrefill(params.get('user_code'))
  }, [location.search])

  const form = useForm<DeviceVerifySchema>({
    resolver: zodResolver(deviceVerifySchema),
    mode: 'onChange',
    defaultValues: { user_code: prefill },
  })

  // Keep the form in sync if the prefill changes (e.g. user navigates back
  // with a different code in the URL).
  useEffect(() => {
    if (prefill) form.reset({ user_code: prefill })
  }, [prefill, form])

  const [status, setStatus] = useState<DeviceVerifyStatus>('idle')
  const [errorMessage, setErrorMessage] = useState<string | null>(null)
  const [pendingAction, setPendingAction] = useState<'approve' | 'deny' | null>(null)
  const redirectingToLogin = useRef(false)

  const { mutateAsync: verifyDevice } = useDeviceVerify()

  const onSubmit = async (
    values: DeviceVerifySchema,
    action: 'approve' | 'deny'
  ) => {
    setStatus('submitting')
    setErrorMessage(null)
    setPendingAction(action)
    try {
      const response = await verifyDevice({
        realm,
        data: { user_code: values.user_code, action },
      })
      setStatus(response.status === 'denied' ? 'denied' : 'approved')
    } catch (err) {
      const error = err as {
        status?: number
        data?: { error?: string; error_description?: string; redirect_uri?: string }
        message?: string
      }

      // 401: backend tells us to log in and come back. Stash the current URL
      // so the callback returns the user to the verification page once the
      // identity cookie is set, then kick off the standard OAuth login flow.
      if (error.status === 401) {
        if (redirectingToLogin.current) return
        redirectingToLogin.current = true
        try {
          sessionStorage.setItem(
            POST_LOGIN_RETURN_KEY,
            error.data?.redirect_uri ?? `${location.pathname}${location.search}`
          )
        } catch {
          // sessionStorage disabled (private mode, quota) — the user will
          // just land on /overview after login. Acceptable degradation.
        }
        navigate(`/realms/${realm}/authentication/login`, { replace: true })
        return
      }

      const description =
        error.data?.error_description ??
        error.message ??
        'Unable to verify this code. Please try again.'

      // 400 from the backend means unknown / expired / already-used code;
      // surface inline so the user can re-enter without losing the page.
      if (error.status === 400) {
        setStatus('error')
        setErrorMessage(description)
        setPendingAction(null)
        return
      }

      // Anything else: keep the form available and toast.
      setStatus('error')
      setErrorMessage(description)
      setPendingAction(null)
      toast.error(description)
    }
  }

  const onBackToStart = () => {
    setStatus('idle')
    setErrorMessage(null)
    setPendingAction(null)
    form.reset({ user_code: '' })
  }

  // Block render while auth state is hydrating to avoid a flash of the form
  // before we know whether to redirect to login.
  if (isAuthLoading) return null

  // Not authenticated yet — we still let the user *see* the form and type a
  // code; we'll bounce to login only when they actually submit (mirrors how
  // Google's device.google.com works). This keeps deep-links to
  // `?user_code=...` discoverable to logged-out users.
  void isAuthenticated

  return (
    <Form {...form}>
      <PageDeviceVerify
        status={status}
        errorMessage={errorMessage}
        pendingAction={pendingAction}
        onSubmit={onSubmit}
        onBackToStart={onBackToStart}
      />
    </Form>
  )
}

export { POST_LOGIN_RETURN_KEY }
