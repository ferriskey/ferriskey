import type { NavigateFunction } from 'react-router'
import type { Schemas } from '@/api/api.client'
import { followServerRedirect } from '@/lib/follow-server-redirect'

export interface LoginContinuation {
  url?: string | null
  required_actions?: Schemas.RequiredAction[]
}

export function continueLogin(
  next: LoginContinuation | null | undefined,
  navigate: NavigateFunction,
  realmName: string | undefined,
): void {
  if (!next) return

  if (next.url) {
    followServerRedirect(next.url)
    return
  }

  const [first] = next.required_actions ?? []
  if (first) {
    navigate(
      `/realms/${realmName}/authentication/required-action?execution=${first.toUpperCase()}`,
    )
  }
}
