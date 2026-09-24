import { useCallback, useRef, useState } from 'react'
import { useRequestElevation } from '@/api/account-security.api'
import { apiErrorMessage, apiErrorReason } from '@/lib/api-error'

export const ELEVATION_REQUIRED = 'elevation_required'
export const PRIMARY_PROOF_REQUIRED = 'primary_proof_required'

const EXPIRY_MARGIN_MS = 5_000

export type ElevationProof = { password: string } | { otpCode: string }

export type ElevatedAction = (elevationId: string) => Promise<unknown>

interface LiveElevation {
  id: string
  expiresAt: number
  provedWithPassword: boolean
}

export interface ElevationState {
  isOpen: boolean
  isSubmitting: boolean
  error?: string
  requiresPassword: boolean
  run: (action: ElevatedAction, options?: { requiresPassword?: boolean }) => Promise<void>
  submit: (proof: ElevationProof) => Promise<void>
  cancel: () => void
  forget: () => void
}

export function useElevation(realm: string): ElevationState {
  const { mutateAsync: requestElevation } = useRequestElevation()

  const [isOpen, setIsOpen] = useState(false)
  const [isSubmitting, setIsSubmitting] = useState(false)
  const [error, setError] = useState<string | undefined>()
  const [requiresPassword, setRequiresPassword] = useState(false)

  const elevation = useRef<LiveElevation | null>(null)
  const pending = useRef<ElevatedAction | null>(null)

  const usable = useCallback((needsPassword: boolean) => {
    const live = elevation.current
    if (!live) return null
    if (live.expiresAt - EXPIRY_MARGIN_MS <= Date.now()) return null
    if (needsPassword && !live.provedWithPassword) return null
    return live
  }, [])

  const attempt = useCallback(
    async (action: ElevatedAction, elevationId: string) => {
      try {
        await action(elevationId)
        return true
      } catch (caught) {
        const reason = apiErrorReason(caught)
        if (reason === ELEVATION_REQUIRED || reason === PRIMARY_PROOF_REQUIRED) {
          elevation.current = null
          return false
        }
        throw caught
      }
    },
    []
  )

  const run = useCallback<ElevationState['run']>(
    async (action, options) => {
      const needsPassword = options?.requiresPassword ?? false
      const live = usable(needsPassword)

      if (live && (await attempt(action, live.id))) return

      pending.current = action
      setRequiresPassword(needsPassword)
      setError(undefined)
      setIsOpen(true)
    },
    [attempt, usable]
  )

  const submit = useCallback<ElevationState['submit']>(
    async (proof) => {
      setIsSubmitting(true)
      setError(undefined)

      try {
        const granted = await requestElevation({
          path: { realm_name: realm },
          body:
            'password' in proof ? { password: proof.password } : { otp_code: proof.otpCode },
        })

        elevation.current = {
          id: granted.elevation_id,
          expiresAt: new Date(granted.expires_at).getTime(),
          provedWithPassword: 'password' in proof,
        }

        const action = pending.current
        pending.current = null
        setIsOpen(false)

        if (action) await action(granted.elevation_id)
      } catch (caught) {
        setError(apiErrorMessage(caught))
      } finally {
        setIsSubmitting(false)
      }
    },
    [realm, requestElevation]
  )

  const cancel = useCallback(() => {
    pending.current = null
    setError(undefined)
    setIsOpen(false)
  }, [])

  const forget = useCallback(() => {
    elevation.current = null
  }, [])

  return { isOpen, isSubmitting, error, requiresPassword, run, submit, cancel, forget }
}
