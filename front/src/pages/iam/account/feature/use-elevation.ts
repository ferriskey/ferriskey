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
  type Settlement = { ok: true } | { ok: false; error: unknown }

  const pending = useRef<{
    action: ElevatedAction
    settle: (outcome: Settlement) => void
  } | null>(null)

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

      return new Promise<void>((resolve, reject) => {
        pending.current?.settle({ ok: true })
        pending.current = {
          action,
          settle: (outcome) => (outcome.ok ? resolve() : reject(outcome.error)),
        }

        setRequiresPassword(needsPassword)
        setError(undefined)
        setIsOpen(true)
      })
    },
    [attempt, usable]
  )

  const submit = useCallback<ElevationState['submit']>(
    async (proof) => {
      setIsSubmitting(true)
      setError(undefined)

      let granted
      try {
        granted = await requestElevation({
          path: { realm_name: realm },
          body:
            'password' in proof ? { password: proof.password } : { otp_code: proof.otpCode },
        })
      } catch (caught) {
        setError(apiErrorMessage(caught))
        setIsSubmitting(false)
        return
      }

      elevation.current = {
        id: granted.elevation_id,
        expiresAt: new Date(granted.expires_at).getTime(),
        provedWithPassword: 'password' in proof,
      }

      const waiting = pending.current
      pending.current = null
      setIsOpen(false)
      setIsSubmitting(false)

      if (!waiting) return

      try {
        await waiting.action(granted.elevation_id)
        waiting.settle({ ok: true })
      } catch (caught) {
        waiting.settle({ ok: false, error: caught })
      }
    },
    [realm, requestElevation]
  )

  const cancel = useCallback(() => {
    const waiting = pending.current
    pending.current = null
    setError(undefined)
    setIsOpen(false)
    waiting?.settle({ ok: true })
  }, [])

  const forget = useCallback(() => {
    elevation.current = null
  }, [])

  return { isOpen, isSubmitting, error, requiresPassword, run, submit, cancel, forget }
}
