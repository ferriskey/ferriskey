export {
  formatDateTime,
  formatTime,
  formatRelative,
  formatTimestamp,
} from '@/shared/format-date'

import { Schemas } from '@/api/api.client'

import CompassFlow = Schemas.CompassFlow
import CompassFlowStep = Schemas.CompassFlowStep
import FlowStepName = Schemas.FlowStepName
import FlowStatus = Schemas.FlowStatus
import StepStatus = Schemas.StepStatus
import type { PillTone } from '@/components/kit'

export const stepLabels: Record<FlowStepName, string> = {
  authorize: 'Authorization request',
  credential_validation: 'Credential validation',
  mfa_challenge: 'Second factor',
  token_exchange: 'Token exchange',
  idp_redirect: 'Redirect to provider',
  idp_callback: 'Provider callback',
  finalize: 'Session opened',
  saml_authn_request: 'SAML AuthnRequest',
  saml_assertion: 'SAML assertion',
}

export const stepDescriptions: Record<FlowStepName, string> = {
  authorize: 'The client asked the realm for an authorization code.',
  credential_validation: 'The submitted secret was compared with the stored credential.',
  mfa_challenge: 'A second factor was requested and verified before any token was issued.',
  token_exchange: 'The code or the refresh token was traded for an access token.',
  idp_redirect: 'The browser was handed over to an external identity provider.',
  idp_callback: 'The external provider sent the browser back with its answer.',
  finalize: 'The session was persisted and the tokens returned to the client.',
  saml_authn_request: 'The service provider request was parsed and its signature checked.',
  saml_assertion: 'The signed assertion was built and posted back to the service provider.',
}

export const flowStatusTone: Record<FlowStatus, PillTone> = {
  success: 'success',
  failure: 'danger',
  expired: 'amber',
  pending: 'info',
}

export const stepStatusTone: Record<StepStatus, PillTone> = {
  success: 'success',
  failure: 'danger',
  skipped: 'neutral',
}

export const stepLabel = (step: CompassFlowStep) =>
  stepLabels[step.step_name] ?? step.step_name

export const formatDuration = (ms?: number | null) => {
  if (ms == null) return '—'
  return ms < 1000 ? `${ms} ms` : `${(ms / 1000).toFixed(1)} s`
}

export const orderedSteps = (flow: CompassFlow) =>
  [...flow.steps].sort(
    (a, b) => new Date(a.started_at).getTime() - new Date(b.started_at).getTime()
  )

export const failingStep = (flow: CompassFlow): CompassFlowStep | null =>
  orderedSteps(flow).find((step) => step.status === 'failure') ?? null
